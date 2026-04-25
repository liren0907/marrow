use quick_xml::events::Event;
use quick_xml::reader::Reader;

use super::attrs::attr_val;

pub(super) struct RawComment {
    pub(super) author_id: String,
    pub(super) text: String,
}

/// Parse a legacy `commentN.xml` (`<p:cmLst>` of `<p:cm>` entries with
/// inline `<p:text>` payloads). Returns one [`RawComment`] per entry.
pub(super) fn parse_legacy_comments(xml: &str) -> Vec<RawComment> {
    let mut reader = Reader::from_str(xml);
    reader.config_mut().trim_text(false);
    let mut buf = Vec::new();
    let mut out = Vec::new();
    let mut cur_author: Option<String> = None;
    let mut in_text = false;
    let mut cur_text = String::new();
    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(e)) => match e.local_name().as_ref() {
                b"cm" => {
                    cur_author = attr_val(&e, b"authorId");
                    cur_text.clear();
                }
                b"text" => in_text = true,
                _ => {}
            },
            Ok(Event::Text(t)) => {
                if in_text {
                    if let Ok(raw) = t.decode() {
                        let un = quick_xml::escape::unescape(&raw)
                            .map(|c| c.into_owned())
                            .unwrap_or_else(|_| raw.into_owned());
                        cur_text.push_str(&un);
                    }
                }
            }
            Ok(Event::End(e)) => match e.local_name().as_ref() {
                b"text" => in_text = false,
                b"cm" => {
                    if let Some(author_id) = cur_author.take() {
                        let text = cur_text.trim().to_string();
                        if !text.is_empty() {
                            out.push(RawComment {
                                author_id,
                                text: std::mem::take(&mut cur_text),
                            });
                        } else {
                            cur_text.clear();
                        }
                    }
                }
                _ => {}
            },
            Ok(Event::Eof) => break,
            Err(_) => break,
            _ => {}
        }
        buf.clear();
    }
    out
}

/// Parse one modern threaded comment file (one comment per file). The
/// shape of interest is `<p188:cm authorId=…><p188:txBody>…<a:t>…</a:t>…
/// </p188:txBody></p188:cm>`. Returns `None` if we can't recover both an
/// author id and any text.
pub(super) fn parse_modern_comment(xml: &str) -> Option<RawComment> {
    let mut reader = Reader::from_str(xml);
    reader.config_mut().trim_text(false);
    let mut buf = Vec::new();
    let mut author_id: Option<String> = None;
    let mut in_t = false;
    let mut text = String::new();
    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(e)) => match e.local_name().as_ref() {
                b"cm" => {
                    if author_id.is_none() {
                        author_id = attr_val(&e, b"authorId");
                    }
                }
                b"t" => in_t = true,
                _ => {}
            },
            Ok(Event::Text(t)) => {
                if in_t {
                    if let Ok(raw) = t.decode() {
                        let un = quick_xml::escape::unescape(&raw)
                            .map(|c| c.into_owned())
                            .unwrap_or_else(|_| raw.into_owned());
                        if !text.is_empty() {
                            text.push(' ');
                        }
                        text.push_str(&un);
                    }
                }
            }
            Ok(Event::End(e)) => {
                if e.local_name().as_ref() == b"t" {
                    in_t = false;
                }
            }
            Ok(Event::Eof) => break,
            Err(_) => break,
            _ => {}
        }
        buf.clear();
    }
    let author_id = author_id?;
    let text = text.trim().to_string();
    if text.is_empty() {
        return None;
    }
    Some(RawComment { author_id, text })
}
