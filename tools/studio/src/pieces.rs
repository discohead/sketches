//! Listing and finding the pieces under `pieces/`.

use std::path::PathBuf;

use anyhow::{Context, Result, bail};
use piece_meta::{PieceMeta, wip_package};

use crate::workspace::Workspace;

#[derive(Debug, Clone)]
pub struct Piece {
    pub dir: PathBuf,
    pub folder: String,
    pub package: String,
    pub meta: PieceMeta,
}

impl Piece {
    /// `wip` until the piece has a catalog number.
    pub fn state(&self) -> &str {
        self.meta.catalog.as_deref().unwrap_or("wip")
    }
}

/// Every folder under `pieces/` that holds a `piece.toml`, sorted by folder name.
pub fn all(ws: &Workspace) -> Result<Vec<Piece>> {
    let dir = ws.pieces_dir();
    let mut out = Vec::new();
    if !dir.is_dir() {
        return Ok(out);
    }
    for entry in std::fs::read_dir(&dir).with_context(|| format!("cannot read {}", dir.display()))? {
        let path = entry?.path();
        let toml_path = path.join("piece.toml");
        if !toml_path.is_file() {
            continue;
        }
        let folder = path.file_name().unwrap().to_string_lossy().into_owned();
        let text = std::fs::read_to_string(&toml_path)?;
        let meta = PieceMeta::from_toml_str(&text)
            .with_context(|| format!("in {}", toml_path.display()))?;
        let package = package_name(&path, &folder, &meta);
        out.push(Piece {
            dir: path,
            folder,
            package,
            meta,
        });
    }
    out.sort_by(|a, b| a.folder.cmp(&b.folder));
    Ok(out)
}

fn package_name(dir: &std::path::Path, folder: &str, meta: &PieceMeta) -> String {
    let from_cargo = std::fs::read_to_string(dir.join("Cargo.toml"))
        .ok()
        .and_then(|t| t.parse::<toml::Table>().ok())
        .and_then(|t| t.get("package")?.get("name")?.as_str().map(str::to_owned));
    from_cargo.unwrap_or_else(|| {
        if meta.catalog.is_some() {
            folder.to_string()
        } else {
            wip_package(&meta.slug)
        }
    })
}

/// Find one piece by folder, slug, or catalog number (case-insensitive).
pub fn resolve(ws: &Workspace, name: &str) -> Result<Piece> {
    let pieces = all(ws)?;
    let matches: Vec<&Piece> = pieces
        .iter()
        .filter(|p| {
            p.folder.eq_ignore_ascii_case(name)
                || p.meta.slug.eq_ignore_ascii_case(name)
                || p.meta
                    .catalog
                    .as_deref()
                    .is_some_and(|c| c.eq_ignore_ascii_case(name))
        })
        .collect();
    match matches.as_slice() {
        [one] => Ok((*one).clone()),
        [] => {
            let names: Vec<&str> = pieces.iter().map(|p| p.folder.as_str()).collect();
            bail!(
                "no piece matches {name:?}. Available: {}",
                if names.is_empty() { "(none)".into() } else { names.join(", ") }
            )
        }
        many => {
            let names: Vec<&str> = many.iter().map(|p| p.folder.as_str()).collect();
            bail!(
                "{name:?} matches {} pieces: {}. Use the folder name.",
                many.len(),
                names.join(", ")
            )
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn piece_toml(slug: &str, catalog: Option<&str>) -> String {
        let cat = catalog.map(|c| format!("catalog = \"{c}\"\n")).unwrap_or_default();
        format!(
            "title = \"\"\nslug = \"{slug}\"\ncreated = \"2026-10-22\"\nstatement = \"\"\n{cat}\n[canvas]\nwidth = 1080\nheight = 1080\n[time]\nfps = 60\nform = \"loop\"\nseconds = 8.0\nstill_at = 0.0\n"
        )
    }

    fn ws_with(pieces: &[(&str, &str, Option<&str>)]) -> (tempfile::TempDir, Workspace) {
        let d = tempfile::tempdir().unwrap();
        std::fs::write(
            d.path().join("studio.toml"),
            "[catalog]\nprefix = \"DSS\"\n[deposit]\ndir = \"o\"\n[private]\ndir = \"p\"\n[nannou]\nworktree = \"n\"\n",
        )
        .unwrap();
        for (folder, slug, cat) in pieces {
            let p = d.path().join("pieces").join(folder);
            std::fs::create_dir_all(&p).unwrap();
            std::fs::write(p.join("piece.toml"), piece_toml(slug, *cat)).unwrap();
        }
        let ws = Workspace::discover(d.path()).unwrap();
        (d, ws)
    }

    #[test]
    fn resolve_ambiguous() {
        let (_d, ws) = ws_with(&[
            ("2026-10-22-tide", "tide", None),
            ("dss0001-tide", "tide", Some("DSS0001")),
        ]);
        let e = resolve(&ws, "tide").unwrap_err().to_string();
        assert!(e.contains("2026-10-22-tide") && e.contains("dss0001-tide"), "{e}");
        assert_eq!(resolve(&ws, "DSS0001").unwrap().folder, "dss0001-tide");
        assert_eq!(resolve(&ws, "dss0001").unwrap().folder, "dss0001-tide");
        assert_eq!(resolve(&ws, "dss0001-tide").unwrap().folder, "dss0001-tide");
        assert_eq!(resolve(&ws, "dss0001-tide").unwrap().package, "dss0001-tide");
        assert_eq!(resolve(&ws, "2026-10-22-tide").unwrap().package, "wip-tide");
    }

    #[test]
    fn resolve_none_lists_available() {
        let (_d, ws) = ws_with(&[("2026-10-22-tide", "tide", None)]);
        let e = resolve(&ws, "zzz").unwrap_err().to_string();
        assert!(e.contains("2026-10-22-tide"), "{e}");
    }
}
