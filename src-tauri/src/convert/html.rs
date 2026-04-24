use crate::convert::ConvertError;
use dom_smoothie::{Config, Readability};
use htmd::HtmlToMarkdown;

const READABILITY_MIN_CHARS: usize = 200;

pub fn html_to_markdown(bytes: &[u8]) -> Result<String, ConvertError> {
    let html = decode_bytes(bytes);
    let main = extract_main(&html);
    let md = to_markdown(&main)?;
    Ok(post_process(md))
}

fn decode_bytes(bytes: &[u8]) -> String {
    use chardetng::{EncodingDetector, Iso2022JpDetection, Utf8Detection};
    let mut det = EncodingDetector::new(Iso2022JpDetection::Allow);
    det.feed(bytes, true);
    let encoding = det.guess(None, Utf8Detection::Allow);
    let (cow, _had_errors) = encoding.decode_with_bom_removal(bytes);
    cow.into_owned()
}

fn extract_main(html: &str) -> String {
    // Readability-style main-content extraction. Falls back to the full
    // document when readability fails or produces output that is too short
    // to be credible (doc-site simple pages, etc.).
    let cfg = Config::default();
    let Ok(mut readability) = Readability::new(html, None, Some(cfg)) else {
        return html.to_string();
    };
    match readability.parse() {
        Ok(article) => {
            let content: &str = &article.content;
            if content.trim().chars().count() >= READABILITY_MIN_CHARS {
                content.to_string()
            } else {
                html.to_string()
            }
        }
        Err(_) => html.to_string(),
    }
}

fn to_markdown(html: &str) -> Result<String, ConvertError> {
    HtmlToMarkdown::builder()
        .skip_tags(vec!["script", "style", "noscript"])
        .build()
        .convert(html)
        .map_err(|e| ConvertError::Serialize(format!("{e}")))
}

fn post_process(md: String) -> String {
    // Collapse 3+ consecutive newlines into exactly 2.
    let mut out = String::with_capacity(md.len());
    let mut run = 0usize;
    for c in md.chars() {
        if c == '\n' {
            run += 1;
            if run <= 2 {
                out.push(c);
            }
        } else {
            run = 0;
            out.push(c);
        }
    }
    let mut trimmed = out.trim().to_string();
    trimmed.push('\n');
    trimmed
}
