use quick_xml::events::BytesStart;

pub(super) fn attr_val(e: &BytesStart, key: &[u8]) -> Option<String> {
    for attr in e.attributes().flatten() {
        if attr.key.local_name().as_ref() == key {
            return attr.unescape_value().ok().map(|c| c.into_owned());
        }
    }
    None
}
