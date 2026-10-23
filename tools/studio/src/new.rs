//! `studio new`: create a work-in-progress piece from the blank template.

use std::fs;
use std::path::Path;

use anyhow::{Context, Result, bail};
use piece_meta::{Canvas, Form, PieceMeta, TimeSpec, validate_slug, wip_package};

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
    let cargo_tmpl = fs::read_to_string(template.join("Cargo.toml.tmpl"))
        .with_context(|| format!("cannot read template in {}", template.display()))?;
    let cargo = cargo_tmpl.replace("{{package}}", &package);

    let mut meta = default_meta(slug, today);
    fs::create_dir_all(dir.join("src"))?;
    match &parent {
        Some(p) => {
            copy_dir(&p.dir.join("src"), &dir.join("src"))?;
            meta.canvas = p.meta.canvas.clone();
            meta.time = p.meta.time.clone();
            meta.parent = Some(p.folder.clone());
        }
        None => {
            fs::copy(template.join("src/main.rs"), dir.join("src/main.rs"))?;
        }
    }
    fs::write(dir.join("Cargo.toml"), cargo)?;
    fs::write(dir.join("piece.toml"), meta.to_toml_string())?;

    Ok(Piece { dir, folder, package, meta })
}

fn copy_dir(from: &Path, to: &Path) -> Result<()> {
    fs::create_dir_all(to)?;
    for entry in fs::read_dir(from).with_context(|| format!("cannot read {}", from.display()))? {
        let entry = entry?;
        let dest = to.join(entry.file_name());
        if entry.file_type()?.is_dir() {
            copy_dir(&entry.path(), &dest)?;
        } else {
            fs::copy(entry.path(), dest)?;
        }
    }
    Ok(())
}
