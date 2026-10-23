//! Rendering chosen frames of a piece offline, through kit's capture mode.

use std::collections::BTreeSet;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

use anyhow::{Context, Result, bail};
use piece_meta::PieceMeta;

use crate::pieces::Piece;
use crate::workspace::Workspace;

/// `60 s + 4 × (max_frame + 1) / fps`, rounded up to whole seconds.
pub fn timeout_for(max_frame: u64, fps: u32) -> Duration {
    let fps = fps.max(1) as u64;
    Duration::from_secs(60 + (4 * (max_frame + 1)).div_ceil(fps))
}

/// `round(t * fps)` for each time, sorted and deduplicated.
pub fn frames_for_times(times: &[f32], fps: u32) -> Vec<u64> {
    let set: BTreeSet<u64> = times
        .iter()
        .map(|t| (t * fps as f32).round().max(0.0) as u64)
        .collect();
    set.into_iter().collect()
}

/// `n` times across `[0, seconds)`, starting at 0.
pub fn evenly_spaced(meta: &PieceMeta, n: usize) -> Vec<f32> {
    (0..n).map(|i| meta.time.seconds * i as f32 / n as f32).collect()
}

/// Where cargo puts the piece binary.
fn binary_path(ws: &Workspace, package: &str) -> PathBuf {
    let target = match std::env::var_os("CARGO_TARGET_DIR") {
        Some(t) if !t.is_empty() => PathBuf::from(t),
        _ => ws.root.join("target"),
    };
    target.join("debug").join(package)
}

fn frame_path(out: &Path, k: u64) -> PathBuf {
    out.join(format!("{k:05}.png"))
}

/// Width and height from a PNG's IHDR chunk.
fn png_size(path: &Path) -> Result<(u32, u32)> {
    let mut head = [0u8; 24];
    std::fs::File::open(path)
        .and_then(|mut f| f.read_exact(&mut head))
        .with_context(|| format!("cannot read {}", path.display()))?;
    if &head[..8] != b"\x89PNG\r\n\x1a\n" {
        bail!("{} is not a PNG file", path.display());
    }
    let w = u32::from_be_bytes(head[16..20].try_into().unwrap());
    let h = u32::from_be_bytes(head[20..24].try_into().unwrap());
    Ok((w, h))
}

fn last_lines(text: &str, n: usize) -> String {
    let lines: Vec<&str> = text.lines().collect();
    lines[lines.len().saturating_sub(n)..].join("\n")
}

/// The command that runs a built piece in capture mode. Bevy looks for assets
/// under `BEVY_ASSET_ROOT` before `CARGO_MANIFEST_DIR`, which studio inherits
/// from `cargo run`, so it is set to the piece's own folder.
fn capture_command(bin: &Path, piece: &Piece, out: &Path, spec: &str) -> Command {
    let mut cmd = Command::new(bin);
    cmd.current_dir(&piece.dir)
        .env("BEVY_ASSET_ROOT", &piece.dir)
        .env("KIT_CAPTURE_DIR", out)
        .env("KIT_CAPTURE_FRAMES", spec);
    cmd
}

/// Build the piece, run it in capture mode, and return the PNG for each frame.
pub fn render(ws: &Workspace, piece: &Piece, frames: &[u64], out: &Path) -> Result<Vec<PathBuf>> {
    let Some(&max_frame) = frames.iter().max() else {
        bail!("no frames requested");
    };
    std::fs::create_dir_all(out).with_context(|| format!("cannot create {}", out.display()))?;

    let build = ws
        .cargo()
        .args(["build", "-p", &piece.package])
        .output()
        .context("could not run cargo")?;
    if !build.status.success() {
        bail!(
            "building {} failed:\n{}",
            piece.package,
            String::from_utf8_lossy(&build.stderr)
        );
    }

    let bin = binary_path(ws, &piece.package);
    if !bin.is_file() {
        bail!("built {} but cannot find the binary at {}", piece.package, bin.display());
    }
    let spec = frames.iter().map(u64::to_string).collect::<Vec<_>>().join(",");
    let stderr_file = out.join("stderr.log");
    let mut child = capture_command(&bin, piece, out, &spec)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(std::fs::File::create(&stderr_file)?)
        .spawn()
        .with_context(|| format!("cannot start {}", bin.display()))?;

    let limit = timeout_for(max_frame, piece.meta.time.fps);
    let start = Instant::now();
    let status = loop {
        if let Some(s) = child.try_wait()? {
            break s;
        }
        if start.elapsed() > limit {
            let _ = child.kill();
            let _ = child.wait();
            bail!(
                "{} did not finish capturing within {} s and was stopped. \
                 Its update loop may be stuck.",
                piece.folder,
                limit.as_secs()
            );
        }
        std::thread::sleep(Duration::from_millis(100));
    };
    if !status.success() {
        let log = std::fs::read_to_string(&stderr_file).unwrap_or_default();
        bail!(
            "{} exited with {} while capturing. Last lines of its output:\n{}",
            piece.folder,
            status,
            last_lines(&log, 40)
        );
    }

    let (w, h) = (piece.meta.canvas.width, piece.meta.canvas.height);
    let mut paths = Vec::new();
    for &k in frames {
        let p = frame_path(out, k);
        if !p.is_file() {
            bail!(
                "frame {k} was not saved ({}). The window may have been closed during capture.",
                p.display()
            );
        }
        let (pw, ph) = png_size(&p)?;
        if (pw, ph) != (w, h) {
            bail!("frame {k} is {pw}x{ph} but the canvas is {w}x{h}");
        }
        paths.push(p);
    }
    Ok(paths)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn meta(seconds: f32) -> PieceMeta {
        crate::new::default_meta("x", "2026-01-01").tap(|m| m.time.seconds = seconds)
    }

    trait Tap: Sized {
        fn tap(mut self, f: impl FnOnce(&mut Self)) -> Self {
            f(&mut self);
            self
        }
    }
    impl Tap for PieceMeta {}

    #[test]
    fn frames_for_times_rounds() {
        assert_eq!(frames_for_times(&[0.0, 0.5, 0.51], 60), vec![0, 30, 31]);
        assert_eq!(frames_for_times(&[0.5, 0.5], 60), vec![30]);
    }

    #[test]
    fn evenly_spaced_six() {
        let t = evenly_spaced(&meta(8.0), 6);
        let want = [0.0, 1.3333, 2.6667, 4.0, 5.3333, 6.6667];
        assert_eq!(t.len(), 6);
        for (a, b) in t.iter().zip(want) {
            assert!((a - b).abs() < 1e-3, "{t:?}");
        }
    }

    #[test]
    fn timeout_scales_with_frames() {
        assert_eq!(timeout_for(59, 30), Duration::from_secs(68));
        assert_eq!(timeout_for(479, 60), Duration::from_secs(92));
    }

    #[test]
    fn capture_command_sets_asset_root() {
        let piece = Piece {
            dir: PathBuf::from("/w/pieces/2026-01-01-x"),
            folder: "2026-01-01-x".into(),
            package: "wip-x".into(),
            meta: meta(1.0),
        };
        let cmd = capture_command(Path::new("/t/wip-x"), &piece, Path::new("/o"), "0,1");
        let env = |k: &str| {
            cmd.get_envs()
                .find(|(key, _)| *key == k)
                .and_then(|(_, v)| v)
                .map(|v| v.to_string_lossy().into_owned())
        };
        assert_eq!(env("BEVY_ASSET_ROOT").as_deref(), Some("/w/pieces/2026-01-01-x"));
        assert_eq!(env("KIT_CAPTURE_DIR").as_deref(), Some("/o"));
        assert_eq!(env("KIT_CAPTURE_FRAMES").as_deref(), Some("0,1"));
        assert_eq!(cmd.get_current_dir(), Some(piece.dir.as_path()));
    }

    #[test]
    fn png_size_reads_header() {
        let d = tempfile::tempdir().unwrap();
        let p = d.path().join("a.png");
        let mut b = b"\x89PNG\r\n\x1a\n\0\0\0\rIHDR".to_vec();
        b.extend(1080u32.to_be_bytes());
        b.extend(720u32.to_be_bytes());
        std::fs::write(&p, b).unwrap();
        assert_eq!(png_size(&p).unwrap(), (1080, 720));
    }

    #[test]
    #[ignore = "needs cargo"]
    fn render_reports_build_failure() {
        let d = tempfile::tempdir().unwrap();
        let root = d.path();
        std::fs::write(
            root.join("studio.toml"),
            "[catalog]\nprefix=\"DSS\"\n[deposit]\ndir=\"outbox\"\n[private]\ndir=\"p\"\n[nannou]\nworktree=\"n\"\n",
        )
        .unwrap();
        std::fs::write(root.join("Cargo.toml"), "[workspace]\nmembers=[\"pieces/*\"]\nresolver=\"2\"\n").unwrap();
        let dir = root.join("pieces/2026-01-01-bad");
        std::fs::create_dir_all(dir.join("src")).unwrap();
        std::fs::write(
            dir.join("Cargo.toml"),
            "[package]\nname=\"wip-bad\"\nversion=\"0.1.0\"\nedition=\"2021\"\n",
        )
        .unwrap();
        std::fs::write(dir.join("src/main.rs"), "fn main() { let x: u32 = \"a\"; }\n").unwrap();
        let ws = Workspace::discover(root).unwrap();
        let piece = Piece {
            dir,
            folder: "2026-01-01-bad".into(),
            package: "wip-bad".into(),
            meta: meta(1.0),
        };
        let err = render(&ws, &piece, &[0], &root.join("out")).unwrap_err().to_string();
        assert!(err.contains("error["), "{err}");
    }
}
