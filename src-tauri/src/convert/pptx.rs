use std::collections::HashMap;

use quick_xml::events::Event;
use quick_xml::reader::Reader;

use crate::convert::ConvertError;
use crate::convert::ooxml_dml::{Paragraph, TextBodyParser, render_paragraph};
use crate::convert::ooxml_util::{
    list_zip_names, open_zip, parse_rels, post_process, read_zip_text,
};

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

struct Slide {
    /// `<p:sld show="0">` — slide marked hidden in the deck.
    hidden: bool,
    shapes: Vec<Shape>,
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

        let mut slide = parse_slide(&slide_xml, &rels)?;
        if slide.hidden {
            // Preserve numbering so readers can map output back to the
            // source deck, but omit body content.
            out.push_str(&format!("<!-- Slide {} (hidden) -->\n\n", idx + 1));
            continue;
        }
        slide.shapes.sort_by_key(|s| (s.y, s.x));

        out.push_str(&format!("<!-- Slide {} -->\n", idx + 1));
        let mut emitted_any = false;
        for shape in &slide.shapes {
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

/// Walks one `slide<N>.xml` and groups its content into ordered shapes.
///
/// Slide-level concerns (shape boundaries, placeholder type, position,
/// tables) live here; everything inside a `<txBody>` is delegated to
/// [`TextBodyParser`] which collects paragraphs that we drain at `</tc>`
/// (cell) or `</sp>` (shape body).
fn parse_slide(
    xml: &str,
    rels: &HashMap<String, String>,
) -> Result<Slide, ConvertError> {
    let mut reader = Reader::from_str(xml);
    reader.config_mut().trim_text(false);
    let mut buf = Vec::new();

    let mut shapes: Vec<Shape> = Vec::new();
    let mut hidden = false;

    let mut in_sp = false;
    let mut in_tbl = false;
    let mut sp_offset: (i64, i64) = (0, 0);
    let mut sp_is_title = false;
    // `<p:cNvPr name="Title 1">` is the most reliable fallback when the
    // placeholder lacks an explicit `type="title"` (some templates and
    // tools omit it). Captured per-shape and consumed at `</p:sp>`.
    let mut sp_name: Option<String> = None;

    let mut parser = TextBodyParser::new();

    let mut tbl_rows: Vec<Vec<String>> = Vec::new();
    let mut cur_row: Vec<String> = Vec::new();

    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(e)) => {
                let name = e.local_name().as_ref().to_vec();
                match name.as_slice() {
                    b"sld" => {
                        if let Some(v) = attr_val(&e, b"show") {
                            if v == "0" || v == "false" {
                                hidden = true;
                            }
                        }
                    }
                    b"sp" => {
                        in_sp = true;
                        sp_offset = (0, 0);
                        sp_is_title = false;
                        sp_name = None;
                    }
                    b"cNvPr" => {
                        if in_sp {
                            sp_name = attr_val(&e, b"name");
                        }
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
                    b"tbl" => {
                        in_tbl = true;
                        tbl_rows.clear();
                    }
                    b"tr" => {
                        cur_row.clear();
                    }
                    b"tc" => {}
                    other => parser.on_start(other, &e, rels),
                }
            }
            Ok(Event::Empty(e)) => {
                let name = e.local_name().as_ref().to_vec();
                match name.as_slice() {
                    b"cNvPr" => {
                        if in_sp {
                            sp_name = attr_val(&e, b"name");
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
                    b"ph" => {
                        if let Some(t) = attr_val(&e, b"type") {
                            if t == "title" || t == "ctrTitle" {
                                sp_is_title = true;
                            }
                        }
                    }
                    other => parser.on_empty(other, &e, rels),
                }
            }
            Ok(Event::Text(t)) => parser.on_text(&t),
            Ok(Event::End(e)) => {
                let name = e.local_name().as_ref().to_vec();
                match name.as_slice() {
                    b"tc" => {
                        let cell_paragraphs = parser.take_paragraphs();
                        let cell_md = cell_paragraphs
                            .iter()
                            .map(|p| {
                                render_paragraph(p)
                                    .replace('\n', " ")
                                    .trim()
                                    .to_string()
                            })
                            .filter(|s| !s.is_empty())
                            .collect::<Vec<_>>()
                            .join(" ");
                        cur_row.push(cell_md);
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
                        let sp_paragraphs = parser.take_paragraphs();
                        let is_title = sp_is_title
                            || sp_name
                                .as_deref()
                                .map(|n| {
                                    let lower = n.trim().to_ascii_lowercase();
                                    lower.starts_with("title")
                                })
                                .unwrap_or(false);
                        sp_name = None;
                        if is_title {
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
                                kind: ShapeKind::Body(sp_paragraphs),
                            });
                        }
                    }
                    other => parser.on_end(other),
                }
            }
            Ok(Event::Eof) => break,
            Err(e) => return Err(ConvertError::Xml(format!("{e}"))),
            _ => {}
        }
        buf.clear();
    }

    Ok(Slide { hidden, shapes })
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
