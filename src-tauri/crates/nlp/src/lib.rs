//! L0 → L1 extraction for Marrow's multi-layer notes.
//!
//! A pre-LLM pipeline: strip Markdown syntax to plain text
//! ([`markdown_to_plain`]), run `jieba-rs` tokenization + POS tagging, keep
//! the noun-ish tokens, and aggregate them by surface form
//! ([`extract_nouns`]). Pure functions — no I/O, no global state — so the
//! Tauri command layer can call them inside `spawn_blocking` and they stay
//! trivially unit-testable.
//!
//! Public surface:
//! - [`Annotation`] — one extracted term: surface form + POS + frequency + spans
//! - [`extract_nouns`] — markdown → ranked annotations (by frequency)
//! - [`markdown_to_plain`] — markdown → plain text (exposed for testing/reuse)
//!
//! Span note: `Annotation::spans` are `(start, end)` **character** offsets into
//! the *extracted plain text* (NOT the original Markdown source). They are
//! collected now for forward-compatibility (a future editor overlay), but the
//! L1 view only uses `text` / `count` / `pos` today.
//!
//! Dictionary note: `jieba-rs` ships a Simplified-Chinese dictionary. Traditional
//! Chinese still segments (HMM fallback) but with lower precision; loading a
//! Traditional dictionary is a deliberate follow-up, not part of this first slice.

use std::collections::HashMap;
use std::io::BufReader;
use std::sync::OnceLock;

use jieba_rs::Jieba;
use pulldown_cmark::{Event, Parser};
use serde::Serialize;

/// Traditional+Simplified dictionary with POS tags (`詞 詞頻 詞性`), bundled at
/// compile time. Source: fxsjy/jieba `extra_dict/dict.txt.big` (MIT) — see
/// `data/DICT_LICENSE`. ~8 MB embedded in the binary.
static TW_DICT: &str = include_str!("../data/dict.txt.big");

/// Shared, lazily-built `Jieba` with the Traditional dictionary merged on top of
/// the bundled Simplified default. Built once and reused — re-parsing an 8 MB
/// dictionary on every `extract_nouns` call would be far too slow.
fn shared_jieba() -> &'static Jieba {
    static JIEBA: OnceLock<Jieba> = OnceLock::new();
    JIEBA.get_or_init(|| {
        let mut jieba = Jieba::new(); // bundled Simplified default
        let mut reader = BufReader::new(TW_DICT.as_bytes());
        jieba
            .load_dict(&mut reader)
            .expect("load bundled dict.txt.big");
        jieba
    })
}

/// One extracted L1 term.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Annotation {
    /// Surface form of the noun / noun-phrase as segmented.
    pub text: String,
    /// POS tag from jieba (`n`, `nr`, `ns`, `nt`, `nz`, …; or the raw tag for
    /// English-fallback terms).
    pub pos: String,
    /// Number of occurrences in the document.
    pub count: u32,
    /// `(start, end)` character offsets into the extracted plain text. One
    /// entry per occurrence. See the module-level span note.
    pub spans: Vec<(usize, usize)>,
}

/// Noun-ish "content term" filter. Keeps:
/// - any tag starting with `n` — all nominal (n / nr / nrt / ns / nt / nz / ng …)
/// - `l` (习用语) — multi-character domain terms jieba classifies as idiomatic
///   phrases rather than plain nouns, e.g. 相對論 / 量子力學. These are exactly
///   the high-value concept terms we want, so we include them deliberately.
/// Grammatical idioms (`i` 成语) and other parts of speech are excluded.
fn is_term(tag: &str) -> bool {
    tag.starts_with('n') || tag == "l"
}

/// Common English function words to drop from the ASCII fallback so English
/// notes surface content words rather than glue words.
const EN_STOPWORDS: &[&str] = &[
    "the", "and", "for", "with", "this", "that", "are", "was", "were", "has",
    "have", "had", "but", "not", "you", "your", "they", "them", "from", "its",
    "into", "out", "can", "will", "would", "could", "should", "than", "then",
    "there", "here", "what", "when", "where", "which", "who", "how", "all",
    "any", "some", "such", "only", "also", "more", "most", "other", "about",
];

/// ASCII fallback: jieba's bundled dictionary is Chinese, so English words get
/// tagged opaquely. Keep alphabetic ASCII tokens of length ≥ 3 that aren't
/// stopwords, so English documents aren't empty in the L1 view.
fn is_english_term(word: &str) -> bool {
    word.len() >= 3
        && word.chars().all(|c| c.is_ascii_alphabetic())
        && !EN_STOPWORDS.contains(&word.to_ascii_lowercase().as_str())
}

/// Strip Markdown syntax down to plain text. Text and inline-code runs are
/// kept; soft/hard breaks and block ends become newlines so adjacent blocks
/// don't fuse into spurious tokens.
pub fn markdown_to_plain(markdown: &str) -> String {
    let mut plain = String::with_capacity(markdown.len());
    for ev in Parser::new(markdown) {
        match ev {
            Event::Text(t) | Event::Code(t) => plain.push_str(&t),
            Event::SoftBreak | Event::HardBreak => plain.push('\n'),
            Event::End(_) => plain.push('\n'),
            _ => {}
        }
    }
    plain
}

/// Extract noun-ish terms from Markdown, aggregated by surface form and ranked
/// by descending frequency (ties broken by surface form for determinism).
pub fn extract_nouns(markdown: &str) -> Vec<Annotation> {
    let plain = markdown_to_plain(markdown);
    let jieba = shared_jieba();
    // hmm = true → recover out-of-vocabulary words (names, neologisms).
    let tagged = jieba.tag(&plain, true);

    // jieba segments the whole input contiguously, so we can recover each
    // token's char span by accumulating char counts in order — no re-search.
    let mut cursor = 0usize;
    let mut by_text: HashMap<String, Annotation> = HashMap::new();

    for token in tagged {
        let len = token.word.chars().count();
        let start = cursor;
        cursor += len;

        if !(is_term(token.tag) || is_english_term(token.word)) {
            continue;
        }

        let entry = by_text
            .entry(token.word.to_string())
            .or_insert_with(|| Annotation {
                text: token.word.to_string(),
                pos: token.tag.to_string(),
                count: 0,
                spans: Vec::new(),
            });
        entry.count += 1;
        entry.spans.push((start, start + len));
    }

    let mut out: Vec<Annotation> = by_text.into_values().collect();
    out.sort_by(|a, b| b.count.cmp(&a.count).then_with(|| a.text.cmp(&b.text)));
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn strips_markdown_syntax() {
        let plain = markdown_to_plain("# Title\n\n**bold** and `code`.");
        assert!(plain.contains("Title"));
        assert!(plain.contains("bold"));
        assert!(plain.contains("code"));
        assert!(!plain.contains('#'));
        assert!(!plain.contains('*'));
    }

    #[test]
    fn extracts_chinese_nouns_with_counts() {
        // Simplified Chinese — "苹果" is an unambiguous dictionary noun, so this
        // test is deterministic and exercises aggregation + span collection.
        let md = "# 标题\n\n我喜欢苹果，苹果非常好吃。";
        let anns = extract_nouns(md);
        let texts: Vec<&str> = anns.iter().map(|a| a.text.as_str()).collect();
        assert!(texts.contains(&"苹果"), "got: {texts:?}");

        let apple = anns.iter().find(|a| a.text == "苹果").unwrap();
        assert_eq!(apple.count, 2, "苹果 should occur twice");
        assert_eq!(apple.spans.len(), 2);
    }

    #[test]
    fn extracts_traditional_chinese_nouns() {
        // The whole reason for bundling dict.txt.big. Before it, jieba's
        // Simplified default mangled these (愛因斯坦 → 因斯, 物理學 → 物理,
        // 相對論 dropped). 相對論/量子力學 are tagged `l`, hence the broadened
        // is_term filter.
        let md = "# 筆記\n\n愛因斯坦提出相對論。相對論很重要。物理學與量子力學。";
        let anns = extract_nouns(md);
        let t: Vec<&str> = anns.iter().map(|a| a.text.as_str()).collect();
        assert!(t.contains(&"愛因斯坦"), "got: {t:?}");
        assert!(t.contains(&"相對論"), "got: {t:?}");
        assert!(t.contains(&"物理學"), "got: {t:?}");
        assert!(t.contains(&"量子力學"), "got: {t:?}");

        let rel = anns.iter().find(|a| a.text == "相對論").unwrap();
        assert_eq!(rel.count, 2, "相對論 should occur twice");
    }

    #[test]
    fn english_fallback_is_nonempty_and_drops_stopwords() {
        let anns = extract_nouns("The relativity theory revolutionized modern physics.");
        let texts: Vec<&str> = anns.iter().map(|a| a.text.as_str()).collect();
        assert!(!anns.is_empty(), "english doc should not be empty");
        assert!(!texts.contains(&"the"), "stopwords should be dropped");
    }

    #[test]
    fn ranked_by_descending_frequency() {
        let anns = extract_nouns("苹果 苹果 苹果 香蕉 香蕉 橙子");
        // First entry must have the highest count.
        assert!(anns.windows(2).all(|w| w[0].count >= w[1].count));
    }
}
