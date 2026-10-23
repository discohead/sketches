//! ffmpeg helpers: contact-sheet montages and the deposit's video encodes.

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

/// The scale-and-matte filter for a social cut: fit inside `w`×`h` without
/// cropping, centre, and pad with black.
pub fn cut_filter(w: u32, h: u32) -> String {
    format!(
        "scale={w}:{h}:force_original_aspect_ratio=decrease:flags=lanczos,pad={w}:{h}:(ow-iw)/2:(oh-ih)/2:black,setsar=1"
    )
}

/// How many times a loop repeats so a cut lasts at least `min_seconds`:
/// `max(1, ceil(min_seconds / seconds))`. Only loops repeat; a fixed-duration
/// piece plays once, so callers pass 1 for those.
pub fn loop_repeats(seconds: f32, min_seconds: f32) -> u32 {
    if seconds <= 0.0 || !seconds.is_finite() {
        return 1;
    }
    ((min_seconds / seconds).ceil() as u32).max(1)
}

/// Input arguments for the numbered PNGs `00000.png`, `00001.png`, … in `frames_dir`,
/// played `repeats` times in a row.
fn input_args(frames_dir: &Path, fps: u32, repeats: u32) -> Vec<String> {
    vec![
        "-y".into(),
        "-loglevel".into(),
        "error".into(),
        "-stream_loop".into(),
        repeats.saturating_sub(1).to_string(),
        "-framerate".into(),
        fps.to_string(),
        "-start_number".into(),
        "0".into(),
        "-i".into(),
        frames_dir.join("%05d.png").display().to_string(),
        "-an".into(),
    ]
}

fn strs(args: &[&str]) -> impl Iterator<Item = String> {
    args.iter().map(|a| a.to_string())
}

/// The filter stage that converts the RGB frames to limited-range BT.709 YUV in
/// `pix_fmt` and tags the frames as BT.709. Without it ffmpeg converts with the
/// BT.601 matrix and leaves the colour untagged.
pub fn color_stage(pix_fmt: &str) -> String {
    format!(
        "scale=out_color_matrix=bt709:out_range=tv,format={pix_fmt},\
         setparams=color_primaries=bt709:color_trc=bt709:colorspace=bt709:range=tv"
    )
}

/// Container-level BT.709 tags, matching `color_stage`.
const COLOR_TAGS: [&str; 8] = [
    "-colorspace", "bt709", "-color_primaries", "bt709", "-color_trc", "bt709", "-color_range", "tv",
];

/// ffmpeg arguments for the archival master: H.264, `-preset slow -crf 10`, 4:4:4, BT.709.
pub fn master_args(frames_dir: &Path, fps: u32, out: &Path) -> Vec<String> {
    let mut args = input_args(frames_dir, fps, 1);
    args.extend(["-vf".into(), color_stage("yuv444p")]);
    args.extend(strs(&["-c:v", "libx264", "-preset", "slow", "-crf", "10", "-pix_fmt", "yuv444p"]));
    args.extend(strs(&COLOR_TAGS));
    args.push(out.display().to_string());
    args
}

const WEB_CODEC: [&str; 8] =
    ["-c:v", "libx264", "-crf", "18", "-pix_fmt", "yuv420p", "-movflags", "+faststart"];

/// ffmpeg arguments for the web copy at native canvas size: H.264, `-crf 18`,
/// 4:2:0, fast start, BT.709.
pub fn web_args(frames_dir: &Path, fps: u32, out: &Path) -> Vec<String> {
    let mut args = input_args(frames_dir, fps, 1);
    args.extend(["-vf".into(), color_stage("yuv420p")]);
    args.extend(strs(&WEB_CODEC));
    args.extend(strs(&COLOR_TAGS));
    args.push(out.display().to_string());
    args
}

/// ffmpeg arguments for a social cut: the frames `repeats` times over, matted
/// on black to `w`×`h`, with the web codec settings.
pub fn cut_args(frames_dir: &Path, fps: u32, repeats: u32, w: u32, h: u32, out: &Path) -> Vec<String> {
    let mut args = input_args(frames_dir, fps, repeats);
    args.extend(["-vf".into(), format!("{},{}", cut_filter(w, h), color_stage("yuv420p"))]);
    args.extend(strs(&WEB_CODEC));
    args.extend(strs(&COLOR_TAGS));
    args.push(out.display().to_string());
    args
}

fn ffmpeg(ws: &Workspace, args: Vec<String>, what: &str) -> Result<()> {
    let output = ws
        .tool("ffmpeg")
        .args(args)
        .output()
        .context("could not run ffmpeg; install it with `brew install ffmpeg`")?;
    if !output.status.success() {
        bail!("ffmpeg could not encode {what}:\n{}", String::from_utf8_lossy(&output.stderr));
    }
    Ok(())
}

fn name(out: &Path) -> String {
    out.file_name().map(|n| n.to_string_lossy().into_owned()).unwrap_or_default()
}

/// Encode the archival master from `frames_dir` to `out`.
pub fn encode_master(ws: &Workspace, frames_dir: &Path, fps: u32, out: &Path) -> Result<()> {
    ffmpeg(ws, master_args(frames_dir, fps, out), &name(out))
}

/// Encode the web copy from `frames_dir` to `out`.
pub fn encode_web(ws: &Workspace, frames_dir: &Path, fps: u32, out: &Path) -> Result<()> {
    ffmpeg(ws, web_args(frames_dir, fps, out), &name(out))
}

/// Encode a `w`×`h` social cut, `repeats` times through the frames, to `out`.
pub fn encode_cut(
    ws: &Workspace,
    frames_dir: &Path,
    fps: u32,
    repeats: u32,
    w: u32,
    h: u32,
    out: &Path,
) -> Result<()> {
    ffmpeg(ws, cut_args(frames_dir, fps, repeats, w, h, out), &name(out))
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

    #[test]
    fn cut_filter_exact() {
        assert_eq!(
            cut_filter(1080, 1350),
            "scale=1080:1350:force_original_aspect_ratio=decrease:flags=lanczos,pad=1080:1350:(ow-iw)/2:(oh-ih)/2:black,setsar=1"
        );
    }

    #[test]
    fn loop_repeats() {
        assert_eq!(super::loop_repeats(8.0, 15.0), 2);
        assert_eq!(super::loop_repeats(4.0, 15.0), 4);
        assert_eq!(super::loop_repeats(20.0, 15.0), 1);
        assert_eq!(super::loop_repeats(15.0, 15.0), 1);
        assert_eq!(super::loop_repeats(1.0, 15.0), 15);
        assert_eq!(super::loop_repeats(0.0, 15.0), 1);
    }

    #[test]
    fn encode_args() {
        let dir = Path::new("frames");
        let master = master_args(dir, 60, Path::new("master.mp4")).join(" ");
        assert!(master.contains("-framerate 60 -start_number 0 -i frames/%05d.png"), "{master}");
        assert!(master.contains("-c:v libx264 -preset slow -crf 10 -pix_fmt yuv444p"), "{master}");
        assert!(master.ends_with("master.mp4"), "{master}");

        let web = web_args(dir, 30, Path::new("loop.mp4")).join(" ");
        assert!(web.contains("-framerate 30"), "{web}");
        assert!(web.contains("-c:v libx264 -crf 18 -pix_fmt yuv420p -movflags +faststart"), "{web}");
        assert!(!web.contains("force_original_aspect_ratio"), "{web}");
        for args in [&master, &web] {
            assert!(args.contains("-colorspace bt709 -color_primaries bt709 -color_trc bt709"), "{args}");
        }
        assert!(master.contains(&format!("-vf {}", color_stage("yuv444p"))), "{master}");
        assert!(web.contains(&format!("-vf {}", color_stage("yuv420p"))), "{web}");
        assert!(color_stage("yuv420p").starts_with("scale=out_color_matrix=bt709:out_range=tv,format=yuv420p,setparams="));

        let cut = cut_args(dir, 60, 2, 1080, 1920, Path::new("reel.mp4"));
        let joined = cut.join(" ");
        assert!(joined.contains("-stream_loop 1 -framerate 60"), "{joined}");
        assert!(joined.contains("-c:v libx264 -crf 18 -pix_fmt yuv420p -movflags +faststart"), "{joined}");
        let vf = cut.iter().position(|a| a == "-vf").unwrap();
        assert_eq!(cut[vf + 1], format!("{},{}", cut_filter(1080, 1920), color_stage("yuv420p")));
        assert!(joined.contains("-colorspace bt709 -color_primaries bt709 -color_trc bt709"), "{joined}");
        let once = cut_args(dir, 60, 1, 1080, 1920, Path::new("reel.mp4")).join(" ");
        assert!(once.contains("-stream_loop 0 "), "{once}");
    }
}
