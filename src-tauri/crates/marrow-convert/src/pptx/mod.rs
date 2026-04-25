//! Native PPTX → Markdown conversion.
//!
//! Entry point: [`pptx_to_markdown`]. Sub-modules:
//!
//! - [`types`] — `Slide`, `Shape`, `ShapeKind`, `StyleCategory`,
//!   `MasterStyles` type alias
//! - [`attrs`] — generic `attr_val` quick-xml helper
//! - [`master`] — `parse_master_styles` (slideMaster bullet defaults)
//! - [`sections`] — `build_section_map` (presentation.xml sectionLst)
//! - [`authors`] — comment author maps (legacy + modern)
//! - [`comments`] — `parse_legacy_comments` / `parse_modern_comment`
//! - [`notes`] — `render_speaker_notes`
//! - [`slide`] — `parse_slide` (the per-slide streaming pass)
//! - [`table`] — markdown table rendering helpers

use std::collections::{HashMap, HashSet};

use crate::ConvertError;
use crate::ooxml::dml::{
    BulletKind, extract_paragraphs, render_paragraph_with_bullet,
};
use crate::ooxml::util::{
    find_rel_target, find_rel_targets, list_zip_names, open_zip, parse_rels,
    post_process, read_zip_bytes, read_zip_text, resolve_rel_path,
    unique_asset_name,
};

mod attrs;
mod authors;
mod comments;
mod master;
mod notes;
mod sections;
mod slide;
mod table;
mod types;

use authors::parse_authors;
use comments::{RawComment, parse_legacy_comments, parse_modern_comment};
use master::parse_master_styles;
use notes::render_speaker_notes;
use sections::build_section_map;
use slide::parse_slide;
use table::render_table;
use types::{MasterStyles, ShapeKind};

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

fn slide_ordinal(path: &str) -> Option<i64> {
    let name = path.rsplit('/').next()?;
    let stem = name.strip_prefix("slide")?.strip_suffix(".xml")?;
    stem.parse().ok()
}
