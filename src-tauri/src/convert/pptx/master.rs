use std::collections::HashMap;

use quick_xml::events::Event;
use quick_xml::reader::Reader;

use crate::convert::ooxml_dml::BulletKind;

use super::types::{MasterStyles, StyleCategory};

/// Walk a `slideMasterN.xml` and harvest default bullet styling per
/// `(category, level)` from the master's `<p:txStyles>` block.
///
/// The master defines three style cohorts:
/// - `<p:titleStyle>` — applied to the title placeholder
/// - `<p:bodyStyle>` — applied to body / outline placeholders
/// - `<p:otherStyle>` — applied to free-form text boxes
///
/// Each cohort holds nine optional `<a:lvlNpPr>` blocks (one per indent
/// level, 1-indexed in XML, 0-indexed in our [`crate::convert::ooxml_dml::Paragraph::ilvl`]).
/// Each level may carry a `<a:buChar>`, `<a:buAutoNum>`, or `<a:buNone>` —
/// that's what we capture. Other DML inside `<a:lvlNpPr>` (sizing,
/// indentation, fonts) is ignored.
pub(super) fn parse_master_styles(xml: &str) -> MasterStyles {
    let mut reader = Reader::from_str(xml);
    reader.config_mut().trim_text(false);
    let mut buf = Vec::new();
    let mut map: MasterStyles = HashMap::new();

    let mut category: Option<StyleCategory> = None;
    let mut level: Option<usize> = None;

    let assign = |map: &mut MasterStyles,
                  category: Option<StyleCategory>,
                  level: Option<usize>,
                  kind: BulletKind| {
        if let (Some(c), Some(l)) = (category, level) {
            map.insert((c, l), kind);
        }
    };

    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(e)) => {
                let n = e.local_name().as_ref().to_vec();
                match n.as_slice() {
                    b"titleStyle" => category = Some(StyleCategory::Title),
                    b"bodyStyle" => category = Some(StyleCategory::Body),
                    b"otherStyle" => category = Some(StyleCategory::Other),
                    name if category.is_some() => {
                        if let Some(l) = parse_lvl_ppr(name) {
                            level = Some(l);
                        } else {
                            match name {
                                b"buChar" => assign(&mut map, category, level, BulletKind::Char),
                                b"buAutoNum" => {
                                    assign(&mut map, category, level, BulletKind::AutoNum)
                                }
                                b"buNone" => assign(&mut map, category, level, BulletKind::None),
                                _ => {}
                            }
                        }
                    }
                    _ => {}
                }
            }
            Ok(Event::Empty(e)) => {
                let n = e.local_name().as_ref().to_vec();
                if category.is_some() {
                    match n.as_slice() {
                        b"buChar" => assign(&mut map, category, level, BulletKind::Char),
                        b"buAutoNum" => assign(&mut map, category, level, BulletKind::AutoNum),
                        b"buNone" => assign(&mut map, category, level, BulletKind::None),
                        _ => {}
                    }
                }
            }
            Ok(Event::End(e)) => {
                let n = e.local_name().as_ref().to_vec();
                match n.as_slice() {
                    b"titleStyle" | b"bodyStyle" | b"otherStyle" => {
                        category = None;
                        level = None;
                    }
                    name if parse_lvl_ppr(name).is_some() => level = None,
                    _ => {}
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

/// Map element names like `lvl1pPr` … `lvl9pPr` to the 0-indexed level
/// used by [`crate::convert::ooxml_dml::Paragraph::ilvl`]. Returns `None`
/// for everything else.
fn parse_lvl_ppr(name: &[u8]) -> Option<usize> {
    let s = std::str::from_utf8(name).ok()?;
    let n: usize = s.strip_prefix("lvl")?.strip_suffix("pPr")?.parse().ok()?;
    if n == 0 { None } else { Some(n - 1) }
}
