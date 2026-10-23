//! `studio hang`: render, number, commit, push and deposit a finished piece.

use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result, bail};
use piece_meta::{Form, PieceMeta, catalog_id, hung_folder};

use crate::deposit::{self, Deposit};
use crate::pieces::{self, Piece};
use crate::workspace::Workspace;
use crate::{capture, catalog, media};

const TRAILER: &str = "Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>";

#[derive(Debug, Clone, Copy)]
pub struct HangOpts {
    pub push: bool,
}

/// Hang `name` and return the deposit dir. Frames are rendered fresh on every
/// attempt; every later step is skipped when already done, so a re-run after a
/// failure resumes with the same catalog number. An existing deposit is never
/// overwritten.
pub fn run(ws: &Workspace, name: &str, opts: HangOpts) -> Result<PathBuf> {
    // 1. Resolve and check the piece is ready.
    let piece = pieces::resolve(ws, name)?;
    let errors = piece.meta.readiness_errors();
    if !errors.is_empty() {
        bail!("{} is not ready to hang:\n  - {}", piece.folder, errors.join("\n  - "));
    }
    let total = piece.meta.total_frames();
    if total == 0 {
        bail!("{} has no frames: time.seconds is {}", piece.folder, piece.meta.time.seconds);
    }
    let origin = git(ws, &["remote", "get-url", "origin"])
        .context("this repo has no `origin` remote, which the deposit's source link needs")?;
    let repo = deposit::normalize_repo_url(&origin);

    // 2. Number: the piece's own number if it has one, otherwise the next free one.
    let prefix = &ws.config.catalog.prefix;
    let n = match &piece.meta.catalog {
        Some(c) => catalog::parse_catalog(prefix, c).with_context(|| {
            format!("catalog {c:?} in {}/piece.toml is not {prefix} plus four digits", piece.folder)
        })?,
        None => catalog::next_number(ws)?,
    };
    let catalog = catalog_id(prefix, n);
    let outbox = ws.outbox_dir();
    let final_dir = outbox.join(&catalog);
    if final_dir.exists() {
        bail!("{} already exists. A deposit is never overwritten.", final_dir.display());
    }

    // 3. Render every frame, fresh each attempt.
    let work = ws.root.join(".studio/hang").join(&catalog);
    let frames_dir = work.join("frames");
    remove_if_present(&frames_dir)?;
    eprintln!("{catalog}: rendering {total} frames of {}", piece.folder);
    let frames: Vec<u64> = (0..u64::from(total)).collect();
    capture::render(ws, &piece, &frames, &frames_dir)?;

    // 4. Encode into a partial dir that only becomes the deposit at the end.
    let partial = outbox.join(format!(".{catalog}.partial"));
    remove_if_present(&partial)?;
    fs::create_dir_all(&partial).with_context(|| format!("cannot create {}", partial.display()))?;
    eprintln!("{catalog}: encoding");
    encode_all(ws, &piece.meta, &frames_dir, &partial)?;

    // 5. Write the number, move to the hung folder, rename the package.
    let piece = move_to_hung(ws, &piece, &catalog, n)?;

    // 6. Commit the piece and Cargo.lock, nothing else.
    if is_committed(ws, &piece)? {
        eprintln!("{catalog}: already committed");
    } else {
        let check = ws
            .cargo()
            .args(["check", "-q", "-p", &piece.package])
            .output()
            .context("could not run cargo")?;
        if !check.status.success() {
            bail!(
                "cargo check -p {} failed:\n{}",
                piece.package,
                String::from_utf8_lossy(&check.stderr)
            );
        }
        commit(ws, &piece)?;
        eprintln!("{catalog}: committed");
    }

    // 7. Push.
    if opts.push {
        eprintln!("{catalog}: pushing");
        let out = ws
            .tool("git")
            .args(["push", "origin", "HEAD"])
            .output()
            .context("could not run git")?;
        if !out.status.success() {
            bail!(
                "push failed; fix and re-run `studio hang {catalog}`\n{}",
                String::from_utf8_lossy(&out.stderr).trim_end()
            );
        }
    }

    // 8. Write piece.json, then make the deposit appear in one rename.
    let commit_sha = git(ws, &["rev-parse", "HEAD"])?;
    let date = chrono::Local::now().format("%Y-%m-%d").to_string();
    let record = Deposit::new(
        &piece.meta,
        &catalog,
        &date,
        parent_catalog(ws, &piece.meta)?,
        &repo,
        &commit_sha,
        &format!("pieces/{}", piece.folder),
    );
    fs::write(partial.join("piece.json"), record.to_json())?;
    for m in &record.media {
        if !partial.join(&m.file).is_file() {
            bail!("{} is missing from {}", m.file, partial.display());
        }
    }
    if final_dir.exists() {
        bail!("{} already exists. A deposit is never overwritten.", final_dir.display());
    }
    fs::rename(&partial, &final_dir)
        .with_context(|| format!("cannot rename {} to {}", partial.display(), final_dir.display()))?;
    remove_if_present(&work)?;
    Ok(final_dir)
}

fn remove_if_present(dir: &Path) -> Result<()> {
    if dir.exists() {
        fs::remove_dir_all(dir).with_context(|| format!("cannot remove {}", dir.display()))?;
    }
    Ok(())
}

/// The still, master, web copy and social cuts, from `frames` into `out`.
fn encode_all(ws: &Workspace, meta: &PieceMeta, frames: &Path, out: &Path) -> Result<()> {
    let fps = meta.time.fps;
    let still = frames.join(format!("{:05}.png", still_frame(meta)));
    fs::copy(&still, out.join(deposit::STILL))
        .with_context(|| format!("cannot copy {}", still.display()))?;
    media::encode_master(ws, frames, fps, &out.join(deposit::MASTER))?;
    media::encode_web(ws, frames, fps, &out.join(deposit::WEB))?;
    let repeats = deposit::cut_repeats(meta);
    for c in deposit::CUTS {
        media::encode_cut(ws, frames, fps, repeats, c.width, c.height, &out.join(c.file))?;
    }
    Ok(())
}

/// The frame copied to `still.png`: `round(still_at * fps)`, kept inside the piece.
/// At the very end, a loop wraps to frame 0 and a fixed duration uses its last frame.
pub fn still_frame(meta: &PieceMeta) -> u64 {
    let total = u64::from(meta.total_frames());
    let k = (meta.time.still_at * meta.time.fps as f32).round().max(0.0) as u64;
    if k < total {
        k
    } else {
        match meta.time.form {
            Form::Loop => 0,
            Form::Duration => total.saturating_sub(1),
        }
    }
}

/// `text` with the `[package]` name set to `name`, everything else untouched.
pub fn set_package_name(text: &str, name: &str) -> Result<String> {
    let mut out = String::with_capacity(text.len() + name.len());
    let mut section = String::new();
    let mut done = false;
    for line in text.split_inclusive('\n') {
        let trimmed = line.trim_start();
        if let Some(rest) = trimmed.strip_prefix('[') {
            section = rest.split(']').next().unwrap_or("").trim().to_string();
        } else if !done
            && section == "package"
            && let Some((key, value)) = trimmed.split_once('=')
            && key.trim() == "name"
            && let Some(open) = value.find('"')
            && let Some(len) = value[open + 1..].find('"')
        {
            let start = line.len() - value.len() + open + 1;
            out.push_str(&line[..start]);
            out.push_str(name);
            out.push_str(&line[start + len..]);
            done = true;
            continue;
        }
        out.push_str(line);
    }
    let parsed: toml::Table = out.parse().context("Cargo.toml is not valid TOML after renaming")?;
    let got = parsed.get("package").and_then(|p| p.get("name")).and_then(|n| n.as_str());
    if !done || got != Some(name) {
        bail!("could not find `name = \"...\"` under [package] in Cargo.toml");
    }
    Ok(out)
}

/// The parent's catalog id when the parent is hung, otherwise `None`.
pub fn parent_catalog(ws: &Workspace, meta: &PieceMeta) -> Result<Option<String>> {
    let Some(parent) = &meta.parent else {
        return Ok(None);
    };
    let all = pieces::all(ws)?;
    if let Some(p) = all.iter().find(|p| &p.folder == parent) {
        return Ok(p.meta.catalog.clone());
    }
    // A working folder `YYYY-MM-DD-slug` whose piece has since been hung.
    if let Some((date, rest)) = parent.split_at_checked(10)
        && let Some(slug) = rest.strip_prefix('-')
    {
        let hung = all
            .iter()
            .find(|p| p.meta.catalog.is_some() && p.meta.created == date && p.meta.slug == slug);
        if let Some(p) = hung {
            return Ok(p.meta.catalog.clone());
        }
    }
    Ok(None)
}

/// Run git in the repo root and return its trimmed stdout.
fn git(ws: &Workspace, args: &[&str]) -> Result<String> {
    let out = ws.tool("git").args(args).output().context("could not run git")?;
    if !out.status.success() {
        bail!("git {} failed:\n{}", args.join(" "), String::from_utf8_lossy(&out.stderr).trim_end());
    }
    Ok(String::from_utf8_lossy(&out.stdout).trim().to_string())
}

/// Write the catalog into `piece.toml`, move the folder to its hung name, and
/// rename the package. Each part is skipped if already done.
pub fn move_to_hung(ws: &Workspace, piece: &Piece, catalog: &str, n: u32) -> Result<Piece> {
    let target = hung_folder(n, &piece.meta.slug);
    let mut meta = piece.meta.clone();
    match meta.catalog.as_deref() {
        Some(c) if c == catalog => {}
        Some(c) => bail!("{} already has catalog {c}, not {catalog}", piece.folder),
        None => {
            meta.catalog = Some(catalog.to_string());
            fs::write(piece.dir.join("piece.toml"), meta.to_toml_string())?;
        }
    }

    let dir = ws.pieces_dir().join(&target);
    if piece.folder != target {
        if dir.exists() {
            bail!("pieces/{target} already exists");
        }
        let from = format!("pieces/{}", piece.folder);
        let to = format!("pieces/{target}");
        if git(ws, &["ls-files", "--", &from])?.is_empty() {
            fs::rename(&piece.dir, &dir)
                .with_context(|| format!("cannot move {from} to {to}"))?;
        } else {
            git(ws, &["mv", &from, &to])?;
        }
    }

    let cargo_toml = dir.join("Cargo.toml");
    let text = fs::read_to_string(&cargo_toml)
        .with_context(|| format!("cannot read {}", cargo_toml.display()))?;
    let renamed = set_package_name(&text, &target)?;
    if renamed != text {
        fs::write(&cargo_toml, renamed)?;
    }

    Ok(Piece { dir, folder: target.clone(), package: target, meta })
}

/// Folders under `pieces/` in `HEAD` that are gone from disk and held this
/// piece (same slug and creation date): its working folder before the hang.
fn old_folders_at_head(ws: &Workspace, piece: &Piece) -> Result<Vec<String>> {
    let Ok(listing) = git(ws, &["ls-tree", "--name-only", "HEAD", "pieces/"]) else {
        return Ok(Vec::new()); // no commits yet
    };
    let mut out = Vec::new();
    for path in listing.lines() {
        if path == format!("pieces/{}", piece.folder) || ws.root.join(path).exists() {
            continue;
        }
        let Ok(text) = git(ws, &["show", &format!("HEAD:{path}/piece.toml")]) else {
            continue;
        };
        if let Ok(meta) = PieceMeta::from_toml_str(&text)
            && meta.slug == piece.meta.slug
            && meta.created == piece.meta.created
        {
            out.push(path.to_string());
        }
    }
    Ok(out)
}

/// True when the hung folder and `Cargo.lock` have no changes, the folder is in
/// `HEAD`, and the working folder is gone from `HEAD`.
pub fn is_committed(ws: &Workspace, piece: &Piece) -> Result<bool> {
    let path = format!("pieces/{}", piece.folder);
    if !git(ws, &["status", "--porcelain", "--", &path, "Cargo.lock"])?.is_empty() {
        return Ok(false);
    }
    let at_head = git(ws, &["ls-tree", "--name-only", "HEAD", "--", &path]).unwrap_or_default();
    Ok(!at_head.is_empty() && old_folders_at_head(ws, piece)?.is_empty())
}

/// Commit the hung folder, the removal of its working folder, and `Cargo.lock`.
/// Nothing else is staged or committed.
pub fn commit(ws: &Workspace, piece: &Piece) -> Result<()> {
    let catalog = piece.meta.catalog.as_deref().context("the piece has no catalog number yet")?;
    let mut paths = vec![format!("pieces/{}", piece.folder)];
    if ws.root.join("Cargo.lock").is_file() {
        paths.push("Cargo.lock".to_string());
    }
    let mut add = vec!["add", "-A", "--"];
    add.extend(paths.iter().map(String::as_str));
    git(ws, &add)?;
    paths.extend(old_folders_at_head(ws, piece)?);

    let message = format!("hang: {catalog} {}\n\n{TRAILER}", piece.meta.title.trim());
    let mut args = vec!["commit", "-q", "-m", &message, "--"];
    args.extend(paths.iter().map(String::as_str));
    git(ws, &args)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::path::Path;
    use std::process::Command;

    const STUDIO: &str = "[catalog]\nprefix = \"DSS\"\n[deposit]\ndir = \"outbox\"\n[private]\ndir = \"p\"\n[nannou]\nworktree = \"n\"\n";

    fn ready_meta(slug: &str) -> PieceMeta {
        let mut m = crate::new::default_meta(slug, "2026-10-22");
        m.title = "Tide Pool".into();
        m.statement = "Water.".into();
        m
    }

    fn add_piece(root: &Path, folder: &str, meta: &PieceMeta, package: &str) -> PathBuf {
        let dir = root.join("pieces").join(folder);
        fs::create_dir_all(dir.join("src")).unwrap();
        fs::write(dir.join("piece.toml"), meta.to_toml_string()).unwrap();
        fs::write(
            dir.join("Cargo.toml"),
            format!("[package]\nname = \"{package}\" # the name\nversion = \"0.1.0\"\n\n[dependencies]\nname = \"not-this\"\n"),
        )
        .unwrap();
        fs::write(dir.join("src/main.rs"), "fn main() {}\n").unwrap();
        dir
    }

    fn sh(root: &Path, args: &[&str]) -> String {
        let out = Command::new("git").current_dir(root).args(args).output().unwrap();
        assert!(out.status.success(), "git {args:?}: {}", String::from_utf8_lossy(&out.stderr));
        String::from_utf8_lossy(&out.stdout).trim().to_string()
    }

    /// A git repo with studio.toml, Cargo.lock and an unrelated file committed.
    fn repo() -> (tempfile::TempDir, Workspace) {
        let d = tempfile::tempdir().unwrap();
        let root = d.path();
        fs::write(root.join("studio.toml"), STUDIO).unwrap();
        fs::write(root.join("Cargo.lock"), "# lock v1\n").unwrap();
        fs::write(root.join("notes.txt"), "a\n").unwrap();
        fs::create_dir_all(root.join("pieces")).unwrap();
        sh(root, &["init", "-q", "-b", "main"]);
        sh(root, &["config", "user.name", "Test"]);
        sh(root, &["config", "user.email", "test@example.com"]);
        sh(root, &["config", "commit.gpgsign", "false"]);
        sh(root, &["add", "-A"]);
        sh(root, &["commit", "-q", "-m", "init"]);
        let ws = Workspace::discover(root).unwrap();
        (d, ws)
    }

    #[test]
    fn refuses_unready() {
        let d = tempfile::tempdir().unwrap();
        fs::write(d.path().join("studio.toml"), STUDIO).unwrap();
        let mut m = ready_meta("tide");
        m.statement = "  ".into();
        add_piece(d.path(), "2026-10-22-tide", &m, "wip-tide");
        let ws = Workspace::discover(d.path()).unwrap();
        let e = run(&ws, "tide", HangOpts { push: false }).unwrap_err().to_string();
        assert!(e.contains("statement"), "{e}");
        assert!(!ws.root.join(".studio").exists(), "rendered before refusing");
        assert!(!ws.outbox_dir().exists());
    }

    #[test]
    fn still_frame_rounds_and_stays_inside() {
        let mut m = ready_meta("x");
        m.time.still_at = 2.51;
        assert_eq!(still_frame(&m), 151);
        m.time.still_at = 8.0; // the loop's end is its start
        assert_eq!(still_frame(&m), 0);
        m.time.form = piece_meta::Form::Duration;
        assert_eq!(still_frame(&m), 479);
    }

    #[test]
    fn set_package_name_edits_only_package() {
        let text = "[package]\nname = \"wip-tide\" # the name\nversion = \"0.1.0\"\n\n[dependencies]\nname = \"not-this\"\n";
        let out = set_package_name(text, "dss0001-tide").unwrap();
        assert_eq!(
            out,
            "[package]\nname = \"dss0001-tide\" # the name\nversion = \"0.1.0\"\n\n[dependencies]\nname = \"not-this\"\n"
        );
        let t: toml::Table = out.parse().unwrap();
        assert_eq!(t["package"]["name"].as_str(), Some("dss0001-tide"));
        assert!(set_package_name("[dependencies]\nx = 1\n", "a").is_err());
    }

    #[test]
    fn parent_catalog_follows_hung_parents() {
        let d = tempfile::tempdir().unwrap();
        fs::write(d.path().join("studio.toml"), STUDIO).unwrap();
        let mut hung = ready_meta("tide");
        hung.catalog = Some("DSS0003".into());
        add_piece(d.path(), "dss0003-tide", &hung, "dss0003-tide");
        add_piece(d.path(), "2026-10-21-ebb", &ready_meta("ebb"), "wip-ebb");
        let ws = Workspace::discover(d.path()).unwrap();

        let mut child = ready_meta("child");
        assert_eq!(parent_catalog(&ws, &child).unwrap(), None);
        // Recorded before the parent was hung: its working folder name.
        child.parent = Some("2026-10-22-tide".into());
        assert_eq!(parent_catalog(&ws, &child).unwrap().as_deref(), Some("DSS0003"));
        // Recorded after: its hung folder.
        child.parent = Some("dss0003-tide".into());
        assert_eq!(parent_catalog(&ws, &child).unwrap().as_deref(), Some("DSS0003"));
        // Parent not hung.
        child.parent = Some("2026-10-21-ebb".into());
        assert_eq!(parent_catalog(&ws, &child).unwrap(), None);
        // Parent gone.
        child.parent = Some("2026-01-01-gone".into());
        assert_eq!(parent_catalog(&ws, &child).unwrap(), None);
    }

    fn hang_in_repo(ws: &Workspace, tracked: bool) {
        let root = &ws.root;
        add_piece(root, "2026-10-22-tide", &ready_meta("tide"), "wip-tide");
        if tracked {
            sh(root, &["add", "pieces"]);
            sh(root, &["commit", "-q", "-m", "wip"]);
        }
        // Unrelated work that must stay out of the hang commit.
        fs::write(root.join("notes.txt"), "a\nb\n").unwrap();
        fs::write(root.join("staged.txt"), "x\n").unwrap();
        sh(root, &["add", "staged.txt"]);
        let piece = crate::pieces::resolve(ws, "tide").unwrap();
        let hung = move_to_hung(ws, &piece, "DSS0001", 1).unwrap();
        // Running it again changes nothing.
        let again = move_to_hung(ws, &hung, "DSS0001", 1).unwrap();
        assert_eq!(again.folder, "dss0001-tide");
        assert_eq!(hung.folder, "dss0001-tide");
        assert_eq!(hung.package, "dss0001-tide");
        assert_eq!(hung.meta.catalog.as_deref(), Some("DSS0001"));
        assert!(!root.join("pieces/2026-10-22-tide").exists());
        let on_disk = PieceMeta::from_toml_str(
            &fs::read_to_string(root.join("pieces/dss0001-tide/piece.toml")).unwrap(),
        )
        .unwrap();
        assert_eq!(on_disk.catalog.as_deref(), Some("DSS0001"));
        fs::write(root.join("Cargo.lock"), "# lock v2\n").unwrap();

        assert!(!is_committed(ws, &hung).unwrap());
        commit(ws, &hung).unwrap();
        assert!(is_committed(ws, &hung).unwrap());

        assert_eq!(
            sh(root, &["log", "-1", "--format=%B"]),
            "hang: DSS0001 Tide Pool\n\nCo-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>"
        );
        let mut files: Vec<String> = sh(root, &["show", "--no-renames", "--name-only", "--format=", "HEAD"])
            .lines()
            .map(str::to_owned)
            .collect();
        files.sort();
        let mut want = vec![
            "Cargo.lock",
            "pieces/dss0001-tide/Cargo.toml",
            "pieces/dss0001-tide/piece.toml",
            "pieces/dss0001-tide/src/main.rs",
        ];
        if tracked {
            want.extend([
                "pieces/2026-10-22-tide/Cargo.toml",
                "pieces/2026-10-22-tide/piece.toml",
                "pieces/2026-10-22-tide/src/main.rs",
            ]);
        }
        want.sort();
        assert_eq!(files, want);
        assert_eq!(sh(root, &["ls-tree", "-r", "--name-only", "HEAD", "--", "pieces/2026-10-22-tide"]), "");
        // The unrelated changes are where they were.
        let status = sh(root, &["status", "--porcelain"]);
        assert!(status.contains("A  staged.txt"), "{status}");
        assert!(status.lines().any(|l| l.trim() == "M notes.txt"), "{status}");
        assert!(!status.contains("pieces"), "{status}");
    }

    #[test]
    fn commit_tracked_piece() {
        let (_d, ws) = repo();
        hang_in_repo(&ws, true);
    }

    #[test]
    fn commit_untracked_piece() {
        let (_d, ws) = repo();
        hang_in_repo(&ws, false);
    }
}
