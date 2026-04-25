use std::collections::{HashMap, HashSet};

use crate::convert::ConvertError;
use crate::convert::ooxml_util::{Zip, read_zip_bytes, unique_asset_name};

use super::DocxAsset;

/// Walk every `Type` ending with `/image` rel and collect its bytes from
/// `word/media/`. Builds `rId → final filename` so the document walker
/// only needs the rels map to emit attachments.
pub(super) fn collect_image_assets(
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
