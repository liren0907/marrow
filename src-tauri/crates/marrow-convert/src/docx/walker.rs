use std::collections::HashSet;

use quick_xml::events::Event;
use quick_xml::reader::Reader;

use crate::ConvertError;
use crate::ooxml::util::escape_md;

use super::attrs::{attr_val, attr_val_with_prefix};
use super::comments::render_comments_for;
use super::fields::{FieldState, parse_hyperlink_instr};
use super::render::{TableAcc, render_paragraph, render_table};
use super::types::{Ctx, Paragraph, Run};

/// Main walk over word/document.xml. Emits Markdown in order.
pub(super) fn walk_document(xml: &str, ctx: &Ctx) -> Result<String, ConvertError> {
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
    let mut in_rpr = false;

    // Field-code hyperlink state machine. Word emits old-style hyperlinks
    // as: <w:fldChar type="begin"> ... <w:instrText>HYPERLINK "url"</w:instrText>
    // ... <w:fldChar type="separate"> [display runs] <w:fldChar type="end">.
    let mut field_state = FieldState::None;
    let mut field_instr = String::new();
    let mut field_display: Vec<Run> = Vec::new();

    // Comment range tracking. When <w:commentRangeStart w:id="N"> opens
    // we mark the id as live; <w:commentRangeEnd w:id="N"> attaches the
    // comment to the paragraph that contains the End element.
    let mut active_comments: HashSet<String> = HashSet::new();

    // Table state (one level; nested tables are flattened to text)
    let mut table_stack: Vec<TableAcc> = Vec::new();
    let mut cur_cell_md: Option<String> = None;

    // Pictures: when <w:drawing> opens we look for the first
    // <a:blip r:embed="rIdN"> inside it and emit the image as a synthetic
    // run. `drawing_alt` carries any wp:docPr descr/title we encounter.
    let mut in_drawing = false;
    let mut drawing_alt: Option<String> = None;

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
                    b"drawing" => {
                        in_drawing = true;
                        drawing_alt = None;
                    }
                    b"instrText" => {
                        // Text events inside this element are treated as
                        // field-code body (HYPERLINK "...") rather than
                        // visible run text.
                        if matches!(field_state, FieldState::Begin) {
                            field_instr.clear();
                        }
                    }
                    _ => {}
                }
            }
            Ok(Event::Empty(e)) => {
                let name = e.local_name().as_ref().to_vec();
                match name.as_slice() {
                    b"b" if in_rpr => cur_run.bold = true,
                    b"i" if in_rpr => cur_run.italic = true,
                    b"u" if in_rpr => {
                        // <w:u w:val="single"|"double"|...> → underline.
                        // <w:u w:val="none"> explicitly disables. Treat
                        // any non-none / missing-val as on.
                        let val = attr_val(&e, b"val");
                        if val.as_deref() != Some("none") {
                            cur_run.underline = true;
                        }
                    }
                    b"strike" if in_rpr => cur_run.strike = true,
                    b"dstrike" if in_rpr => cur_run.strike = true,
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
                        // <w:br w:type="page"> → markdown thematic break,
                        // emitted between paragraphs. Plain breaks become
                        // a hard newline in the current run.
                        let ty = attr_val(&e, b"type");
                        if ty.as_deref() == Some("page") {
                            // Break the current paragraph, flush, emit ---,
                            // then continue. We can only do this cleanly
                            // when we're inside a paragraph.
                            if let Some(mut p) = cur_p.take() {
                                if in_run && !cur_run.text.is_empty() {
                                    p.runs.push(std::mem::take(&mut cur_run));
                                }
                                let line = render_paragraph(&p, ctx);
                                if let Some(cell) = cur_cell_md.as_mut() {
                                    let inline = line.replace('\n', " ").trim().to_string();
                                    if !cell.is_empty() && !inline.is_empty() {
                                        cell.push(' ');
                                    }
                                    cell.push_str(&inline);
                                } else {
                                    out.push_str(&line);
                                    out.push_str("\n\n---\n\n");
                                }
                                cur_p = Some(Paragraph::default());
                            }
                        } else if in_run {
                            cur_run.text.push('\n');
                        }
                    }
                    b"drawing" => {
                        in_drawing = false;
                        drawing_alt = None;
                    }
                    b"blip" if in_drawing => {
                        // r:embed → final image filename via image_rels.
                        let rid = attr_val_with_prefix(&e, b"r", b"embed")
                            .or_else(|| attr_val(&e, b"embed"));
                        if let (Some(rid), Some(p)) = (rid, cur_p.as_mut()) {
                            if let Some(name) = ctx.image_rels.get(&rid) {
                                let alt = drawing_alt
                                    .clone()
                                    .unwrap_or_default()
                                    .replace('[', "")
                                    .replace(']', "");
                                p.runs.push(Run {
                                    text: format!(
                                        "![{}](attachments/{})",
                                        alt, name
                                    ),
                                    ..Default::default()
                                });
                            }
                        }
                    }
                    b"docPr" if in_drawing => {
                        let descr = attr_val(&e, b"descr");
                        let title = attr_val(&e, b"title");
                        let nm = attr_val(&e, b"name");
                        drawing_alt = descr
                            .or(title)
                            .or(nm)
                            .filter(|s| !s.is_empty());
                    }
                    b"commentRangeStart" => {
                        if let Some(id) = attr_val(&e, b"id") {
                            active_comments.insert(id);
                        }
                    }
                    b"commentRangeEnd" => {
                        if let (Some(id), Some(p)) =
                            (attr_val(&e, b"id"), cur_p.as_mut())
                        {
                            if active_comments.remove(&id) {
                                p.comment_ids.push(id);
                            }
                        }
                    }
                    b"commentReference" => {
                        // Comment with no inline range — attach to the
                        // current paragraph anyway so the comment text
                        // still appears somewhere.
                        if let (Some(id), Some(p)) =
                            (attr_val(&e, b"id"), cur_p.as_mut())
                        {
                            if !p.comment_ids.iter().any(|x| x == &id) {
                                p.comment_ids.push(id);
                            }
                        }
                    }
                    b"footnoteReference" => {
                        if let (Some(id), Some(_)) = (attr_val(&e, b"id"), cur_p.as_mut())
                        {
                            cur_run.text.push_str(&format!("[^fn{id}]"));
                        }
                    }
                    b"endnoteReference" => {
                        if let (Some(id), Some(_)) = (attr_val(&e, b"id"), cur_p.as_mut())
                        {
                            cur_run.text.push_str(&format!("[^en{id}]"));
                        }
                    }
                    b"fldChar" => {
                        match attr_val(&e, b"fldCharType").as_deref() {
                            Some("begin") => {
                                field_state = FieldState::Begin;
                                field_instr.clear();
                                field_display.clear();
                            }
                            Some("separate") => {
                                field_state = FieldState::Separate;
                            }
                            Some("end") => {
                                if let Some(url) = parse_hyperlink_instr(&field_instr) {
                                    let merged: String = field_display
                                        .iter()
                                        .map(|r| r.text.as_str())
                                        .collect::<Vec<_>>()
                                        .join("");
                                    let text = escape_md(&merged);
                                    if let Some(p) = cur_p.as_mut() {
                                        p.runs.push(Run {
                                            text: format!("[{text}]({url})"),
                                            ..Default::default()
                                        });
                                    }
                                } else {
                                    // Not a HYPERLINK field — fall back to
                                    // the display runs as plain text.
                                    if let Some(p) = cur_p.as_mut() {
                                        for r in field_display.drain(..) {
                                            p.runs.push(r);
                                        }
                                    }
                                }
                                field_state = FieldState::None;
                                field_instr.clear();
                                field_display.clear();
                            }
                            _ => {}
                        }
                    }
                    _ => {}
                }
            }
            Ok(Event::Text(t)) => {
                let raw = match t.decode() {
                    Ok(r) => quick_xml::escape::unescape(&r)
                        .map(|c| c.into_owned())
                        .unwrap_or_else(|_| r.into_owned()),
                    Err(_) => continue,
                };
                if matches!(field_state, FieldState::Begin) {
                    // Inside <w:instrText> — accumulate field code body.
                    // (Plain <w:t> text inside the begin block is rare;
                    // we capture both.)
                    field_instr.push_str(&raw);
                } else if in_run {
                    cur_run.text.push_str(&raw);
                }
            }
            Ok(Event::End(e)) => {
                let name = e.local_name().as_ref().to_vec();
                match name.as_slice() {
                    b"rPr" => in_rpr = false,
                    b"r" => {
                        in_run = false;
                        if matches!(field_state, FieldState::Separate) {
                            // Display run for an in-flight HYPERLINK field.
                            if !cur_run.text.is_empty() {
                                field_display.push(std::mem::take(&mut cur_run));
                            } else {
                                cur_run = Run::default();
                            }
                        } else if !cur_run.text.is_empty() {
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
                                    ..Default::default()
                                });
                            }
                            hyperlink_buf.clear();
                        }
                    }
                    b"p" => {
                        if let Some(p) = cur_p.take() {
                            let line = render_paragraph(&p, ctx);
                            let comment_block = render_comments_for(&p, ctx);
                            if let Some(cell) = cur_cell_md.as_mut() {
                                let inline = line.replace('\n', " ").trim().to_string();
                                if !cell.is_empty() && !inline.is_empty() {
                                    cell.push(' ');
                                }
                                cell.push_str(&inline);
                                // Comments inside cells: append inline so
                                // the row stays valid markdown table syntax.
                                if !comment_block.is_empty() {
                                    cell.push(' ');
                                    cell.push_str(&comment_block.replace('\n', " "));
                                }
                            } else {
                                out.push_str(&line);
                                out.push('\n');
                                if !comment_block.is_empty() {
                                    out.push_str(&comment_block);
                                    out.push('\n');
                                }
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
                                let _ = md;
                            } else {
                                out.push('\n');
                                out.push_str(&md);
                                out.push('\n');
                            }
                        }
                    }
                    b"drawing" => {
                        in_drawing = false;
                        drawing_alt = None;
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
