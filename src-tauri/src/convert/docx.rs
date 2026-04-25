use std::collections::{HashMap, HashSet};

use quick_xml::events::Event;
use quick_xml::reader::Reader;

use crate::convert::ConvertError;
use crate::convert::ooxml_util::{
    Zip, escape_md, open_zip, parse_rels, post_process, read_zip_bytes,
    read_zip_text, unique_asset_name, wrap_run_full,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ListKind {
    Bullet,
    Ordered,
}

/// Per-level format inside an abstract numbering definition. Indexed by
/// `(numId, ilvl)` after `numId → abstractNumId` resolution.
type NumberingMap = HashMap<(String, usize), ListKind>;

struct Ctx {
    rels: HashMap<String, String>,
    styles: HashMap<String, usize>, // style_id → heading level (1..=6)
    numbering: NumberingMap,
    /// rId → final attachment filename (e.g. `image1.png`). Populated up
    /// front so the walker can emit `![alt](attachments/...)` without
    /// touching the zip a second time.
    image_rels: HashMap<String, String>,
    /// id → "Author Name". Empty when there are no comments.
    comment_authors: HashMap<String, String>,
    /// id → flattened comment text (single line, paragraph breaks → spaces).
    comment_texts: HashMap<String, String>,
    /// id → footnote markdown body (already rendered, paragraph break → space).
    footnotes: HashMap<String, String>,
    /// id → endnote markdown body.
    endnotes: HashMap<String, String>,
}

#[derive(Default, Clone)]
struct Run {
    text: String,
    bold: bool,
    italic: bool,
    underline: bool,
    strike: bool,
}

#[derive(Default)]
struct Paragraph {
    style_id: Option<String>,
    num_id: Option<String>,
    ilvl: usize,
    runs: Vec<Run>,
    /// Comments whose range ends at (or covers) this paragraph. Rendered
    /// as `> 💬 **Author**: text` blockquotes after the paragraph body.
    comment_ids: Vec<String>,
}

/// Sidecar asset extracted from the docx (`word/media/*`).
pub struct DocxAsset {
    pub name: String,
    pub bytes: Vec<u8>,
}

pub struct DocxResult {
    pub markdown: String,
    pub assets: Vec<DocxAsset>,
}

pub fn docx_to_markdown(bytes: &[u8]) -> Result<DocxResult, ConvertError> {
    let mut zip = open_zip(bytes)?;
    let rels = read_zip_text(&mut zip, "word/_rels/document.xml.rels")?
        .map(|s| parse_rels(&s))
        .unwrap_or_default();
    let styles = read_zip_text(&mut zip, "word/styles.xml")?
        .map(|s| parse_styles(&s))
        .unwrap_or_default();
    let numbering = read_zip_text(&mut zip, "word/numbering.xml")?
        .map(|s| parse_numbering(&s))
        .unwrap_or_default();

    // Pictures: walk every Image relationship and pull the bytes once,
    // de-duping by zip path. The walker only needs `image_rels` to
    // resolve r:embed → final attachment filename.
    let (image_rels, assets) = collect_image_assets(&mut zip, &rels)?;

    // Comments: load author/text up front so the walker can refer by id.
    let (comment_authors, comment_texts) = load_comments(&mut zip)?;

    // Footnotes / endnotes: pre-render bodies keyed by id. Inline runs
    // inside the body lose formatting (we flatten to plain markdown text).
    let footnotes = load_notes(&mut zip, "word/footnotes.xml", "footnote")?;
    let endnotes = load_notes(&mut zip, "word/endnotes.xml", "endnote")?;

    let body = read_zip_text(&mut zip, "word/document.xml")?
        .ok_or_else(|| ConvertError::Zip("missing word/document.xml".into()))?;

    let ctx = Ctx {
        rels,
        styles,
        numbering,
        image_rels,
        comment_authors,
        comment_texts,
        footnotes,
        endnotes,
    };
    let mut md = walk_document(&body, &ctx)?;

    // Headers / footers: emit at the very top / bottom of the document
    // wrapped in HTML comments so they don't pollute the main heading
    // outline. Each part is rendered as plain paragraphs separated by
    // newlines.
    let (header_md, footer_md) = render_headers_footers(&mut zip)?;
    if !header_md.is_empty() {
        md = format!("<!-- header -->\n{header_md}\n<!-- /header -->\n\n{md}");
    }
    if !footer_md.is_empty() {
        md.push_str(&format!("\n<!-- footer -->\n{footer_md}\n<!-- /footer -->\n"));
    }

    // Footnote / endnote definitions tail.
    let mut tail = String::new();
    let used_fn = footnote_ids_in(&md, "fn");
    let used_en = footnote_ids_in(&md, "en");
    for id in &used_fn {
        if let Some(body) = ctx.footnotes.get(id) {
            tail.push_str(&format!("[^fn{id}]: {body}\n"));
        }
    }
    for id in &used_en {
        if let Some(body) = ctx.endnotes.get(id) {
            tail.push_str(&format!("[^en{id}]: {body}\n"));
        }
    }
    if !tail.is_empty() {
        md.push_str("\n");
        md.push_str(&tail);
    }

    Ok(DocxResult {
        markdown: post_process(md),
        assets,
    })
}

/// styles.xml: find `<w:style w:styleId="X">` with inner `<w:name w:val="heading N">`
/// or `<w:name w:val="Heading N">` → X maps to N.
fn parse_styles(xml: &str) -> HashMap<String, usize> {
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
        map.entry(format!("Heading{n}"))
            .or_insert(n);
        map.entry(format!("heading {n}"))
            .or_insert(n);
    }
    map
}

fn parse_heading_level(name: &str) -> Option<usize> {
    let lower = name.to_ascii_lowercase();
    let rest = lower.strip_prefix("heading ").or_else(|| lower.strip_prefix("heading"))?;
    let n: usize = rest.trim().parse().ok()?;
    if (1..=6).contains(&n) { Some(n) } else { None }
}

/// numbering.xml: parse the two-step `numId → abstractNumId → list def`
/// chain with per-level `<w:numFmt>` resolution. Returns `(numId, ilvl)`
/// → `ListKind`, populated for every level the spec declares plus the
/// implicit fallback for missing levels (Bullet).
fn parse_numbering(xml: &str) -> NumberingMap {
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

fn attr_val(e: &quick_xml::events::BytesStart, key: &[u8]) -> Option<String> {
    for attr in e.attributes().flatten() {
        if attr.key.local_name().as_ref() == key {
            return attr.unescape_value().ok().map(|c| c.into_owned());
        }
    }
    None
}

/// Walk every `Type` ending with `/image` rel and collect its bytes from
/// `word/media/`. Builds `rId → final filename` so the document walker
/// only needs the rels map to emit attachments.
fn collect_image_assets(
    zip: &mut Zip,
    rels: &HashMap<String, String>,
) -> Result<(HashMap<String, String>, Vec<DocxAsset>), ConvertError> {
    let mut image_rels = HashMap::new();
    let mut assets = Vec::new();
    let mut used: HashSet<String> = HashSet::new();
    let mut path_to_name: HashMap<String, String> = HashMap::new();

    for (rid, target) in rels {
        // Only treat targets under `media/` as picture parts. (Other rel
        // kinds — fontTable, theme, settings — share the rels file and
        // would otherwise leak in.)
        if !target.starts_with("media/") && !target.starts_with("../media/") {
            continue;
        }
        // Targets are relative to `word/_rels/document.xml.rels`, which
        // lives in `word/`, so `media/foo.png` resolves to
        // `word/media/foo.png`.
        let normalized = target.trim_start_matches("../");
        let zip_path = format!("word/{normalized}");

        let final_name = if let Some(existing) = path_to_name.get(&zip_path) {
            existing.clone()
        } else {
            let bytes = match read_zip_bytes(zip, &zip_path)? {
                Some(b) => b,
                None => continue,
            };
            let basename = zip_path
                .rsplit('/')
                .next()
                .unwrap_or(&zip_path)
                .to_string();
            let n = unique_asset_name(&basename, &used);
            used.insert(n.clone());
            path_to_name.insert(zip_path.clone(), n.clone());
            assets.push(DocxAsset { name: n.clone(), bytes });
            n
        };
        image_rels.insert(rid.clone(), final_name);
    }
    Ok((image_rels, assets))
}

/// Load `word/comments.xml` if present. Returns `(authors, texts)`,
/// each keyed by comment id. Text is flattened: paragraphs collapse to
/// space-joined runs, formatting is dropped.
fn load_comments(
    zip: &mut Zip,
) -> Result<(HashMap<String, String>, HashMap<String, String>), ConvertError> {
    let mut authors = HashMap::new();
    let mut texts = HashMap::new();
    let xml = match read_zip_text(zip, "word/comments.xml")? {
        Some(s) => s,
        None => return Ok((authors, texts)),
    };
    let mut reader = Reader::from_str(&xml);
    reader.config_mut().trim_text(false);
    let mut buf = Vec::new();
    let mut cur_id: Option<String> = None;
    let mut cur_text = String::new();
    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(e)) if e.local_name().as_ref() == b"comment" => {
                cur_id = attr_val(&e, b"id");
                if let Some(author) = attr_val(&e, b"author") {
                    if let Some(id) = cur_id.as_deref() {
                        authors.insert(id.to_string(), author);
                    }
                }
                cur_text.clear();
            }
            Ok(Event::End(e)) if e.local_name().as_ref() == b"comment" => {
                if let Some(id) = cur_id.take() {
                    texts.insert(id, cur_text.trim().to_string());
                }
                cur_text.clear();
            }
            Ok(Event::Text(t)) => {
                if cur_id.is_some() {
                    if let Ok(raw) = t.decode() {
                        if !cur_text.is_empty() && !cur_text.ends_with(' ') {
                            cur_text.push(' ');
                        }
                        cur_text.push_str(raw.trim());
                    }
                }
            }
            Ok(Event::Eof) => break,
            Err(_) => break,
            _ => {}
        }
        buf.clear();
    }
    Ok((authors, texts))
}

/// Load footnotes / endnotes from `word/footnotes.xml` or
/// `word/endnotes.xml`. `wrapper` is the local-name of the per-note
/// element (`footnote` or `endnote`). Returns id → flattened body. Skips
/// the implicit separator/continuation-separator notes (id 0 / -1) which
/// have `w:type="separator"` and carry no real content.
fn load_notes(
    zip: &mut Zip,
    path: &str,
    wrapper: &str,
) -> Result<HashMap<String, String>, ConvertError> {
    let mut out = HashMap::new();
    let xml = match read_zip_text(zip, path)? {
        Some(s) => s,
        None => return Ok(out),
    };
    let mut reader = Reader::from_str(&xml);
    reader.config_mut().trim_text(false);
    let mut buf = Vec::new();
    let wrapper_b = wrapper.as_bytes();
    let mut cur_id: Option<String> = None;
    let mut skip = false;
    let mut cur_text = String::new();
    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(e)) if e.local_name().as_ref() == wrapper_b => {
                cur_id = attr_val(&e, b"id");
                let ty = attr_val(&e, b"type");
                skip = matches!(
                    ty.as_deref(),
                    Some("separator") | Some("continuationSeparator")
                );
                cur_text.clear();
            }
            Ok(Event::End(e)) if e.local_name().as_ref() == wrapper_b => {
                if let Some(id) = cur_id.take() {
                    if !skip {
                        out.insert(id, cur_text.trim().to_string());
                    }
                }
                cur_text.clear();
                skip = false;
            }
            Ok(Event::Text(t)) => {
                if cur_id.is_some() && !skip {
                    if let Ok(raw) = t.decode() {
                        let trimmed = raw.trim();
                        if !trimmed.is_empty() {
                            if !cur_text.is_empty() && !cur_text.ends_with(' ') {
                                cur_text.push(' ');
                            }
                            cur_text.push_str(trimmed);
                        }
                    }
                }
            }
            Ok(Event::Eof) => break,
            Err(_) => break,
            _ => {}
        }
        buf.clear();
    }
    Ok(out)
}

/// Render every header*.xml + footer*.xml in the zip. Order is alpha by
/// filename (ZIP-order in practice), which matches the section order. Text
/// runs are flattened (no formatting).
fn render_headers_footers(zip: &mut Zip) -> Result<(String, String), ConvertError> {
    let names: Vec<String> = zip
        .file_names()
        .filter(|n| {
            n.starts_with("word/header") || n.starts_with("word/footer")
        })
        .filter(|n| n.ends_with(".xml"))
        .map(|s| s.to_string())
        .collect();
    let mut headers = String::new();
    let mut footers = String::new();
    for name in names {
        let xml = match read_zip_text(zip, &name)? {
            Some(s) => s,
            None => continue,
        };
        let body = flatten_paragraphs(&xml);
        if body.trim().is_empty() {
            continue;
        }
        if name.contains("header") {
            if !headers.is_empty() {
                headers.push('\n');
            }
            headers.push_str(&body);
        } else {
            if !footers.is_empty() {
                footers.push('\n');
            }
            footers.push_str(&body);
        }
    }
    Ok((headers, footers))
}

/// Strip-down walker that emits one line of text per `<w:p>`. Used for
/// headers / footers where we don't care about styles, lists, or rels.
fn flatten_paragraphs(xml: &str) -> String {
    let mut reader = Reader::from_str(xml);
    reader.config_mut().trim_text(false);
    let mut buf = Vec::new();
    let mut out = String::new();
    let mut cur = String::new();
    let mut in_run = false;
    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(e)) if e.local_name().as_ref() == b"r" => {
                in_run = true;
            }
            Ok(Event::End(e)) if e.local_name().as_ref() == b"r" => {
                in_run = false;
            }
            Ok(Event::Text(t)) if in_run => {
                if let Ok(raw) = t.decode() {
                    cur.push_str(&raw);
                }
            }
            Ok(Event::End(e)) if e.local_name().as_ref() == b"p" => {
                let line = cur.trim();
                if !line.is_empty() {
                    out.push_str(line);
                    out.push('\n');
                }
                cur.clear();
            }
            Ok(Event::Eof) => break,
            Err(_) => break,
            _ => {}
        }
        buf.clear();
    }
    out
}

/// Scan rendered markdown for `[^fn123]` / `[^en123]` references and
/// return the unique referenced ids in first-occurrence order. Pads the
/// definitions tail to only what was actually cited.
fn footnote_ids_in(md: &str, prefix: &str) -> Vec<String> {
    let needle = format!("[^{prefix}");
    let mut out: Vec<String> = Vec::new();
    let mut seen: HashSet<String> = HashSet::new();
    let bytes = md.as_bytes();
    let mut i = 0;
    while let Some(pos) = md[i..].find(&needle) {
        let start = i + pos + needle.len();
        let mut end = start;
        while end < bytes.len() && bytes[end].is_ascii_digit() {
            end += 1;
        }
        if end > start && bytes.get(end) == Some(&b']') {
            let id = &md[start..end];
            if seen.insert(id.to_string()) {
                out.push(id.to_string());
            }
        }
        i = end;
    }
    out
}

/// Main walk over word/document.xml. Emits Markdown in order.
fn walk_document(xml: &str, ctx: &Ctx) -> Result<String, ConvertError> {
    let mut reader = Reader::from_str(xml);
    reader.config_mut().trim_text(false);
    let mut buf = Vec::new();
    let mut out = String::new();

    // Paragraph state
    let mut cur_p: Option<Paragraph> = None;
    let mut in_run = false;
    let mut cur_run = Run::default();
    // When true, text inside <w:r> goes into a hyperlink buffer instead of cur_p
    let mut in_hyperlink: Option<String> = None;
    let mut hyperlink_buf: Vec<Run> = Vec::new();
    let mut in_rpr = false;

    // Field-code hyperlink state machine. Word emits old-style hyperlinks
    // as: <w:fldChar type="begin"> ... <w:instrText>HYPERLINK "url"</w:instrText>
    // ... <w:fldChar type="separate"> [display runs] <w:fldChar type="end">.
    let mut field_state = FieldState::None;
    let mut field_instr = String::new();
    let mut field_display: Vec<Run> = Vec::new();

    // Comment range tracking. When <w:commentRangeStart w:id="N"> opens
    // we mark the id as live; <w:commentRangeEnd w:id="N"> attaches the
    // comment to the paragraph that contains the End element.
    let mut active_comments: HashSet<String> = HashSet::new();

    // Table state (one level; nested tables are flattened to text)
    let mut table_stack: Vec<TableAcc> = Vec::new();
    let mut cur_cell_md: Option<String> = None;

    // Pictures: when <w:drawing> opens we look for the first
    // <a:blip r:embed="rIdN"> inside it and emit the image as a synthetic
    // run. `drawing_alt` carries any wp:docPr descr/title we encounter.
    let mut in_drawing = false;
    let mut drawing_alt: Option<String> = None;

    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(e)) => {
                let name = e.local_name().as_ref().to_vec();
                match name.as_slice() {
                    b"p" => {
                        cur_p = Some(Paragraph::default());
                    }
                    b"pStyle" => {
                        if let Some(p) = cur_p.as_mut() {
                            p.style_id = attr_val(&e, b"val");
                        }
                    }
                    b"numId" => {
                        if let Some(p) = cur_p.as_mut() {
                            p.num_id = attr_val(&e, b"val");
                        }
                    }
                    b"ilvl" => {
                        if let Some(p) = cur_p.as_mut() {
                            if let Some(v) = attr_val(&e, b"val") {
                                p.ilvl = v.parse().unwrap_or(0);
                            }
                        }
                    }
                    b"r" => {
                        in_run = true;
                        cur_run = Run::default();
                    }
                    b"rPr" => in_rpr = true,
                    b"hyperlink" => {
                        let rid = attr_val(&e, b"id");
                        in_hyperlink = rid.and_then(|id| ctx.rels.get(&id).cloned());
                        hyperlink_buf.clear();
                    }
                    b"tbl" => table_stack.push(TableAcc::default()),
                    b"tr" => {
                        if let Some(t) = table_stack.last_mut() {
                            t.rows.push(Vec::new());
                        }
                    }
                    b"tc" => {
                        cur_cell_md = Some(String::new());
                    }
                    b"drawing" => {
                        in_drawing = true;
                        drawing_alt = None;
                    }
                    b"instrText" => {
                        // Text events inside this element are treated as
                        // field-code body (HYPERLINK "...") rather than
                        // visible run text.
                        if matches!(field_state, FieldState::Begin) {
                            field_instr.clear();
                        }
                    }
                    _ => {}
                }
            }
            Ok(Event::Empty(e)) => {
                let name = e.local_name().as_ref().to_vec();
                match name.as_slice() {
                    b"b" if in_rpr => cur_run.bold = true,
                    b"i" if in_rpr => cur_run.italic = true,
                    b"u" if in_rpr => {
                        // <w:u w:val="single"|"double"|...> → underline.
                        // <w:u w:val="none"> explicitly disables. Treat
                        // any non-none / missing-val as on.
                        let val = attr_val(&e, b"val");
                        if val.as_deref() != Some("none") {
                            cur_run.underline = true;
                        }
                    }
                    b"strike" if in_rpr => cur_run.strike = true,
                    b"dstrike" if in_rpr => cur_run.strike = true,
                    b"pStyle" => {
                        if let Some(p) = cur_p.as_mut() {
                            p.style_id = attr_val(&e, b"val");
                        }
                    }
                    b"numId" => {
                        if let Some(p) = cur_p.as_mut() {
                            p.num_id = attr_val(&e, b"val");
                        }
                    }
                    b"ilvl" => {
                        if let Some(p) = cur_p.as_mut() {
                            if let Some(v) = attr_val(&e, b"val") {
                                p.ilvl = v.parse().unwrap_or(0);
                            }
                        }
                    }
                    b"tab" => {
                        if in_run {
                            cur_run.text.push(' ');
                        }
                    }
                    b"br" => {
                        // <w:br w:type="page"> → markdown thematic break,
                        // emitted between paragraphs. Plain breaks become
                        // a hard newline in the current run.
                        let ty = attr_val(&e, b"type");
                        if ty.as_deref() == Some("page") {
                            // Break the current paragraph, flush, emit ---,
                            // then continue. We can only do this cleanly
                            // when we're inside a paragraph.
                            if let Some(mut p) = cur_p.take() {
                                if in_run && !cur_run.text.is_empty() {
                                    p.runs.push(std::mem::take(&mut cur_run));
                                }
                                let line = render_paragraph(&p, ctx);
                                if let Some(cell) = cur_cell_md.as_mut() {
                                    let inline = line.replace('\n', " ").trim().to_string();
                                    if !cell.is_empty() && !inline.is_empty() {
                                        cell.push(' ');
                                    }
                                    cell.push_str(&inline);
                                } else {
                                    out.push_str(&line);
                                    out.push_str("\n\n---\n\n");
                                }
                                cur_p = Some(Paragraph::default());
                            }
                        } else if in_run {
                            cur_run.text.push('\n');
                        }
                    }
                    b"drawing" => {
                        in_drawing = false;
                        drawing_alt = None;
                    }
                    b"blip" if in_drawing => {
                        // r:embed → final image filename via image_rels.
                        let rid = attr_val_with_prefix(&e, b"r", b"embed")
                            .or_else(|| attr_val(&e, b"embed"));
                        if let (Some(rid), Some(p)) = (rid, cur_p.as_mut()) {
                            if let Some(name) = ctx.image_rels.get(&rid) {
                                let alt = drawing_alt
                                    .clone()
                                    .unwrap_or_default()
                                    .replace('[', "")
                                    .replace(']', "");
                                p.runs.push(Run {
                                    text: format!(
                                        "![{}](attachments/{})",
                                        alt, name
                                    ),
                                    ..Default::default()
                                });
                            }
                        }
                    }
                    b"docPr" if in_drawing => {
                        let descr = attr_val(&e, b"descr");
                        let title = attr_val(&e, b"title");
                        let nm = attr_val(&e, b"name");
                        drawing_alt = descr
                            .or(title)
                            .or(nm)
                            .filter(|s| !s.is_empty());
                    }
                    b"commentRangeStart" => {
                        if let Some(id) = attr_val(&e, b"id") {
                            active_comments.insert(id);
                        }
                    }
                    b"commentRangeEnd" => {
                        if let (Some(id), Some(p)) =
                            (attr_val(&e, b"id"), cur_p.as_mut())
                        {
                            if active_comments.remove(&id) {
                                p.comment_ids.push(id);
                            }
                        }
                    }
                    b"commentReference" => {
                        // Comment with no inline range — attach to the
                        // current paragraph anyway so the comment text
                        // still appears somewhere.
                        if let (Some(id), Some(p)) =
                            (attr_val(&e, b"id"), cur_p.as_mut())
                        {
                            if !p.comment_ids.iter().any(|x| x == &id) {
                                p.comment_ids.push(id);
                            }
                        }
                    }
                    b"footnoteReference" => {
                        if let (Some(id), Some(_)) = (attr_val(&e, b"id"), cur_p.as_mut())
                        {
                            cur_run.text.push_str(&format!("[^fn{id}]"));
                        }
                    }
                    b"endnoteReference" => {
                        if let (Some(id), Some(_)) = (attr_val(&e, b"id"), cur_p.as_mut())
                        {
                            cur_run.text.push_str(&format!("[^en{id}]"));
                        }
                    }
                    b"fldChar" => {
                        match attr_val(&e, b"fldCharType").as_deref() {
                            Some("begin") => {
                                field_state = FieldState::Begin;
                                field_instr.clear();
                                field_display.clear();
                            }
                            Some("separate") => {
                                field_state = FieldState::Separate;
                            }
                            Some("end") => {
                                if let Some(url) = parse_hyperlink_instr(&field_instr) {
                                    let merged: String = field_display
                                        .iter()
                                        .map(|r| r.text.as_str())
                                        .collect::<Vec<_>>()
                                        .join("");
                                    let text = escape_md(&merged);
                                    if let Some(p) = cur_p.as_mut() {
                                        p.runs.push(Run {
                                            text: format!("[{text}]({url})"),
                                            ..Default::default()
                                        });
                                    }
                                } else {
                                    // Not a HYPERLINK field — fall back to
                                    // the display runs as plain text.
                                    if let Some(p) = cur_p.as_mut() {
                                        for r in field_display.drain(..) {
                                            p.runs.push(r);
                                        }
                                    }
                                }
                                field_state = FieldState::None;
                                field_instr.clear();
                                field_display.clear();
                            }
                            _ => {}
                        }
                    }
                    _ => {}
                }
            }
            Ok(Event::Text(t)) => {
                let raw = match t.decode() {
                    Ok(r) => quick_xml::escape::unescape(&r)
                        .map(|c| c.into_owned())
                        .unwrap_or_else(|_| r.into_owned()),
                    Err(_) => continue,
                };
                if matches!(field_state, FieldState::Begin) {
                    // Inside <w:instrText> — accumulate field code body.
                    // (Plain <w:t> text inside the begin block is rare;
                    // we capture both.)
                    field_instr.push_str(&raw);
                } else if in_run {
                    cur_run.text.push_str(&raw);
                }
            }
            Ok(Event::End(e)) => {
                let name = e.local_name().as_ref().to_vec();
                match name.as_slice() {
                    b"rPr" => in_rpr = false,
                    b"r" => {
                        in_run = false;
                        if matches!(field_state, FieldState::Separate) {
                            // Display run for an in-flight HYPERLINK field.
                            if !cur_run.text.is_empty() {
                                field_display.push(std::mem::take(&mut cur_run));
                            } else {
                                cur_run = Run::default();
                            }
                        } else if !cur_run.text.is_empty() {
                            if in_hyperlink.is_some() {
                                hyperlink_buf.push(std::mem::take(&mut cur_run));
                            } else if let Some(p) = cur_p.as_mut() {
                                p.runs.push(std::mem::take(&mut cur_run));
                            }
                        } else {
                            cur_run = Run::default();
                        }
                    }
                    b"hyperlink" => {
                        if let Some(url) = in_hyperlink.take() {
                            let merged: String = hyperlink_buf
                                .iter()
                                .map(|r| r.text.as_str())
                                .collect::<Vec<_>>()
                                .join("");
                            let text = escape_md(&merged);
                            if let Some(p) = cur_p.as_mut() {
                                p.runs.push(Run {
                                    text: format!("[{text}]({url})"),
                                    ..Default::default()
                                });
                            }
                            hyperlink_buf.clear();
                        }
                    }
                    b"p" => {
                        if let Some(p) = cur_p.take() {
                            let line = render_paragraph(&p, ctx);
                            let comment_block = render_comments_for(&p, ctx);
                            if let Some(cell) = cur_cell_md.as_mut() {
                                let inline = line.replace('\n', " ").trim().to_string();
                                if !cell.is_empty() && !inline.is_empty() {
                                    cell.push(' ');
                                }
                                cell.push_str(&inline);
                                // Comments inside cells: append inline so
                                // the row stays valid markdown table syntax.
                                if !comment_block.is_empty() {
                                    cell.push(' ');
                                    cell.push_str(&comment_block.replace('\n', " "));
                                }
                            } else {
                                out.push_str(&line);
                                out.push('\n');
                                if !comment_block.is_empty() {
                                    out.push_str(&comment_block);
                                    out.push('\n');
                                }
                            }
                        }
                    }
                    b"tc" => {
                        if let Some(cell) = cur_cell_md.take() {
                            if let Some(t) = table_stack.last_mut() {
                                if let Some(row) = t.rows.last_mut() {
                                    row.push(cell);
                                }
                            }
                        }
                    }
                    b"tbl" => {
                        if let Some(tbl) = table_stack.pop() {
                            let md = render_table(&tbl);
                            if let Some(cell) = cur_cell_md.as_mut() {
                                let flat = tbl
                                    .rows
                                    .iter()
                                    .flat_map(|r| r.iter())
                                    .cloned()
                                    .collect::<Vec<_>>()
                                    .join(" ");
                                if !cell.is_empty() {
                                    cell.push(' ');
                                }
                                cell.push_str(&flat);
                                let _ = md;
                            } else {
                                out.push('\n');
                                out.push_str(&md);
                                out.push('\n');
                            }
                        }
                    }
                    b"drawing" => {
                        in_drawing = false;
                        drawing_alt = None;
                    }
                    _ => {}
                }
            }
            Ok(Event::Eof) => break,
            Err(e) => return Err(ConvertError::Xml(format!("{e}"))),
            _ => {}
        }
        buf.clear();
    }
    Ok(out)
}

#[derive(Default)]
struct TableAcc {
    rows: Vec<Vec<String>>,
}

enum FieldState {
    None,
    /// Between `fldCharType="begin"` and `"separate"` — collecting
    /// instrText.
    Begin,
    /// Between `"separate"` and `"end"` — collecting display runs.
    Separate,
}

fn render_table(t: &TableAcc) -> String {
    if t.rows.is_empty() {
        return String::new();
    }
    let cols = t.rows.iter().map(|r| r.len()).max().unwrap_or(0);
    if cols == 0 {
        return String::new();
    }
    let mut out = String::new();
    let header = normalize_row(&t.rows[0], cols);
    out.push_str(&format_row(&header));
    out.push('\n');
    out.push_str(&format_row(&vec!["---".to_string(); cols]));
    out.push('\n');
    for row in t.rows.iter().skip(1) {
        out.push_str(&format_row(&normalize_row(row, cols)));
        out.push('\n');
    }
    out
}

fn normalize_row(row: &[String], cols: usize) -> Vec<String> {
    let mut v: Vec<String> = row
        .iter()
        .map(|c| c.replace('|', "\\|").replace('\n', " ").trim().to_string())
        .collect();
    while v.len() < cols {
        v.push(String::new());
    }
    v
}

fn format_row(cells: &[String]) -> String {
    let mut s = String::from("|");
    for c in cells {
        s.push(' ');
        s.push_str(c);
        s.push_str(" |");
    }
    s
}

fn render_paragraph(p: &Paragraph, ctx: &Ctx) -> String {
    // Merge adjacent runs with identical formatting (cheap dedup before
    // we pay for markdown markers).
    let merged = merge_adjacent_runs(&p.runs);
    let mut body = String::new();
    for r in &merged {
        let escaped = escape_md_preserve_brackets(&r.text);
        body.push_str(&wrap_run_full(
            &escaped,
            r.bold,
            r.italic,
            r.underline,
            r.strike,
        ));
    }
    let body = body.trim().to_string();

    // Heading?
    if let Some(id) = &p.style_id {
        if let Some(level) = ctx.styles.get(id).copied() {
            let hash = "#".repeat(level);
            return if body.is_empty() {
                String::new()
            } else {
                format!("\n{hash} {body}\n")
            };
        }
    }

    // List?
    if let Some(num_id) = &p.num_id {
        let kind = ctx
            .numbering
            .get(&(num_id.clone(), p.ilvl))
            .copied()
            .unwrap_or(ListKind::Bullet);
        let marker = match kind {
            ListKind::Bullet => "-".to_string(),
            ListKind::Ordered => "1.".to_string(),
        };
        let indent = "  ".repeat(p.ilvl);
        return format!("{indent}{marker} {body}");
    }

    if body.is_empty() {
        String::new()
    } else {
        format!("{body}\n")
    }
}

/// Render the comment blockquotes attached to `p`. Returns "" when there
/// are no comments. Each comment becomes a `> 💬 **Author**: text` line.
fn render_comments_for(p: &Paragraph, ctx: &Ctx) -> String {
    if p.comment_ids.is_empty() {
        return String::new();
    }
    let mut out = String::new();
    for id in &p.comment_ids {
        let author = ctx
            .comment_authors
            .get(id)
            .cloned()
            .unwrap_or_else(|| "Unknown".to_string());
        let text = ctx.comment_texts.get(id).cloned().unwrap_or_default();
        if text.is_empty() {
            continue;
        }
        out.push_str(&format!("> 💬 **{author}**: {text}\n"));
    }
    out
}

fn attr_val_with_prefix(
    e: &quick_xml::events::BytesStart,
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

/// Extract the URL from a HYPERLINK field-code body. Body looks like:
/// `HYPERLINK "https://example.com" \o "tooltip"` (or single-quoted, or
/// no quotes at all). Returns the URL portion when the body starts with
/// HYPERLINK; `None` otherwise (so non-link fields fall back to display
/// runs).
fn parse_hyperlink_instr(instr: &str) -> Option<String> {
    let trimmed = instr.trim();
    let rest = trimmed.strip_prefix("HYPERLINK")?.trim_start();
    // Quoted URL.
    if let Some(rest) = rest.strip_prefix('"') {
        let end = rest.find('"')?;
        return Some(rest[..end].to_string());
    }
    // Unquoted URL up to first whitespace.
    let end = rest.find(char::is_whitespace).unwrap_or(rest.len());
    if end == 0 {
        None
    } else {
        Some(rest[..end].to_string())
    }
}

/// Same as escape_md in ooxml_util but hyperlink runs arrive pre-formatted
/// with `[..](..)` — don't double-escape brackets if the text is clearly
/// already a link form. Heuristic: skip escaping when text matches `[*](*)`.
fn escape_md_preserve_brackets(text: &str) -> String {
    if looks_like_link(text) || looks_like_image(text) || looks_like_footnote(text) {
        return text.to_string();
    }
    escape_md(text)
}

fn looks_like_link(text: &str) -> bool {
    if !text.starts_with('[') {
        return false;
    }
    text.contains("](") && text.ends_with(')')
}

fn looks_like_image(text: &str) -> bool {
    text.starts_with("![") && text.contains("](") && text.ends_with(')')
}

fn looks_like_footnote(text: &str) -> bool {
    // [^fn3] or [^en7]
    text.starts_with("[^") && text.ends_with(']')
}

fn merge_adjacent_runs(runs: &[Run]) -> Vec<Run> {
    let mut out: Vec<Run> = Vec::with_capacity(runs.len());
    for r in runs {
        if let Some(last) = out.last_mut() {
            if last.bold == r.bold
                && last.italic == r.italic
                && last.underline == r.underline
                && last.strike == r.strike
            {
                last.text.push_str(&r.text);
                continue;
            }
        }
        out.push(r.clone());
    }
    out
}

