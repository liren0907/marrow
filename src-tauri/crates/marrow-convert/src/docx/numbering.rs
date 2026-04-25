use std::collections::HashMap;

use quick_xml::events::Event;
use quick_xml::reader::Reader;

use super::attrs::attr_val;
use super::types::{ListKind, NumberingMap};

/// numbering.xml: parse the two-step `numId → abstractNumId → list def`
/// chain with per-level `<w:numFmt>` resolution. Returns `(numId, ilvl)`
/// → `ListKind`, populated for every level the spec declares plus the
/// implicit fallback for missing levels (Bullet).
pub(super) fn parse_numbering(xml: &str) -> NumberingMap {
    let mut reader = Reader::from_str(xml);
    reader.config_mut().trim_text(false);
    let mut buf = Vec::new();

    // First pass: build (abstract_id, ilvl) → ListKind, plus numId →
    // abstract_id linkage.
    let mut abstract_levels: HashMap<(String, usize), ListKind> = HashMap::new();
    let mut num_to_abstract: HashMap<String, String> = HashMap::new();

    enum Scope {
        None,
        Abstract(String),
        Num(String),
    }
    let mut scope = Scope::None;
    let mut cur_lvl: Option<usize> = None;

    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(e)) | Ok(Event::Empty(e)) => {
                let name = e.local_name();
                match name.as_ref() {
                    b"abstractNum" => {
                        if let Some(id) = attr_val(&e, b"abstractNumId") {
                            scope = Scope::Abstract(id);
                        }
                    }
                    b"num" => {
                        if let Some(id) = attr_val(&e, b"numId") {
                            scope = Scope::Num(id);
                        }
                    }
                    b"abstractNumId" => {
                        if let Scope::Num(num_id) = &scope {
                            if let Some(v) = attr_val(&e, b"val") {
                                num_to_abstract.insert(num_id.clone(), v);
                            }
                        }
                    }
                    b"lvl" => {
                        if let Some(v) = attr_val(&e, b"ilvl") {
                            cur_lvl = v.parse().ok();
                        }
                    }
                    b"numFmt" => {
                        if let (Scope::Abstract(aid), Some(lvl), Some(v)) =
                            (&scope, cur_lvl, attr_val(&e, b"val"))
                        {
                            let kind = if v == "bullet" {
                                ListKind::Bullet
                            } else {
                                ListKind::Ordered
                            };
                            abstract_levels.insert((aid.clone(), lvl), kind);
                        }
                    }
                    _ => {}
                }
            }
            Ok(Event::End(e)) => {
                let name = e.local_name();
                match name.as_ref() {
                    b"abstractNum" | b"num" => scope = Scope::None,
                    b"lvl" => cur_lvl = None,
                    _ => {}
                }
            }
            Ok(Event::Eof) => break,
            Err(_) => break,
            _ => {}
        }
        buf.clear();
    }

    // Second pass: project abstract levels onto each numId.
    let mut out = NumberingMap::new();
    for (num_id, abstract_id) in &num_to_abstract {
        for ((aid, lvl), kind) in &abstract_levels {
            if aid == abstract_id {
                out.insert((num_id.clone(), *lvl), *kind);
            }
        }
    }
    out
}
