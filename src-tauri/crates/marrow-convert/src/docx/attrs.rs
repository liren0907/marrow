use quick_xml::events::BytesStart;

pub(super) fn attr_val(e: &BytesStart, key: &[u8]) -> Option<String> {
    for attr in e.attributes().flatten() {
        if attr.key.local_name().as_ref() == key {
            return attr.unescape_value().ok().map(|c| c.into_owned());
        }
    }
    None
}

pub(super) fn attr_val_with_prefix(
    e: &BytesStart,
    prefix: &[u8],
    local: &[u8],
) -> Option<String> {
    for attr in e.attributes().flatten() {
        if attr.key.local_name().as_ref() == local
            && attr.key.prefix().map(|p| p.as_ref() == prefix).unwrap_or(false)
        {
            return attr.unescape_value().ok().map(|c| c.into_owned());
        }
    }
    None
}
