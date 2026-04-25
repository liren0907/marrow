use std::collections::HashMap;

use quick_xml::events::Event;
use quick_xml::reader::Reader;

use super::attrs::attr_val;

/// styles.xml: find `<w:style w:styleId="X">` with inner `<w:name w:val="heading N">`
/// or `<w:name w:val="Heading N">` → X maps to N.
pub(super) fn parse_styles(xml: &str) -> HashMap<String, usize> {
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
        map.entry(format!("Heading{n}")).or_insert(n);
        map.entry(format!("heading {n}")).or_insert(n);
    }
    map
}

fn parse_heading_level(name: &str) -> Option<usize> {
    let lower = name.to_ascii_lowercase();
    let rest = lower.strip_prefix("heading ").or_else(|| lower.strip_prefix("heading"))?;
    let n: usize = rest.trim().parse().ok()?;
    if (1..=6).contains(&n) { Some(n) } else { None }
}
