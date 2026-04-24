use std::collections::HashMap;

use quick_xml::events::Event;
use quick_xml::reader::Reader;

use crate::convert::ConvertError;
use crate::convert::ooxml_util::{
    escape_md, list_zip_names, open_zip, parse_rels, post_process, read_zip_text, wrap_run,
};

#[derive(Default, Clone)]
struct Run {
    text: String,
    bold: bool,
    italic: bool,
    hyperlink: Option<String>,
}

#[derive(Default)]
struct Paragraph {
    bullet: BulletKind,
    ilvl: usize,
    runs: Vec<Run>,
}

#[derive(Default, Clone, Copy, PartialEq)]
enum BulletKind {
    #[default]
    None,
    Char,
    AutoNum,
}

enum ShapeKind {
    /// Title or centerTitle placeholder — emit as heading.
    Title(String),
    /// Body text: ordered list of paragraphs.
    Body(Vec<Paragraph>),
    /// Table: pre-rendered cell strings.
    Table(Vec<Vec<String>>),
}

struct Shape {
    y: i64,
    x: i64,
    kind: ShapeKind,
}

pub fn pptx_to_markdown(bytes: &[u8]) -> Result<String, ConvertError> {
    let mut zip = open_zip(bytes)?;

    // Collect slide paths by numeric ordinal.
    let mut slide_paths: Vec<String> = list_zip_names(&zip, "ppt/slides/slide")
        .into_iter()
        .filter(|n| n.ends_with(".xml") && !n.contains("_rels"))
        .collect();
    slide_paths.sort_by_key(|p| slide_ordinal(p).unwrap_or(i64::MAX));

    let mut out = String::new();
    for (idx, path) in slide_paths.iter().enumerate() {
        let slide_xml = match read_zip_text(&mut zip, path)? {
            Some(s) => s,
            None => continue,
        };
        let rels_path = path
            .rsplit_once('/')
            .map(|(dir, file)| format!("{dir}/_rels/{file}.rels"))
            .unwrap_or_default();
        let rels = read_zip_text(&mut zip, &rels_path)?
            .map(|s| parse_rels(&s))
            .unwrap_or_default();

        let mut shapes = parse_slide(&slide_xml, &rels)?;
        shapes.sort_by_key(|s| (s.y, s.x));

        out.push_str(&format!("<!-- Slide {} -->\n", idx + 1));
        let mut emitted_any = false;
        // Emit title(s) first, then body, then table — but respect the
        // y/x sort; titles naturally tend to be at top so this usually
        // already holds.
        for shape in &shapes {
            match &shape.kind {
                ShapeKind::Title(t) => {
                    let trimmed = t.trim();
                    if !trimmed.is_empty() {
                        out.push_str(&format!("## {trimmed}\n\n"));
                        emitted_any = true;
                    }
                }
                ShapeKind::Body(paras) => {
                    for p in paras {
                        let line = render_paragraph(p);
                        if !line.trim().is_empty() {
                            out.push_str(&line);
                            out.push('\n');
                            emitted_any = true;
                        }
                    }
                    if !paras.is_empty() {
                        out.push('\n');
                    }
                }
                ShapeKind::Table(rows) => {
                    let rendered = render_table(rows);
                    if !rendered.is_empty() {
                        out.push_str(&rendered);
                        out.push('\n');
                        emitted_any = true;
                    }
                }
            }
        }
        if !emitted_any {
            out.push('\n');
        }
    }

    Ok(post_process(out))
}

fn slide_ordinal(path: &str) -> Option<i64> {
    // ppt/slides/slide<N>.xml → N
    let name = path.rsplit('/').next()?;
    let stem = name.strip_prefix("slide")?.strip_suffix(".xml")?;
    stem.parse().ok()
}

fn attr_val(e: &quick_xml::events::BytesStart, key: &[u8]) -> Option<String> {
    for attr in e.attributes().flatten() {
        if attr.key.local_name().as_ref() == key {
            return attr.unescape_value().ok().map(|c| c.into_owned());
        }
    }
    None
}

fn parse_slide(
    xml: &str,
    rels: &HashMap<String, String>,
) -> Result<Vec<Shape>, ConvertError> {
    let mut reader = Reader::from_str(xml);
    reader.config_mut().trim_text(false);
    let mut buf = Vec::new();

    let mut shapes: Vec<Shape> = Vec::new();

    // Per-shape state
    let mut in_sp = false;
    let mut in_tbl = false;
    let mut sp_offset: (i64, i64) = (0, 0);
    let mut sp_is_title = false;
    let mut sp_paragraphs: Vec<Paragraph> = Vec::new();

    // Per-paragraph / run state
    let mut cur_p: Option<Paragraph> = None;
    let mut in_run = false;
    let mut cur_run = Run::default();
    let mut pending_hyperlink: Option<String> = None;

    // Table state
    let mut tbl_rows: Vec<Vec<String>> = Vec::new();
    let mut cur_row: Vec<String> = Vec::new();
    let mut cur_cell_paragraphs: Vec<Paragraph> = Vec::new();
    let mut in_cell = false;

    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(e)) => match e.local_name().as_ref() {
                b"sp" => {
                    in_sp = true;
                    sp_offset = (0, 0);
                    sp_is_title = false;
                    sp_paragraphs.clear();
                }
                b"ph" => {
                    if let Some(t) = attr_val(&e, b"type") {
                        if t == "title" || t == "ctrTitle" {
                            sp_is_title = true;
                        }
                    }
                }
                b"off" => {
                    let x = attr_val(&e, b"x")
                        .and_then(|v| v.parse::<i64>().ok())
                        .unwrap_or(0);
                    let y = attr_val(&e, b"y")
                        .and_then(|v| v.parse::<i64>().ok())
                        .unwrap_or(0);
                    if in_sp || in_tbl {
                        sp_offset = (y, x);
                    }
                }
                b"p" => {
                    cur_p = Some(Paragraph::default());
                }
                b"pPr" => {
                    if let Some(p) = cur_p.as_mut() {
                        if let Some(v) = attr_val(&e, b"lvl") {
                            p.ilvl = v.parse().unwrap_or(0);
                        }
                    }
                }
                b"buChar" => {
                    if let Some(p) = cur_p.as_mut() {
                        p.bullet = BulletKind::Char;
                    }
                }
                b"buAutoNum" => {
                    if let Some(p) = cur_p.as_mut() {
                        p.bullet = BulletKind::AutoNum;
                    }
                }
                b"buNone" => {
                    if let Some(p) = cur_p.as_mut() {
                        p.bullet = BulletKind::None;
                    }
                }
                b"r" => {
                    in_run = true;
                    cur_run = Run {
                        hyperlink: pending_hyperlink.clone(),
                        ..Default::default()
                    };
                }
                b"rPr" => {
                    if let Some(v) = attr_val(&e, b"b") {
                        if v == "1" || v == "true" {
                            cur_run.bold = true;
                        }
                    }
                    if let Some(v) = attr_val(&e, b"i") {
                        if v == "1" || v == "true" {
                            cur_run.italic = true;
                        }
                    }
                }
                b"hlinkClick" => {
                    if let Some(id) = attr_val(&e, b"id") {
                        if let Some(url) = rels.get(&id) {
                            cur_run.hyperlink = Some(url.clone());
                            pending_hyperlink = Some(url.clone());
                        }
                    }
                }
                b"tbl" => {
                    in_tbl = true;
                    tbl_rows.clear();
                }
                b"tr" => {
                    cur_row.clear();
                }
                b"tc" => {
                    in_cell = true;
                    cur_cell_paragraphs.clear();
                }
                _ => {}
            },
            Ok(Event::Empty(e)) => match e.local_name().as_ref() {
                b"off" => {
                    let x = attr_val(&e, b"x")
                        .and_then(|v| v.parse::<i64>().ok())
                        .unwrap_or(0);
                    let y = attr_val(&e, b"y")
                        .and_then(|v| v.parse::<i64>().ok())
                        .unwrap_or(0);
                    if in_sp || in_tbl {
                        sp_offset = (y, x);
                    }
                }
                b"buChar" => {
                    if let Some(p) = cur_p.as_mut() {
                        p.bullet = BulletKind::Char;
                    }
                }
                b"buAutoNum" => {
                    if let Some(p) = cur_p.as_mut() {
                        p.bullet = BulletKind::AutoNum;
                    }
                }
                b"buNone" => {
                    if let Some(p) = cur_p.as_mut() {
                        p.bullet = BulletKind::None;
                    }
                }
                b"br" => {
                    if in_run {
                        cur_run.text.push('\n');
                    }
                }
                b"ph" => {
                    if let Some(t) = attr_val(&e, b"type") {
                        if t == "title" || t == "ctrTitle" {
                            sp_is_title = true;
                        }
                    }
                }
                b"hlinkClick" => {
                    if let Some(id) = attr_val(&e, b"id") {
                        if let Some(url) = rels.get(&id) {
                            cur_run.hyperlink = Some(url.clone());
                        }
                    }
                }
                _ => {}
            },
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
            Ok(Event::End(e)) => match e.local_name().as_ref() {
                b"r" => {
                    in_run = false;
                    if !cur_run.text.is_empty() {
                        if let Some(p) = cur_p.as_mut() {
                            p.runs.push(std::mem::take(&mut cur_run));
                        }
                    } else {
                        cur_run = Run::default();
                    }
                }
                b"hlinkClick" => {
                    // The hyperlink attribute lives on the run's rPr; scoped to this run.
                    pending_hyperlink = None;
                }
                b"p" => {
                    if let Some(p) = cur_p.take() {
                        if in_cell {
                            cur_cell_paragraphs.push(p);
                        } else if in_sp {
                            sp_paragraphs.push(p);
                        }
                    }
                }
                b"tc" => {
                    let cell_md = cur_cell_paragraphs
                        .iter()
                        .map(|p| render_paragraph(p).replace('\n', " ").trim().to_string())
                        .filter(|s| !s.is_empty())
                        .collect::<Vec<_>>()
                        .join(" ");
                    cur_row.push(cell_md);
                    cur_cell_paragraphs.clear();
                    in_cell = false;
                }
                b"tr" => {
                    tbl_rows.push(std::mem::take(&mut cur_row));
                }
                b"tbl" => {
                    in_tbl = false;
                    shapes.push(Shape {
                        y: sp_offset.0,
                        x: sp_offset.1,
                        kind: ShapeKind::Table(std::mem::take(&mut tbl_rows)),
                    });
                }
                b"sp" => {
                    in_sp = false;
                    if sp_is_title {
                        let text = sp_paragraphs
                            .iter()
                            .map(|p| {
                                p.runs
                                    .iter()
                                    .map(|r| r.text.as_str())
                                    .collect::<Vec<_>>()
                                    .join("")
                            })
                            .collect::<Vec<_>>()
                            .join(" ");
                        shapes.push(Shape {
                            y: sp_offset.0,
                            x: sp_offset.1,
                            kind: ShapeKind::Title(text),
                        });
                    } else if !sp_paragraphs.is_empty() {
                        shapes.push(Shape {
                            y: sp_offset.0,
                            x: sp_offset.1,
                            kind: ShapeKind::Body(std::mem::take(&mut sp_paragraphs)),
                        });
                    }
                    sp_paragraphs.clear();
                }
                _ => {}
            },
            Ok(Event::Eof) => break,
            Err(e) => return Err(ConvertError::Xml(format!("{e}"))),
            _ => {}
        }
        buf.clear();
    }

    Ok(shapes)
}

fn render_paragraph(p: &Paragraph) -> String {
    // Merge adjacent runs with identical formatting (and no hyperlink
    // transitions) to avoid `**a****b**` style joins.
    let merged = merge_adjacent_runs(&p.runs);
    let mut body = String::new();
    for r in &merged {
        let text_escaped = escape_md(&r.text);
        let wrapped = wrap_run(&text_escaped, r.bold, r.italic);
        if let Some(url) = &r.hyperlink {
            body.push_str(&format!("[{}]({})", wrapped, url));
        } else {
            body.push_str(&wrapped);
        }
    }
    let body = body.trim().to_string();

    if body.is_empty() {
        return String::new();
    }

    match p.bullet {
        BulletKind::None => body,
        BulletKind::Char => {
            let indent = "  ".repeat(p.ilvl);
            format!("{indent}- {body}")
        }
        BulletKind::AutoNum => {
            let indent = "  ".repeat(p.ilvl);
            format!("{indent}1. {body}")
        }
    }
}

fn render_table(rows: &[Vec<String>]) -> String {
    if rows.is_empty() {
        return String::new();
    }
    let cols = rows.iter().map(|r| r.len()).max().unwrap_or(0);
    if cols == 0 {
        return String::new();
    }
    let mut out = String::new();
    let header = normalize_row(&rows[0], cols);
    out.push_str(&format_row(&header));
    out.push('\n');
    out.push_str(&format_row(&vec!["---".to_string(); cols]));
    out.push('\n');
    for row in rows.iter().skip(1) {
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

fn merge_adjacent_runs(runs: &[Run]) -> Vec<Run> {
    let mut out: Vec<Run> = Vec::with_capacity(runs.len());
    for r in runs {
        if let Some(last) = out.last_mut() {
            if last.bold == r.bold
                && last.italic == r.italic
                && last.hyperlink == r.hyperlink
            {
                last.text.push_str(&r.text);
                continue;
            }
        }
        out.push(r.clone());
    }
    out
}
