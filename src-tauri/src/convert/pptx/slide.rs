use std::collections::HashMap;

use quick_xml::events::{BytesStart, Event};
use quick_xml::reader::Reader;

use crate::convert::ConvertError;
use crate::convert::ooxml_dml::{TextBodyParser, render_paragraph_with_bullet};

use super::attrs::attr_val;
use super::types::{Shape, ShapeKind, Slide, StyleCategory};

/// Pull the `(y, x)` integer pair off an `<a:off>` / `<a:chOff>` element.
/// Missing or unparseable attributes default to 0.
fn read_xy(e: &BytesStart) -> (i64, i64) {
    let x = attr_val(e, b"x")
        .and_then(|v| v.parse::<i64>().ok())
        .unwrap_or(0);
    let y = attr_val(e, b"y")
        .and_then(|v| v.parse::<i64>().ok())
        .unwrap_or(0);
    (y, x)
}

/// Add the running group-translation stack to a shape's local offset.
fn effective_pos(sp: (i64, i64), stack: &[(i64, i64)]) -> (i64, i64) {
    let (ty, tx) = stack
        .iter()
        .fold((0i64, 0i64), |a, b| (a.0 + b.0, a.1 + b.1));
    (sp.0 + ty, sp.1 + tx)
}

/// Walks one `slide<N>.xml` and groups its content into ordered shapes.
///
/// Slide-level concerns (shape boundaries, placeholder type, position,
/// tables) live here; everything inside a `<txBody>` is delegated to
/// [`TextBodyParser`] which collects paragraphs that we drain at `</tc>`
/// (cell) or `</sp>` (shape body).
pub(super) fn parse_slide(
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
    // Did this shape carry any `<p:ph>`? Distinguishes a placeholder shape
    // (inherits master body style) from a free-form text box (master
    // other style).
    let mut sp_has_ph = false;

    // Picture-shape state. Mirrors the `in_sp` machinery but for `<p:pic>`
    // — we want a separate offset and a captured rel id pointing at
    // `ppt/media/imageN.*`.
    let mut in_pic = false;
    let mut pic_offset: (i64, i64) = (0, 0);
    let mut pic_rel_id: Option<String> = None;
    let mut pic_alt: Option<String> = None;

    // Group shape translation stack. Each `<p:grpSp>` introduces its own
    // child coordinate frame; the relative shift between the group's own
    // `<a:off>` (where it sits on the slide) and its `<a:chOff>` (the
    // origin children measure from) becomes a translation that applies
    // to every descendant. Nesting composes additively, which is correct
    // when groups don't scale (the common case). When they do scale, our
    // y/x are still in the right neighbourhood for sorting purposes.
    let mut grp_translation_stack: Vec<(i64, i64)> = Vec::new();
    let mut in_grp_sppr = false;
    let mut cur_grp_off: (i64, i64) = (0, 0);
    let mut cur_grp_choff: (i64, i64) = (0, 0);

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
                        sp_has_ph = false;
                    }
                    b"pic" => {
                        in_pic = true;
                        pic_offset = (0, 0);
                        pic_rel_id = None;
                        pic_alt = None;
                    }
                    b"cNvPr" => {
                        if in_sp {
                            sp_name = attr_val(&e, b"name");
                        } else if in_pic {
                            // Prefer alt-text (`descr`) over the shape's
                            // display name when both are present.
                            pic_alt = attr_val(&e, b"descr")
                                .or_else(|| attr_val(&e, b"name"));
                        }
                    }
                    b"blip" => {
                        if in_pic && pic_rel_id.is_none() {
                            pic_rel_id = attr_val(&e, b"embed");
                        }
                    }
                    b"ph" => {
                        sp_has_ph = true;
                        if let Some(t) = attr_val(&e, b"type") {
                            if t == "title" || t == "ctrTitle" {
                                sp_is_title = true;
                            }
                        }
                    }
                    b"off" => {
                        let (y, x) = read_xy(&e);
                        if in_grp_sppr {
                            cur_grp_off = (y, x);
                        } else if in_sp || in_tbl {
                            sp_offset = (y, x);
                        } else if in_pic {
                            pic_offset = (y, x);
                        }
                    }
                    b"chOff" => {
                        if in_grp_sppr {
                            cur_grp_choff = read_xy(&e);
                        }
                    }
                    b"grpSpPr" => {
                        in_grp_sppr = true;
                        cur_grp_off = (0, 0);
                        cur_grp_choff = (0, 0);
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
                        } else if in_pic {
                            pic_alt = attr_val(&e, b"descr")
                                .or_else(|| attr_val(&e, b"name"));
                        }
                    }
                    b"blip" => {
                        if in_pic && pic_rel_id.is_none() {
                            pic_rel_id = attr_val(&e, b"embed");
                        }
                    }
                    b"off" => {
                        let (y, x) = read_xy(&e);
                        if in_grp_sppr {
                            cur_grp_off = (y, x);
                        } else if in_sp || in_tbl {
                            sp_offset = (y, x);
                        } else if in_pic {
                            pic_offset = (y, x);
                        }
                    }
                    b"chOff" => {
                        if in_grp_sppr {
                            cur_grp_choff = read_xy(&e);
                        }
                    }
                    b"ph" => {
                        sp_has_ph = true;
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
                                render_paragraph_with_bullet(p, None)
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
                        let (y, x) = effective_pos(sp_offset, &grp_translation_stack);
                        shapes.push(Shape {
                            y,
                            x,
                            kind: ShapeKind::Table(std::mem::take(&mut tbl_rows)),
                        });
                    }
                    b"grpSpPr" => {
                        in_grp_sppr = false;
                        grp_translation_stack.push((
                            cur_grp_off.0 - cur_grp_choff.0,
                            cur_grp_off.1 - cur_grp_choff.1,
                        ));
                    }
                    b"grpSp" => {
                        // Each `<p:grpSp>` pushes exactly one translation
                        // when its grpSpPr closes. Pop the matching one.
                        grp_translation_stack.pop();
                    }
                    b"pic" => {
                        in_pic = false;
                        if let Some(rel_id) = pic_rel_id.take() {
                            let (y, x) =
                                effective_pos(pic_offset, &grp_translation_stack);
                            let alt = pic_alt.take().unwrap_or_default();
                            shapes.push(Shape {
                                y,
                                x,
                                kind: ShapeKind::Picture { rel_id, alt },
                            });
                        }
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
                        let had_ph = sp_has_ph;
                        sp_name = None;
                        sp_has_ph = false;
                        let (y, x) = effective_pos(sp_offset, &grp_translation_stack);
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
                                y,
                                x,
                                kind: ShapeKind::Title(text),
                            });
                        } else if !sp_paragraphs.is_empty() {
                            // Placeholder shapes inherit body styles; free
                            // text boxes inherit "other" (typically no
                            // bullets in default masters).
                            let category = if had_ph {
                                StyleCategory::Body
                            } else {
                                StyleCategory::Other
                            };
                            shapes.push(Shape {
                                y,
                                x,
                                kind: ShapeKind::Body {
                                    category,
                                    paras: sp_paragraphs,
                                },
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
