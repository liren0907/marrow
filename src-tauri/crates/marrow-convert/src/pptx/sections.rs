use std::collections::HashMap;

use quick_xml::events::{BytesStart, Event};
use quick_xml::reader::Reader;

use crate::ConvertError;
use crate::ooxml::util::{Zip, parse_rels, read_zip_text, resolve_rel_path};

use super::attrs::attr_val;

/// Build a `slide_path → section_name` map by walking
/// `ppt/presentation.xml` and `ppt/_rels/presentation.xml.rels`. Only
/// the slide that opens each section gets an entry — that's where we
/// emit the `# Section` heading. Returns an empty map for decks with no
/// `<p:sectionLst>` (the common case).
pub(super) fn build_section_map(
    zip: &mut Zip<'_>,
) -> Result<HashMap<String, String>, ConvertError> {
    let pres_xml = match read_zip_text(zip, "ppt/presentation.xml")? {
        Some(s) => s,
        None => return Ok(HashMap::new()),
    };
    let pres_rels = read_zip_text(zip, "ppt/_rels/presentation.xml.rels")?
        .as_deref()
        .map(parse_rels)
        .unwrap_or_default();

    // sld_id_to_path: "256" → "ppt/slides/slide1.xml"
    let mut sld_id_to_path: HashMap<String, String> = HashMap::new();
    // sections: list of (name, first_sld_id) in document order.
    let mut sections: Vec<(String, String)> = Vec::new();

    let mut reader = Reader::from_str(&pres_xml);
    reader.config_mut().trim_text(false);
    let mut buf = Vec::new();

    let mut in_section_lst = false;
    let mut cur_section_name: Option<String> = None;
    let mut cur_section_first: Option<String> = None;

    let handle_sld_id = |e: &BytesStart,
                         in_section_lst: bool,
                         cur_section_first: &mut Option<String>,
                         sld_id_to_path: &mut HashMap<String, String>| {
        if in_section_lst {
            // Inside `<p14:sldIdLst>`: just ids, first one wins.
            if cur_section_first.is_none() {
                if let Some(id) = attr_val_unprefixed(e, b"id") {
                    *cur_section_first = Some(id);
                }
            }
        } else {
            // Top-level `<p:sldIdLst>`: id + r:id, resolve r:id via rels.
            let id = attr_val_unprefixed(e, b"id");
            let rid = attr_val_prefixed(e, b"r", b"id");
            if let (Some(id), Some(rid)) = (id, rid) {
                if let Some(target) = pres_rels.get(&rid) {
                    let path = resolve_rel_path("ppt/presentation.xml", target);
                    sld_id_to_path.insert(id, path);
                }
            }
        }
    };

    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(e)) => {
                let n = e.local_name().as_ref().to_vec();
                match n.as_slice() {
                    b"sectionLst" => in_section_lst = true,
                    b"section" => {
                        cur_section_name = attr_val(&e, b"name");
                        cur_section_first = None;
                    }
                    b"sldId" => handle_sld_id(
                        &e,
                        in_section_lst,
                        &mut cur_section_first,
                        &mut sld_id_to_path,
                    ),
                    _ => {}
                }
            }
            Ok(Event::Empty(e)) => {
                if e.local_name().as_ref() == b"sldId" {
                    handle_sld_id(
                        &e,
                        in_section_lst,
                        &mut cur_section_first,
                        &mut sld_id_to_path,
                    );
                }
            }
            Ok(Event::End(e)) => {
                let n = e.local_name().as_ref().to_vec();
                match n.as_slice() {
                    b"sectionLst" => in_section_lst = false,
                    b"section" => {
                        if let (Some(name), Some(first)) =
                            (cur_section_name.take(), cur_section_first.take())
                        {
                            sections.push((name, first));
                        }
                    }
                    _ => {}
                }
            }
            Ok(Event::Eof) => break,
            Err(_) => break,
            _ => {}
        }
        buf.clear();
    }

    let mut out = HashMap::new();
    for (name, first_id) in sections {
        if let Some(path) = sld_id_to_path.get(&first_id) {
            out.insert(path.clone(), name);
        }
    }
    Ok(out)
}

/// Match `local` only when the attribute has no namespace prefix.
fn attr_val_unprefixed(e: &BytesStart, local: &[u8]) -> Option<String> {
    for attr in e.attributes().flatten() {
        if attr.key.local_name().as_ref() == local && attr.key.prefix().is_none() {
            return attr.unescape_value().ok().map(|c| c.into_owned());
        }
    }
    None
}

/// Match `local` only when the attribute carries the given namespace
/// prefix (e.g. `r:id`).
fn attr_val_prefixed(e: &BytesStart, prefix: &[u8], local: &[u8]) -> Option<String> {
    for attr in e.attributes().flatten() {
        if attr.key.local_name().as_ref() == local
            && attr.key.prefix().map(|p| p.as_ref() == prefix).unwrap_or(false)
        {
            return attr.unescape_value().ok().map(|c| c.into_owned());
        }
    }
    None
}
