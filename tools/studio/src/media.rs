//! ffmpeg helpers: contact-sheet montages.

use std::path::{Path, PathBuf};

use anyhow::{Context, Result, bail};

use crate::workspace::Workspace;

const TILE_WIDTH: u32 = 360;
const COLUMNS: usize = 3;

/// The ffmpeg arguments that tile `frames` into one image: each scaled to 360 px
/// wide, 3 columns, black padding.
pub fn montage_args(frames: &[PathBuf], out: &Path) -> Vec<String> {
    let n = frames.len();
    let rows = n.div_ceil(COLUMNS).max(1);
    let mut args: Vec<String> = vec!["-y".into(), "-loglevel".into(), "error".into()];
    for f in frames {
        args.push("-i".into());
        args.push(f.display().to_string());
    }
    let inputs: String = (0..n).map(|i| format!("[{i}:v]")).collect();
    let filter = format!(
        "{inputs}concat=n={n}:v=1:a=0,scale={TILE_WIDTH}:-1,tile={COLUMNS}x{rows}:padding=0:color=black"
    );
    args.extend(["-filter_complex".into(), filter]);
    args.extend(["-frames:v".into(), "1".into(), out.display().to_string()]);
    args
}

/// Build a contact sheet of `frames` at `out` with ffmpeg.
pub fn montage(ws: &Workspace, frames: &[PathBuf], out: &Path) -> Result<()> {
    if frames.is_empty() {
        bail!("no frames to put in a montage");
    }
    let output = ws
        .tool("ffmpeg")
        .args(montage_args(frames, out))
        .output()
        .context("could not run ffmpeg; install it with `brew install ffmpeg`")?;
    if !output.status.success() {
        bail!(
            "ffmpeg could not build the montage:\n{}",
            String::from_utf8_lossy(&output.stderr)
        );
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn montage_args() {
        let frames: Vec<PathBuf> = (0..6).map(|i| PathBuf::from(format!("{i:05}.png"))).collect();
        let args = super::montage_args(&frames, Path::new("out.png")).join(" ");
        assert!(args.contains("tile=3x2"), "{args}");
        assert!(args.contains("scale=360:-1"), "{args}");
        assert_eq!(args.matches("-i ").count(), 6);
    }
}
