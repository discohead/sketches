//! Catalog numbers: the next free number after every piece already hung.

use std::path::Path;

use anyhow::{Context, Result};

use crate::workspace::Workspace;

/// One more than the highest number in `pieces/dss####-*` folders and
/// `outbox/DSS####` deposits. Gaps are never filled. An empty workspace gives 1.
pub fn next_number(ws: &Workspace) -> Result<u32> {
    let prefix = &ws.config.catalog.prefix;
    let mut max = 0;
    for name in dir_names(&ws.pieces_dir())? {
        // `dss0001-slug`: the catalog id in lower case, then a hyphen.
        if let Some((id, rest)) = name.split_once('-')
            && !rest.is_empty()
            && id.chars().all(|c| !c.is_ascii_uppercase())
            && let Some(n) = parse_catalog(prefix, id)
        {
            max = max.max(n);
        }
    }
    for name in dir_names(&ws.outbox_dir())? {
        if name.starts_with(prefix.as_str())
            && let Some(n) = parse_catalog(prefix, &name)
        {
            max = max.max(n);
        }
    }
    Ok(max + 1)
}

fn dir_names(dir: &Path) -> Result<Vec<String>> {
    if !dir.is_dir() {
        return Ok(Vec::new());
    }
    let mut out = Vec::new();
    for entry in std::fs::read_dir(dir).with_context(|| format!("cannot read {}", dir.display()))? {
        let entry = entry?;
        if entry.file_type()?.is_dir() {
            out.push(entry.file_name().to_string_lossy().into_owned());
        }
    }
    Ok(out)
}

/// The number in a catalog id: `parse_catalog("DSS", "DSS0012")` is `Some(12)`.
pub fn parse_catalog(prefix: &str, id: &str) -> Option<u32> {
    let head = id.get(..prefix.len())?;
    let digits = &id[prefix.len()..];
    if !head.eq_ignore_ascii_case(prefix) || digits.len() != 4 || !digits.bytes().all(|b| b.is_ascii_digit()) {
        return None;
    }
    digits.parse().ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ws() -> (tempfile::TempDir, Workspace) {
        let d = tempfile::tempdir().unwrap();
        std::fs::write(
            d.path().join("studio.toml"),
            "[catalog]\nprefix = \"DSS\"\n[deposit]\ndir = \"outbox\"\n[private]\ndir = \"p\"\n[nannou]\nworktree = \"n\"\n",
        )
        .unwrap();
        let ws = Workspace::discover(d.path()).unwrap();
        (d, ws)
    }

    #[test]
    fn next_number_skips_gaps() {
        let (_d, ws) = ws();
        assert_eq!(next_number(&ws).unwrap(), 1);
        for f in ["dss0001-a", "dss0003-b", "2026-10-22-wip"] {
            std::fs::create_dir_all(ws.pieces_dir().join(f)).unwrap();
        }
        assert_eq!(next_number(&ws).unwrap(), 4);
        std::fs::create_dir_all(ws.outbox_dir().join("DSS0004")).unwrap();
        // An unfinished attempt does not take a number.
        std::fs::create_dir_all(ws.outbox_dir().join(".DSS0009.partial")).unwrap();
        assert_eq!(next_number(&ws).unwrap(), 5);
    }

    #[test]
    fn parses_catalog_ids() {
        assert_eq!(parse_catalog("DSS", "DSS0012"), Some(12));
        assert_eq!(parse_catalog("DSS", "dss0012"), Some(12));
        assert_eq!(parse_catalog("DSS", "DSS12"), None);
        assert_eq!(parse_catalog("DSS", "DSS00a2"), None);
        assert_eq!(parse_catalog("DSS", "XYZ0001"), None);
    }
}
