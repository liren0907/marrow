use std::collections::{HashMap, HashSet};

use quick_xml::events::Event;
use quick_xml::reader::Reader;

use crate::convert::ConvertError;
use crate::convert::ooxml_dml::{
    BulletKind, Paragraph, TextBodyParser, extract_paragraphs,
    render_paragraph_with_bullet,
};
use crate::convert::ooxml_util::{
    find_rel_target, find_rel_targets, list_zip_names, open_zip, parse_rels,
    post_process, read_zip_bytes, read_zip_text, resolve_rel_path,
};

/// Which `<p:txStyles>` block in the slide master a shape's text inherits
/// its default bullet styling from.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
enum StyleCategory {
    Title,
    Body,
    Other,
}

type MasterStyles = HashMap<(StyleCategory, usize), BulletKind>;

enum ShapeKind {
    /// Title or centerTitle placeholder — emit as heading.
    Title(String),
    /// Body text: ordered list of paragraphs. `category` selects which
    /// master `<p:txStyles>` block we fall back to for bullet defaults.
    Body {
        category: StyleCategory,
        paras: Vec<Paragraph>,
    },
    /// Table: pre-rendered cell strings.
    Table(Vec<Vec<String>>),
    /// Picture: `rel_id` references a `ppt/media/imageN.*` entry via the
    /// slide's rels. Resolved to a sidecar asset by [`pptx_to_markdown`].
    Picture { rel_id: String, alt: String },
}

struct Shape {
    y: i64,
    x: i64,
    kind: ShapeKind,
}

struct Slide {
    /// `<p:sld show="0">` — slide marked hidden in the deck.
    hidden: bool,
    shapes: Vec<Shape>,
}

/// One sidecar asset extracted from the deck (currently embedded
/// pictures only). Bytes are raw — base64 encoding happens at the IPC
/// boundary in `commands/convert.rs`.
pub struct PptxAsset {
    pub name: String,
    pub bytes: Vec<u8>,
}

pub struct PptxResult {
    pub markdown: String,
    pub assets: Vec<PptxAsset>,
}

pub fn pptx_to_markdown(bytes: &[u8]) -> Result<PptxResult, ConvertError> {
    let mut zip = open_zip(bytes)?;

    // Build a merged bullet-style table from every slide master in the
    // deck. Most decks have one master; for multi-master decks we just
    // overlay them (last-write-wins per category+level), which is good
    // enough since same-category styles are typically consistent.
    let master_paths: Vec<String> = list_zip_names(&zip, "ppt/slideMasters/slideMaster")
        .into_iter()
        .filter(|n| n.ends_with(".xml") && !n.contains("_rels"))
        .collect();
    let mut master_styles: MasterStyles = HashMap::new();
    for m in &master_paths {
        if let Some(xml) = read_zip_text(&mut zip, m)? {
            for (k, v) in parse_master_styles(&xml) {
                master_styles.insert(k, v);
            }
        }
    }

    // Section headings, if the deck declares any. Map full slide path →
    // section name so we know where to inject `# Section` separators.
    let section_for_slide = build_section_map(&mut zip)?;

    // Comment authors live at presentation scope: legacy
    // (`commentAuthors.xml`, integer ids) and modern (`authors.xml`, GUID
    // ids). Both kinds share the same `id → name` map since the id
    // namespaces don't overlap.
    let mut authors: HashMap<String, String> = HashMap::new();
    if let Some(xml) = read_zip_text(&mut zip, "ppt/commentAuthors.xml")? {
        for (k, v) in parse_authors(&xml, b"cmAuthor") {
            authors.insert(k, v);
        }
    }
    if let Some(xml) = read_zip_text(&mut zip, "ppt/authors.xml")? {
        for (k, v) in parse_authors(&xml, b"author") {
            authors.insert(k, v);
        }
    }

    // Collect slide paths by numeric ordinal.
    let mut slide_paths: Vec<String> = list_zip_names(&zip, "ppt/slides/slide")
        .into_iter()
        .filter(|n| n.ends_with(".xml") && !n.contains("_rels"))
        .collect();
    slide_paths.sort_by_key(|p| slide_ordinal(p).unwrap_or(i64::MAX));

    let mut out = String::new();
    let mut assets: Vec<PptxAsset> = Vec::new();
    // De-dup by zip path: the same media file may be embedded in multiple
    // slides; we only want one copy in attachments/. Maps zip path →
    // final asset filename.
    let mut asset_path_to_name: HashMap<String, String> = HashMap::new();
    let mut asset_names_used: HashSet<String> = HashSet::new();
    for (idx, path) in slide_paths.iter().enumerate() {
        let slide_xml = match read_zip_text(&mut zip, path)? {
            Some(s) => s,
            None => continue,
        };
        let rels_path = path
            .rsplit_once('/')
            .map(|(dir, file)| format!("{dir}/_rels/{file}.rels"))
            .unwrap_or_default();
        let rels_xml = read_zip_text(&mut zip, &rels_path)?;
        let rels = rels_xml
            .as_deref()
            .map(parse_rels)
            .unwrap_or_default();
        let notes_target = rels_xml
            .as_deref()
            .and_then(|x| find_rel_target(x, "notesSlide"));

        let mut slide = parse_slide(&slide_xml, &rels)?;
        if let Some(section) = section_for_slide.get(path) {
            out.push_str(&format!("# {}\n\n", section.trim()));
        }
        if slide.hidden {
            // Preserve numbering so readers can map output back to the
            // source deck, but omit body content.
            out.push_str(&format!("<!-- Slide {} (hidden) -->\n\n", idx + 1));
            continue;
        }
        slide.shapes.sort_by_key(|s| (s.y, s.x));

        out.push_str(&format!("<!-- Slide {} -->\n", idx + 1));
        let mut emitted_any = false;
        for shape in &slide.shapes {
            match &shape.kind {
                ShapeKind::Title(t) => {
                    let trimmed = t.trim();
                    if !trimmed.is_empty() {
                        out.push_str(&format!("## {trimmed}\n\n"));
                        emitted_any = true;
                    }
                }
                ShapeKind::Body { category, paras } => {
                    for p in paras {
                        let inherited = master_styles.get(&(*category, p.ilvl));
                        let line = render_paragraph_with_bullet(p, inherited);
                        if !line.trim().is_empty() {
                            out.push_str(&line);
                            out.push('\n');
                            emitted_any = true;
                        }
                    }
                    if !paras.is_empty() {
                        out.push('\n');
                    }
                }
                ShapeKind::Table(rows) => {
                    let rendered = render_table(rows);
                    if !rendered.is_empty() {
                        out.push_str(&rendered);
                        out.push('\n');
                        emitted_any = true;
                    }
                }
                ShapeKind::Picture { rel_id, alt } => {
                    let target = match rels.get(rel_id) {
                        Some(t) => t.clone(),
                        None => continue,
                    };
                    let zip_path = resolve_rel_path(path, &target);
                    let final_name = match asset_path_to_name.get(&zip_path) {
                        Some(n) => n.clone(),
                        None => {
                            let bytes = match read_zip_bytes(&mut zip, &zip_path)? {
                                Some(b) => b,
                                None => continue,
                            };
                            let basename = zip_path
                                .rsplit('/')
                                .next()
                                .unwrap_or(&zip_path)
                                .to_string();
                            let n = unique_asset_name(&basename, &asset_names_used);
                            asset_names_used.insert(n.clone());
                            asset_path_to_name.insert(zip_path.clone(), n.clone());
                            assets.push(PptxAsset { name: n.clone(), bytes });
                            n
                        }
                    };
                    let alt_safe = alt.replace('[', "").replace(']', "");
                    out.push_str(&format!(
                        "![{}](attachments/{})\n",
                        alt_safe, final_name
                    ));
                    emitted_any = true;
                }
            }
        }
        // SmartArt diagrams live in sibling `ppt/diagrams/dataN.xml`
        // parts referenced from the slide rels. We render them as a
        // bullet list (we don't try to reproduce the visual layout).
        if let Some(rels_xml_str) = rels_xml.as_deref() {
            for diagram_target in find_rel_targets(rels_xml_str, "diagramData") {
                let p = resolve_rel_path(path, &diagram_target);
                if let Some(xml) = read_zip_text(&mut zip, &p)? {
                    let paragraphs = extract_paragraphs(&xml, &HashMap::new())?;
                    let lines: Vec<String> = paragraphs
                        .iter()
                        .map(|para| {
                            // Force a bullet so the SmartArt structure
                            // reads as a list even though source XML
                            // rarely declares one.
                            render_paragraph_with_bullet(para, Some(&BulletKind::Char))
                        })
                        .filter(|s| !s.trim().is_empty())
                        .collect();
                    if !lines.is_empty() {
                        out.push_str("\n");
                        for line in &lines {
                            out.push_str(line);
                            out.push('\n');
                        }
                        emitted_any = true;
                    }
                }
            }

            // Charts: emit a placeholder per chart rel so readers know
            // a visual was here without us trying to reproduce it.
            for _chart_target in find_rel_targets(rels_xml_str, "/chart") {
                if !emitted_any {
                    out.push('\n');
                }
                out.push_str("[Chart]\n");
                emitted_any = true;
            }
        }

        if let Some(target) = notes_target.as_deref() {
            let notes_path = resolve_rel_path(path, target);
            if let Some(rendered) =
                render_speaker_notes(&mut zip, &notes_path, &master_styles)?
            {
                if !emitted_any {
                    // Make sure the notes block has a leading blank line even
                    // when the slide body was empty.
                    out.push('\n');
                }
                out.push_str(&rendered);
                out.push('\n');
                emitted_any = true;
            }
        }

        // Comments: legacy `<p:cmLst>` plus modern threaded `<p188:cm>`.
        // Both kinds may be referenced from the slide rels.
        if let Some(rels_xml_str) = rels_xml.as_deref() {
            let mut comments: Vec<RawComment> = Vec::new();
            for legacy_target in find_rel_targets(rels_xml_str, "/comments") {
                let p = resolve_rel_path(path, &legacy_target);
                if let Some(xml) = read_zip_text(&mut zip, &p)? {
                    comments.extend(parse_legacy_comments(&xml));
                }
            }
            for modern_target in find_rel_targets(rels_xml_str, "threadedComment") {
                let p = resolve_rel_path(path, &modern_target);
                if let Some(xml) = read_zip_text(&mut zip, &p)? {
                    if let Some(c) = parse_modern_comment(&xml) {
                        comments.push(c);
                    }
                }
            }
            if !comments.is_empty() {
                if !emitted_any {
                    out.push('\n');
                }
                for c in &comments {
                    let author = authors
                        .get(&c.author_id)
                        .cloned()
                        .unwrap_or_else(|| "Unknown".to_string());
                    out.push_str(&format!(
                        "> 💬 **{}**: {}\n",
                        author,
                        c.text.replace('\n', " ").trim()
                    ));
                }
                emitted_any = true;
            }
        }

        if !emitted_any {
            out.push('\n');
        }
    }

    Ok(PptxResult {
        markdown: post_process(out),
        assets,
    })
}

/// Build a `slide_path → section_name` map by walking
/// `ppt/presentation.xml` and `ppt/_rels/presentation.xml.rels`. Only
/// the slide that opens each section gets an entry — that's where we
/// emit the `# Section` heading. Returns an empty map for decks with no
/// `<p:sectionLst>` (the common case).
fn build_section_map(zip: &mut crate::convert::ooxml_util::Zip<'_>) -> Result<HashMap<String, String>, ConvertError> {
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

    let handle_sld_id = |e: &quick_xml::events::BytesStart,
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
fn attr_val_unprefixed(e: &quick_xml::events::BytesStart, local: &[u8]) -> Option<String> {
    for attr in e.attributes().flatten() {
        if attr.key.local_name().as_ref() == local && attr.key.prefix().is_none() {
            return attr.unescape_value().ok().map(|c| c.into_owned());
        }
    }
    None
}

/// Match `local` only when the attribute carries the given namespace
/// prefix (e.g. `r:id`).
fn attr_val_prefixed(
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

/// Pick a non-conflicting filename for an asset. If `base` is unused, it
/// is returned verbatim; otherwise we append `-2`, `-3`, … before the
/// extension until a free name is found.
fn unique_asset_name(base: &str, used: &HashSet<String>) -> String {
    if !used.contains(base) {
        return base.to_string();
    }
    let (stem, ext) = match base.rfind('.') {
        Some(i) => (&base[..i], &base[i..]),
        None => (base, ""),
    };
    let mut n = 2usize;
    loop {
        let candidate = format!("{stem}-{n}{ext}");
        if !used.contains(&candidate) {
            return candidate;
        }
        n += 1;
    }
}

/// Read `notes_path` from `zip` and render its paragraphs as a Markdown
/// blockquote. Returns `Ok(None)` when the notes part is missing or
/// contains no rendered text.
fn render_speaker_notes(
    zip: &mut crate::convert::ooxml_util::Zip<'_>,
    notes_path: &str,
    master_styles: &MasterStyles,
) -> Result<Option<String>, ConvertError> {
    let xml = match read_zip_text(zip, notes_path)? {
        Some(s) => s,
        None => return Ok(None),
    };
    // Notes can carry their own hyperlinks; the rels file is optional.
    let notes_rels_path = notes_path
        .rsplit_once('/')
        .map(|(dir, file)| format!("{dir}/_rels/{file}.rels"))
        .unwrap_or_default();
    let rels = read_zip_text(zip, &notes_rels_path)?
        .as_deref()
        .map(parse_rels)
        .unwrap_or_default();

    let paragraphs = extract_paragraphs(&xml, &rels)?;
    let body: Vec<String> = paragraphs
        .iter()
        .map(|p| {
            let inherited = master_styles.get(&(StyleCategory::Body, p.ilvl));
            render_paragraph_with_bullet(p, inherited)
        })
        .filter(|s| !s.trim().is_empty())
        .collect();
    if body.is_empty() {
        return Ok(None);
    }

    let mut block = String::from("> **Notes:**\n>\n");
    for (i, p) in body.iter().enumerate() {
        if i > 0 {
            block.push_str(">\n");
        }
        for line in p.lines() {
            if line.is_empty() {
                block.push_str(">\n");
            } else {
                block.push_str("> ");
                block.push_str(line);
                block.push('\n');
            }
        }
    }
    Ok(Some(block))
}

fn slide_ordinal(path: &str) -> Option<i64> {
    let name = path.rsplit('/').next()?;
    let stem = name.strip_prefix("slide")?.strip_suffix(".xml")?;
    stem.parse().ok()
}

fn attr_val(e: &quick_xml::events::BytesStart, key: &[u8]) -> Option<String> {
    for attr in e.attributes().flatten() {
        if attr.key.local_name().as_ref() == key {
            return attr.unescape_value().ok().map(|c| c.into_owned());
        }
    }
    None
}

/// Pull the `(y, x)` integer pair off an `<a:off>` / `<a:chOff>` element.
/// Missing or unparseable attributes default to 0.
fn read_xy(e: &quick_xml::events::BytesStart) -> (i64, i64) {
    let x = attr_val(e, b"x")
        .and_then(|v| v.parse::<i64>().ok())
        .unwrap_or(0);
    let y = attr_val(e, b"y")
        .and_then(|v| v.parse::<i64>().ok())
        .unwrap_or(0);
    (y, x)
}

/// Add the running group-translation stack to a shape's local offset.
fn effective_pos(sp: (i64, i64), stack: &[(i64, i64)]) -> (i64, i64) {
    let (ty, tx) = stack
        .iter()
        .fold((0i64, 0i64), |a, b| (a.0 + b.0, a.1 + b.1));
    (sp.0 + ty, sp.1 + tx)
}

/// Walks one `slide<N>.xml` and groups its content into ordered shapes.
///
/// Slide-level concerns (shape boundaries, placeholder type, position,
/// tables) live here; everything inside a `<txBody>` is delegated to
/// [`TextBodyParser`] which collects paragraphs that we drain at `</tc>`
/// (cell) or `</sp>` (shape body).
fn parse_slide(
    xml: &str,
    rels: &HashMap<String, String>,
) -> Result<Slide, ConvertError> {
    let mut reader = Reader::from_str(xml);
    reader.config_mut().trim_text(false);
    let mut buf = Vec::new();

    let mut shapes: Vec<Shape> = Vec::new();
    let mut hidden = false;

    let mut in_sp = false;
    let mut in_tbl = false;
    let mut sp_offset: (i64, i64) = (0, 0);
    let mut sp_is_title = false;
    // `<p:cNvPr name="Title 1">` is the most reliable fallback when the
    // placeholder lacks an explicit `type="title"` (some templates and
    // tools omit it). Captured per-shape and consumed at `</p:sp>`.
    let mut sp_name: Option<String> = None;
    // Did this shape carry any `<p:ph>`? Distinguishes a placeholder shape
    // (inherits master body style) from a free-form text box (master
    // other style).
    let mut sp_has_ph = false;

    // Picture-shape state. Mirrors the `in_sp` machinery but for `<p:pic>`
    // — we want a separate offset and a captured rel id pointing at
    // `ppt/media/imageN.*`.
    let mut in_pic = false;
    let mut pic_offset: (i64, i64) = (0, 0);
    let mut pic_rel_id: Option<String> = None;
    let mut pic_alt: Option<String> = None;

    // Group shape translation stack. Each `<p:grpSp>` introduces its own
    // child coordinate frame; the relative shift between the group's own
    // `<a:off>` (where it sits on the slide) and its `<a:chOff>` (the
    // origin children measure from) becomes a translation that applies
    // to every descendant. Nesting composes additively, which is correct
    // when groups don't scale (the common case). When they do scale, our
    // y/x are still in the right neighbourhood for sorting purposes.
    let mut grp_translation_stack: Vec<(i64, i64)> = Vec::new();
    let mut in_grp_sppr = false;
    let mut cur_grp_off: (i64, i64) = (0, 0);
    let mut cur_grp_choff: (i64, i64) = (0, 0);

    let mut parser = TextBodyParser::new();

    let mut tbl_rows: Vec<Vec<String>> = Vec::new();
    let mut cur_row: Vec<String> = Vec::new();

    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(e)) => {
                let name = e.local_name().as_ref().to_vec();
                match name.as_slice() {
                    b"sld" => {
                        if let Some(v) = attr_val(&e, b"show") {
                            if v == "0" || v == "false" {
                                hidden = true;
                            }
                        }
                    }
                    b"sp" => {
                        in_sp = true;
                        sp_offset = (0, 0);
                        sp_is_title = false;
                        sp_name = None;
                        sp_has_ph = false;
                    }
                    b"pic" => {
                        in_pic = true;
                        pic_offset = (0, 0);
                        pic_rel_id = None;
                        pic_alt = None;
                    }
                    b"cNvPr" => {
                        if in_sp {
                            sp_name = attr_val(&e, b"name");
                        } else if in_pic {
                            // Prefer alt-text (`descr`) over the shape's
                            // display name when both are present.
                            pic_alt = attr_val(&e, b"descr")
                                .or_else(|| attr_val(&e, b"name"));
                        }
                    }
                    b"blip" => {
                        if in_pic && pic_rel_id.is_none() {
                            pic_rel_id = attr_val(&e, b"embed");
                        }
                    }
                    b"ph" => {
                        sp_has_ph = true;
                        if let Some(t) = attr_val(&e, b"type") {
                            if t == "title" || t == "ctrTitle" {
                                sp_is_title = true;
                            }
                        }
                    }
                    b"off" => {
                        let (y, x) = read_xy(&e);
                        if in_grp_sppr {
                            cur_grp_off = (y, x);
                        } else if in_sp || in_tbl {
                            sp_offset = (y, x);
                        } else if in_pic {
                            pic_offset = (y, x);
                        }
                    }
                    b"chOff" => {
                        if in_grp_sppr {
                            cur_grp_choff = read_xy(&e);
                        }
                    }
                    b"grpSpPr" => {
                        in_grp_sppr = true;
                        cur_grp_off = (0, 0);
                        cur_grp_choff = (0, 0);
                    }
                    b"tbl" => {
                        in_tbl = true;
                        tbl_rows.clear();
                    }
                    b"tr" => {
                        cur_row.clear();
                    }
                    b"tc" => {}
                    other => parser.on_start(other, &e, rels),
                }
            }
            Ok(Event::Empty(e)) => {
                let name = e.local_name().as_ref().to_vec();
                match name.as_slice() {
                    b"cNvPr" => {
                        if in_sp {
                            sp_name = attr_val(&e, b"name");
                        } else if in_pic {
                            pic_alt = attr_val(&e, b"descr")
                                .or_else(|| attr_val(&e, b"name"));
                        }
                    }
                    b"blip" => {
                        if in_pic && pic_rel_id.is_none() {
                            pic_rel_id = attr_val(&e, b"embed");
                        }
                    }
                    b"off" => {
                        let (y, x) = read_xy(&e);
                        if in_grp_sppr {
                            cur_grp_off = (y, x);
                        } else if in_sp || in_tbl {
                            sp_offset = (y, x);
                        } else if in_pic {
                            pic_offset = (y, x);
                        }
                    }
                    b"chOff" => {
                        if in_grp_sppr {
                            cur_grp_choff = read_xy(&e);
                        }
                    }
                    b"ph" => {
                        sp_has_ph = true;
                        if let Some(t) = attr_val(&e, b"type") {
                            if t == "title" || t == "ctrTitle" {
                                sp_is_title = true;
                            }
                        }
                    }
                    other => parser.on_empty(other, &e, rels),
                }
            }
            Ok(Event::Text(t)) => parser.on_text(&t),
            Ok(Event::End(e)) => {
                let name = e.local_name().as_ref().to_vec();
                match name.as_slice() {
                    b"tc" => {
                        let cell_paragraphs = parser.take_paragraphs();
                        let cell_md = cell_paragraphs
                            .iter()
                            .map(|p| {
                                render_paragraph_with_bullet(p, None)
                                    .replace('\n', " ")
                                    .trim()
                                    .to_string()
                            })
                            .filter(|s| !s.is_empty())
                            .collect::<Vec<_>>()
                            .join(" ");
                        cur_row.push(cell_md);
                    }
                    b"tr" => {
                        tbl_rows.push(std::mem::take(&mut cur_row));
                    }
                    b"tbl" => {
                        in_tbl = false;
                        let (y, x) = effective_pos(sp_offset, &grp_translation_stack);
                        shapes.push(Shape {
                            y,
                            x,
                            kind: ShapeKind::Table(std::mem::take(&mut tbl_rows)),
                        });
                    }
                    b"grpSpPr" => {
                        in_grp_sppr = false;
                        grp_translation_stack.push((
                            cur_grp_off.0 - cur_grp_choff.0,
                            cur_grp_off.1 - cur_grp_choff.1,
                        ));
                    }
                    b"grpSp" => {
                        // Each `<p:grpSp>` pushes exactly one translation
                        // when its grpSpPr closes. Pop the matching one.
                        grp_translation_stack.pop();
                    }
                    b"pic" => {
                        in_pic = false;
                        if let Some(rel_id) = pic_rel_id.take() {
                            let (y, x) =
                                effective_pos(pic_offset, &grp_translation_stack);
                            let alt = pic_alt.take().unwrap_or_default();
                            shapes.push(Shape {
                                y,
                                x,
                                kind: ShapeKind::Picture { rel_id, alt },
                            });
                        }
                    }
                    b"sp" => {
                        in_sp = false;
                        let sp_paragraphs = parser.take_paragraphs();
                        let is_title = sp_is_title
                            || sp_name
                                .as_deref()
                                .map(|n| {
                                    let lower = n.trim().to_ascii_lowercase();
                                    lower.starts_with("title")
                                })
                                .unwrap_or(false);
                        let had_ph = sp_has_ph;
                        sp_name = None;
                        sp_has_ph = false;
                        let (y, x) = effective_pos(sp_offset, &grp_translation_stack);
                        if is_title {
                            let text = sp_paragraphs
                                .iter()
                                .map(|p| {
                                    p.runs
                                        .iter()
                                        .map(|r| r.text.as_str())
                                        .collect::<Vec<_>>()
                                        .join("")
                                })
                                .collect::<Vec<_>>()
                                .join(" ");
                            shapes.push(Shape {
                                y,
                                x,
                                kind: ShapeKind::Title(text),
                            });
                        } else if !sp_paragraphs.is_empty() {
                            // Placeholder shapes inherit body styles; free
                            // text boxes inherit "other" (typically no
                            // bullets in default masters).
                            let category = if had_ph {
                                StyleCategory::Body
                            } else {
                                StyleCategory::Other
                            };
                            shapes.push(Shape {
                                y,
                                x,
                                kind: ShapeKind::Body {
                                    category,
                                    paras: sp_paragraphs,
                                },
                            });
                        }
                    }
                    other => parser.on_end(other),
                }
            }
            Ok(Event::Eof) => break,
            Err(e) => return Err(ConvertError::Xml(format!("{e}"))),
            _ => {}
        }
        buf.clear();
    }

    Ok(Slide { hidden, shapes })
}

/// Walk a `slideMasterN.xml` and harvest default bullet styling per
/// `(category, level)` from the master's `<p:txStyles>` block.
///
/// The master defines three style cohorts:
/// - `<p:titleStyle>` — applied to the title placeholder
/// - `<p:bodyStyle>` — applied to body / outline placeholders
/// - `<p:otherStyle>` — applied to free-form text boxes
///
/// Each cohort holds nine optional `<a:lvlNpPr>` blocks (one per indent
/// level, 1-indexed in XML, 0-indexed in our [`Paragraph::ilvl`]). Each
/// level may carry a `<a:buChar>`, `<a:buAutoNum>`, or `<a:buNone>` —
/// that's what we capture. Other DML inside `<a:lvlNpPr>` (sizing,
/// indentation, fonts) is ignored.
fn parse_master_styles(xml: &str) -> MasterStyles {
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

struct RawComment {
    author_id: String,
    text: String,
}

/// Parse an authors document (`commentAuthors.xml` or `authors.xml`) into
/// `id → display_name`. Pass the local element name we expect on each
/// author entry (`b"cmAuthor"` for legacy, `b"author"` for modern).
fn parse_authors(xml: &str, element_local: &[u8]) -> HashMap<String, String> {
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

/// Parse a legacy `commentN.xml` (`<p:cmLst>` of `<p:cm>` entries with
/// inline `<p:text>` payloads). Returns one [`RawComment`] per entry.
fn parse_legacy_comments(xml: &str) -> Vec<RawComment> {
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
fn parse_modern_comment(xml: &str) -> Option<RawComment> {
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

/// Map element names like `lvl1pPr` … `lvl9pPr` to the 0-indexed level
/// used by [`Paragraph::ilvl`]. Returns `None` for everything else.
fn parse_lvl_ppr(name: &[u8]) -> Option<usize> {
    let s = std::str::from_utf8(name).ok()?;
    let n: usize = s.strip_prefix("lvl")?.strip_suffix("pPr")?.parse().ok()?;
    if n == 0 { None } else { Some(n - 1) }
}

fn render_table(rows: &[Vec<String>]) -> String {
    if rows.is_empty() {
        return String::new();
    }
    let cols = rows.iter().map(|r| r.len()).max().unwrap_or(0);
    if cols == 0 {
        return String::new();
    }
    let mut out = String::new();
    let header = normalize_row(&rows[0], cols);
    out.push_str(&format_row(&header));
    out.push('\n');
    out.push_str(&format_row(&vec!["---".to_string(); cols]));
    out.push('\n');
    for row in rows.iter().skip(1) {
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
