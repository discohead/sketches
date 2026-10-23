//! `studio new`: create a work-in-progress piece from the blank template.

use std::fs;
use std::path::Path;

use anyhow::{Context, Result, bail};
use piece_meta::{Canvas, Form, PieceMeta, TimeSpec, validate_slug, wip_package};

use crate::hang::set_package_name;
use crate::pieces::{self, Piece};
use crate::workspace::Workspace;

pub fn default_meta(slug: &str, today: &str) -> PieceMeta {
    PieceMeta {
        title: String::new(),
        slug: slug.to_string(),
        created: today.to_string(),
        statement: String::new(),
        tags: Vec::new(),
        series: None,
        parent: None,
        catalog: None,
        live: false,
        audio: false,
        canvas: Canvas { width: 1080, height: 1080 },
        time: TimeSpec { fps: 60, form: Form::Loop, seconds: 8.0, still_at: 0.0 },
    }
}

/// Create `pieces/<today>-<slug>/`. With `from`, it starts as a copy of that
/// piece's folder (everything but `target/`, dotfiles and `piece.toml`), with
/// the parent's `Cargo.toml` renamed to `wip-<slug>` and a fresh `piece.toml`
/// that keeps the parent's canvas, time, `live` and `audio`.
///
/// The piece is built in `.studio/new/` and renamed into `pieces/` at the end.
/// A folder under `pieces/` without a `Cargo.toml` breaks every cargo command in
/// the workspace, and cargo's `pieces/*` glob matches dotted folders too, so a
/// half-built piece must never sit there.
pub fn create(ws: &Workspace, slug: &str, from: Option<&str>, today: &str) -> Result<Piece> {
    validate_slug(slug)?;
    let folder = format!("{today}-{slug}");
    let package = wip_package(slug);
    let dir = ws.pieces_dir().join(&folder);
    if dir.exists() {
        bail!("pieces/{folder} already exists");
    }
    if pieces::all(ws)?.iter().any(|p| p.package == package) {
        bail!("package {package} already exists; pick another slug");
    }

    let parent = from.map(|name| pieces::resolve(ws, name)).transpose()?;

    let template = ws.templates_dir().join("blank");
    let cargo = match &parent {
        Some(p) => {
            let path = p.dir.join("Cargo.toml");
            let text = fs::read_to_string(&path)
                .with_context(|| format!("cannot read {}", path.display()))?;
            set_package_name(&text, &package)
                .with_context(|| format!("in {}", path.display()))?
        }
        None => fs::read_to_string(template.join("Cargo.toml.tmpl"))
            .with_context(|| format!("cannot read template in {}", template.display()))?
            .replace("{{package}}", &package),
    };

    let mut meta = default_meta(slug, today);
    if let Some(p) = &parent {
        meta.canvas = p.meta.canvas.clone();
        meta.time = p.meta.time.clone();
        meta.live = p.meta.live;
        meta.audio = p.meta.audio;
        meta.parent = Some(p.folder.clone());
    }

    let staging = ws.root.join(".studio/new").join(&folder);
    if staging.exists() {
        fs::remove_dir_all(&staging)
            .with_context(|| format!("cannot remove {}", staging.display()))?;
    }
    let filled = (|| -> Result<()> {
        fs::create_dir_all(staging.join("src"))
            .with_context(|| format!("cannot create {}", staging.display()))?;
        match &parent {
            Some(p) => copy_piece(&p.dir, &staging)?,
            None => {
                fs::copy(template.join("src/main.rs"), staging.join("src/main.rs"))
                    .context("cannot copy the template's src/main.rs")?;
            }
        }
        fs::write(staging.join("Cargo.toml"), &cargo)?;
        fs::write(staging.join("piece.toml"), meta.to_toml_string())?;
        fs::create_dir_all(ws.pieces_dir())?;
        if dir.exists() {
            bail!("pieces/{folder} already exists");
        }
        fs::rename(&staging, &dir)
            .with_context(|| format!("cannot move {} to {}", staging.display(), dir.display()))
    })();
    if let Err(e) = filled {
        let _ = fs::remove_dir_all(&staging);
        return Err(e);
    }

    Ok(Piece { dir, folder, package, meta })
}

/// Copy a parent piece's folder for a variation. `Cargo.toml` and `piece.toml`
/// are written fresh, `target/` is build output, and dotfiles (`.DS_Store`) are
/// litter.
fn copy_piece(from: &Path, to: &Path) -> Result<()> {
    for entry in fs::read_dir(from).with_context(|| format!("cannot read {}", from.display()))? {
        let entry = entry?;
        let name = entry.file_name();
        if matches!(name.to_str(), Some("target" | "Cargo.toml" | "piece.toml")) {
            continue;
        }
        copy_entry(&entry, &to.join(&name))?;
    }
    Ok(())
}

fn copy_dir(from: &Path, to: &Path) -> Result<()> {
    fs::create_dir_all(to).with_context(|| format!("cannot create {}", to.display()))?;
    for entry in fs::read_dir(from).with_context(|| format!("cannot read {}", from.display()))? {
        let entry = entry?;
        copy_entry(&entry, &to.join(entry.file_name()))?;
    }
    Ok(())
}

fn copy_entry(entry: &fs::DirEntry, dest: &Path) -> Result<()> {
    if entry.file_name().to_string_lossy().starts_with('.') {
        return Ok(());
    }
    let path = entry.path();
    if path.is_dir() {
        copy_dir(&path, dest)
    } else {
        fs::copy(&path, dest).with_context(|| format!("cannot copy {}", path.display()))?;
        Ok(())
    }
}
