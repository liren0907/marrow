use std::collections::HashMap;

use quick_xml::events::Event;
use quick_xml::reader::Reader;

use super::attrs::attr_val;

/// Parse an authors document (`commentAuthors.xml` or `authors.xml`) into
/// `id → display_name`. Pass the local element name we expect on each
/// author entry (`b"cmAuthor"` for legacy, `b"author"` for modern).
pub(super) fn parse_authors(xml: &str, element_local: &[u8]) -> HashMap<String, String> {
    let mut reader = Reader::from_str(xml);
    reader.config_mut().trim_text(false);
    let mut buf = Vec::new();
    let mut out = HashMap::new();
    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Empty(e)) | Ok(Event::Start(e))
                if e.local_name().as_ref() == element_local =>
            {
                if let (Some(id), Some(name)) =
                    (attr_val(&e, b"id"), attr_val(&e, b"name"))
                {
                    out.insert(id, name);
                }
            }
            Ok(Event::Eof) => break,
            Err(_) => break,
            _ => {}
        }
        buf.clear();
    }
    out
}
