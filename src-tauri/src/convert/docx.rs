use std::collections::HashMap;

use quick_xml::events::Event;
use quick_xml::reader::Reader;

use crate::convert::ConvertError;
use crate::convert::ooxml_util::{
    escape_md, open_zip, parse_rels, post_process, read_zip_text, wrap_run,
};

#[derive(Clone, Copy, Debug)]
enum ListKind {
    Bullet,
    Ordered,
}

struct Ctx {
    rels: HashMap<String, String>,
    styles: HashMap<String, usize>, // style_id → heading level (1..=6)
    numbering: HashMap<String, ListKind>, // numId → list kind
}

#[derive(Default, Clone)]
struct Run {
    text: String,
    bold: bool,
    italic: bool,
}

#[derive(Default)]
struct Paragraph {
    style_id: Option<String>,
    num_id: Option<String>,
    ilvl: usize,
    runs: Vec<Run>,
}

pub fn docx_to_markdown(bytes: &[u8]) -> Result<String, ConvertError> {
    let mut zip = open_zip(bytes)?;
    let rels = read_zip_text(&mut zip, "word/_rels/document.xml.rels")?
        .map(|s| parse_rels(&s))
        .unwrap_or_default();
    let styles = read_zip_text(&mut zip, "word/styles.xml")?
        .map(|s| parse_styles(&s))
        .unwrap_or_default();
    let numbering = read_zip_text(&mut zip, "word/numbering.xml")?
        .map(|s| parse_numbering(&s))
        .unwrap_or_default();
    let body = read_zip_text(&mut zip, "word/document.xml")?
        .ok_or_else(|| ConvertError::Zip("missing word/document.xml".into()))?;

    let ctx = Ctx {
        rels,
        styles,
        numbering,
    };
    let md = walk_document(&body, &ctx)?;
    Ok(post_process(md))
}

/// styles.xml: find `<w:style w:styleId="X">` with inner `<w:name w:val="heading N">`
/// or `<w:name w:val="Heading N">` → X maps to N.
fn parse_styles(xml: &str) -> HashMap<String, usize> {
    let mut reader = Reader::from_str(xml);
    reader.config_mut().trim_text(false);
    let mut map = HashMap::new();
    let mut buf = Vec::new();
    let mut current_id: Option<String> = None;
    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(e)) if e.local_name().as_ref() == b"style" => {
                current_id = attr_val(&e, b"styleId");
            }
            Ok(Event::End(e)) if e.local_name().as_ref() == b"style" => {
                current_id = None;
            }
            Ok(Event::Empty(e)) | Ok(Event::Start(e))
                if e.local_name().as_ref() == b"name" =>
            {
                if let (Some(id), Some(v)) = (current_id.as_deref(), attr_val(&e, b"val")) {
                    if let Some(level) = parse_heading_level(&v) {
                        map.insert(id.to_string(), level);
                    }
                }
            }
            Ok(Event::Eof) => break,
            Err(_) => break,
            _ => {}
        }
        buf.clear();
    }
    // Also accept the conventional IDs ("Heading1"..="Heading9") directly.
    for n in 1..=6 {
        map.entry(format!("Heading{n}"))
            .or_insert(n);
        map.entry(format!("heading {n}"))
            .or_insert(n);
    }
    map
}

fn parse_heading_level(name: &str) -> Option<usize> {
    let lower = name.to_ascii_lowercase();
    let rest = lower.strip_prefix("heading ").or_else(|| lower.strip_prefix("heading"))?;
    let n: usize = rest.trim().parse().ok()?;
    if (1..=6).contains(&n) { Some(n) } else { None }
}

/// numbering.xml: gather numId → ListKind. Approximate: if `<w:numFmt val="bullet">`
/// anywhere inside the definition, treat as Bullet; otherwise Ordered.
fn parse_numbering(xml: &str) -> HashMap<String, ListKind> {
    let mut reader = Reader::from_str(xml);
    reader.config_mut().trim_text(false);
    let mut map = HashMap::new();
    let mut buf = Vec::new();
    let mut current_num: Option<String> = None;
    let mut saw_bullet = false;
    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(e)) if e.local_name().as_ref() == b"num" => {
                current_num = attr_val(&e, b"numId");
                saw_bullet = false;
            }
            Ok(Event::End(e)) if e.local_name().as_ref() == b"num" => {
                if let Some(id) = current_num.take() {
                    map.insert(
                        id,
                        if saw_bullet {
                            ListKind::Bullet
                        } else {
                            ListKind::Ordered
                        },
                    );
                }
            }
            Ok(Event::Empty(e)) | Ok(Event::Start(e))
                if e.local_name().as_ref() == b"numFmt" =>
            {
                if let Some(v) = attr_val(&e, b"val") {
                    if v == "bullet" {
                        saw_bullet = true;
                    }
                }
            }
            Ok(Event::Eof) => break,
            Err(_) => break,
            _ => {}
        }
        buf.clear();
    }
    map
}

fn attr_val(e: &quick_xml::events::BytesStart, key: &[u8]) -> Option<String> {
    for attr in e.attributes().flatten() {
        if attr.key.local_name().as_ref() == key {
            return attr.unescape_value().ok().map(|c| c.into_owned());
        }
    }
    None
}

/// Main walk over word/document.xml. Emits Markdown in order.
fn walk_document(xml: &str, ctx: &Ctx) -> Result<String, ConvertError> {
    let mut reader = Reader::from_str(xml);
    reader.config_mut().trim_text(false);
    let mut buf = Vec::new();
    let mut out = String::new();

    // Paragraph state
    let mut cur_p: Option<Paragraph> = None;
    let mut in_run = false;
    let mut cur_run = Run::default();
    // When true, text inside <w:r> goes into a hyperlink buffer instead of cur_p
    let mut in_hyperlink: Option<String> = None;
    let mut hyperlink_buf: Vec<Run> = Vec::new();
    // Are we currently inside <w:rPr>? bold/italic only apply to the
    // enclosing run when found here.
    let mut in_rpr = false;

    // Table state (one level; nested tables are flattened to text)
    let mut table_stack: Vec<TableAcc> = Vec::new();
    let mut cur_cell_md: Option<String> = None;
    // Remember the "pre-cell" paragraph so a cell's paragraphs go to cur_cell_md instead.

    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(e)) => {
                let name = e.local_name().as_ref().to_vec();
                match name.as_slice() {
                    b"p" => {
                        cur_p = Some(Paragraph::default());
                    }
                    b"pStyle" => {
                        if let Some(p) = cur_p.as_mut() {
                            p.style_id = attr_val(&e, b"val");
                        }
                    }
                    b"numId" => {
                        if let Some(p) = cur_p.as_mut() {
                            p.num_id = attr_val(&e, b"val");
                        }
                    }
                    b"ilvl" => {
                        if let Some(p) = cur_p.as_mut() {
                            if let Some(v) = attr_val(&e, b"val") {
                                p.ilvl = v.parse().unwrap_or(0);
                            }
                        }
                    }
                    b"r" => {
                        in_run = true;
                        cur_run = Run::default();
                    }
                    b"rPr" => in_rpr = true,
                    b"hyperlink" => {
                        let rid = attr_val(&e, b"id");
                        in_hyperlink = rid.and_then(|id| ctx.rels.get(&id).cloned());
                        hyperlink_buf.clear();
                    }
                    b"tbl" => table_stack.push(TableAcc::default()),
                    b"tr" => {
                        if let Some(t) = table_stack.last_mut() {
                            t.rows.push(Vec::new());
                        }
                    }
                    b"tc" => {
                        cur_cell_md = Some(String::new());
                    }
                    _ => {}
                }
            }
            Ok(Event::Empty(e)) => {
                let name = e.local_name().as_ref().to_vec();
                match name.as_slice() {
                    b"b" if in_rpr => cur_run.bold = true,
                    b"i" if in_rpr => cur_run.italic = true,
                    b"pStyle" => {
                        if let Some(p) = cur_p.as_mut() {
                            p.style_id = attr_val(&e, b"val");
                        }
                    }
                    b"numId" => {
                        if let Some(p) = cur_p.as_mut() {
                            p.num_id = attr_val(&e, b"val");
                        }
                    }
                    b"ilvl" => {
                        if let Some(p) = cur_p.as_mut() {
                            if let Some(v) = attr_val(&e, b"val") {
                                p.ilvl = v.parse().unwrap_or(0);
                            }
                        }
                    }
                    b"tab" => {
                        if in_run {
                            cur_run.text.push(' ');
                        }
                    }
                    b"br" => {
                        if in_run {
                            cur_run.text.push('\n');
                        }
                    }
                    _ => {}
                }
            }
            Ok(Event::Text(t)) => {
                if in_run {
                    if let Ok(raw) = t.decode() {
                        let un = quick_xml::escape::unescape(&raw)
                            .map(|c| c.into_owned())
                            .unwrap_or_else(|_| raw.into_owned());
                        cur_run.text.push_str(&un);
                    }
                }
            }
            Ok(Event::End(e)) => {
                let name = e.local_name().as_ref().to_vec();
                match name.as_slice() {
                    b"rPr" => in_rpr = false,
                    b"r" => {
                        in_run = false;
                        if !cur_run.text.is_empty() {
                            if in_hyperlink.is_some() {
                                hyperlink_buf.push(std::mem::take(&mut cur_run));
                            } else if let Some(p) = cur_p.as_mut() {
                                p.runs.push(std::mem::take(&mut cur_run));
                            }
                        } else {
                            cur_run = Run::default();
                        }
                    }
                    b"hyperlink" => {
                        if let Some(url) = in_hyperlink.take() {
                            let merged: String = hyperlink_buf
                                .iter()
                                .map(|r| r.text.as_str())
                                .collect::<Vec<_>>()
                                .join("");
                            let text = escape_md(&merged);
                            if let Some(p) = cur_p.as_mut() {
                                p.runs.push(Run {
                                    text: format!("[{text}]({url})"),
                                    bold: false,
                                    italic: false,
                                });
                            }
                            hyperlink_buf.clear();
                        }
                    }
                    b"p" => {
                        if let Some(p) = cur_p.take() {
                            let line = render_paragraph(&p, ctx);
                            if let Some(cell) = cur_cell_md.as_mut() {
                                // Inside a table cell: flatten to inline.
                                let inline = line.replace('\n', " ").trim().to_string();
                                if !cell.is_empty() && !inline.is_empty() {
                                    cell.push(' ');
                                }
                                cell.push_str(&inline);
                            } else {
                                out.push_str(&line);
                                out.push('\n');
                            }
                        }
                    }
                    b"tc" => {
                        if let Some(cell) = cur_cell_md.take() {
                            if let Some(t) = table_stack.last_mut() {
                                if let Some(row) = t.rows.last_mut() {
                                    row.push(cell);
                                }
                            }
                        }
                    }
                    b"tbl" => {
                        if let Some(tbl) = table_stack.pop() {
                            let md = render_table(&tbl);
                            if let Some(cell) = cur_cell_md.as_mut() {
                                // Nested table: flatten to plain text of cells joined.
                                let flat = tbl
                                    .rows
                                    .iter()
                                    .flat_map(|r| r.iter())
                                    .cloned()
                                    .collect::<Vec<_>>()
                                    .join(" ");
                                if !cell.is_empty() {
                                    cell.push(' ');
                                }
                                cell.push_str(&flat);
                                // Suppress the outer md for the nested case.
                                let _ = md;
                            } else {
                                out.push('\n');
                                out.push_str(&md);
                                out.push('\n');
                            }
                        }
                    }
                    _ => {}
                }
            }
            Ok(Event::Eof) => break,
            Err(e) => return Err(ConvertError::Xml(format!("{e}"))),
            _ => {}
        }
        buf.clear();
    }
    Ok(out)
}

#[derive(Default)]
struct TableAcc {
    rows: Vec<Vec<String>>,
}

fn render_table(t: &TableAcc) -> String {
    if t.rows.is_empty() {
        return String::new();
    }
    let cols = t.rows.iter().map(|r| r.len()).max().unwrap_or(0);
    if cols == 0 {
        return String::new();
    }
    let mut out = String::new();
    let header = normalize_row(&t.rows[0], cols);
    out.push_str(&format_row(&header));
    out.push('\n');
    out.push_str(&format_row(&vec!["---".to_string(); cols]));
    out.push('\n');
    for row in t.rows.iter().skip(1) {
        out.push_str(&format_row(&normalize_row(row, cols)));
        out.push('\n');
    }
    out
}

fn normalize_row(row: &[String], cols: usize) -> Vec<String> {
    let mut v: Vec<String> = row
        .iter()
        .map(|c| c.replace('|', "\\|").replace('\n', " ").trim().to_string())
        .collect();
    while v.len() < cols {
        v.push(String::new());
    }
    v
}

fn format_row(cells: &[String]) -> String {
    let mut s = String::from("|");
    for c in cells {
        s.push(' ');
        s.push_str(c);
        s.push_str(" |");
    }
    s
}

fn render_paragraph(p: &Paragraph, ctx: &Ctx) -> String {
    // Merge adjacent runs with identical formatting.
    let merged = merge_adjacent_runs(&p.runs);
    let mut body = String::new();
    for r in &merged {
        body.push_str(&wrap_run(&escape_md_preserve_brackets(&r.text), r.bold, r.italic));
    }
    let body = body.trim().to_string();

    // Heading?
    if let Some(id) = &p.style_id {
        if let Some(level) = ctx.styles.get(id).copied() {
            let hash = "#".repeat(level);
            return if body.is_empty() {
                String::new()
            } else {
                format!("\n{hash} {body}\n")
            };
        }
    }

    // List?
    if let Some(num_id) = &p.num_id {
        let kind = ctx.numbering.get(num_id).copied().unwrap_or(ListKind::Bullet);
        let marker = match kind {
            ListKind::Bullet => "-".to_string(),
            ListKind::Ordered => "1.".to_string(),
        };
        let indent = "  ".repeat(p.ilvl);
        return format!("{indent}{marker} {body}");
    }

    if body.is_empty() {
        String::new()
    } else {
        format!("{body}\n")
    }
}

/// Same as escape_md in ooxml_util but hyperlink runs arrive pre-formatted
/// with `[..](..)` — don't double-escape brackets if the text is clearly
/// already a link form. Heuristic: skip escaping when text matches `[*](*)`.
fn escape_md_preserve_brackets(text: &str) -> String {
    if looks_like_link(text) {
        return text.to_string();
    }
    escape_md(text)
}

fn looks_like_link(text: &str) -> bool {
    if !text.starts_with('[') {
        return false;
    }
    // crude: has "](" after first char and ends with ')'
    text.contains("](") && text.ends_with(')')
}

fn merge_adjacent_runs(runs: &[Run]) -> Vec<Run> {
    let mut out: Vec<Run> = Vec::with_capacity(runs.len());
    for r in runs {
        if let Some(last) = out.last_mut() {
            if last.bold == r.bold && last.italic == r.italic {
                last.text.push_str(&r.text);
                continue;
            }
        }
        out.push(r.clone());
    }
    out
}
