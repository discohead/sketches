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

/// When a capture run counts as stuck. Rendering speed varies a lot (a heavy
/// shader, a long piece), so a run is stopped when frames stop arriving, not
/// when it has taken long. The total cap is a backstop for a piece that crawls.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CaptureLimits {
    /// From launch (after the build) to the first frame: startup and warm-up.
    pub first_frame: Duration,
    /// From one saved frame to the next: `max(60 s, 20 frame intervals)`.
    pub between_frames: Duration,
    /// The whole run: `60 s + 30 s × (max_frame + 1) / fps`, rounded up.
    pub total: Duration,
}

impl CaptureLimits {
    pub fn for_frames(max_frame: u64, fps: u32) -> CaptureLimits {
        let fps = u64::from(fps.max(1));
        let twenty_frames = Duration::from_secs_f64(20.0 / fps as f64);
        CaptureLimits {
            first_frame: Duration::from_secs(60),
            between_frames: twenty_frames.max(Duration::from_secs(60)),
            total: Duration::from_secs(60 + (30 * (max_frame + 1)).div_ceil(fps)),
        }
    }

    /// Why the run should be stopped, given the time since it started, the time
    /// since the last new frame (or since the start), and the frames saved so far.
    /// `None` means let it run.
    pub fn exceeded(&self, elapsed: Duration, since_progress: Duration, done: usize) -> Option<String> {
        if elapsed > self.total {
            return Some(format!("it ran past the {} s cap for this piece", self.total.as_secs()));
        }
        if done == 0 && since_progress > self.first_frame {
            return Some(format!(
                "no frame was saved in the first {} s (startup and warm-up)",
                self.first_frame.as_secs()
            ));
        }
        if done > 0 && since_progress > self.between_frames {
            return Some(format!(
                "no new frame for {} s after {done} frames",
                self.between_frames.as_secs()
            ));
        }
        None
    }
}

/// `KIT_CAPTURE_FRAMES` for `frames` (sorted): a half-open range `a..b` when
/// they are contiguous, otherwise a comma list. A long piece is thousands of
/// frames, too much to pass as a list in the environment.
pub fn frame_spec(frames: &[u64]) -> String {
    match frames {
        [first, .., last] if frames.windows(2).all(|w| w[1] == w[0] + 1) => {
            format!("{first}..{}", last + 1)
        }
        _ => frames.iter().map(u64::to_string).collect::<Vec<_>>().join(","),
    }
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

/// The error for a frame whose size is not the canvas size.
fn size_mismatch(k: u64, (pw, ph): (u32, u32), (w, h): (u32, u32)) -> String {
    format!(
        "frame {k} is {pw}x{ph} but the canvas is {w}x{h}. The likely cause is a \
         canvas that does not fit on the display at exact pixels: the window is \
         canvas-sized, and macOS limits a window to the screen. \
         Use a smaller canvas (the default 1080x1080 fits), or check the screen \
         with `system_profiler SPDisplaysDataType`."
    )
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
    let spec = frame_spec(frames);
    let stderr_file = out.join("stderr.log");
    let mut child = capture_command(&bin, piece, out, &spec)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(std::fs::File::create(&stderr_file)?)
        .spawn()
        .with_context(|| format!("cannot start {}", bin.display()))?;

    // Frames are saved in order, so progress is the next expected frame
    // appearing on disk.
    let limits = CaptureLimits::for_frames(max_frame, piece.meta.time.fps);
    let start = Instant::now();
    let mut last_progress = start;
    let mut done = 0;
    let status = loop {
        if let Some(s) = child.try_wait()? {
            break s;
        }
        while done < frames.len() && frame_path(out, frames[done]).is_file() {
            done += 1;
            last_progress = Instant::now();
        }
        if let Some(why) = limits.exceeded(start.elapsed(), last_progress.elapsed(), done) {
            let _ = child.kill();
            let _ = child.wait();
            bail!(
                "{} was stopped after saving {done} of {} frames: {why}. \
                 Its update loop may be stuck.",
                piece.folder,
                frames.len()
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
            bail!("{}", size_mismatch(k, (pw, ph), (w, h)));
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

    const S: fn(u64) -> Duration = Duration::from_secs;

    #[test]
    fn limits_for_an_eight_second_loop() {
        let l = CaptureLimits::for_frames(479, 60);
        assert_eq!(l.first_frame, S(60));
        assert_eq!(l.between_frames, S(60));
        assert_eq!(l.total, S(60 + 240)); // 60 s + 30 s × 8 s
    }

    #[test]
    fn limits_cap_scales_with_piece_length() {
        assert_eq!(CaptureLimits::for_frames(59, 30).total, S(60 + 60));
        assert_eq!(CaptureLimits::for_frames(3599, 60).total, S(60 + 1800));
        // A fraction of a second rounds up.
        assert_eq!(CaptureLimits::for_frames(0, 60).total, S(61));
    }

    #[test]
    fn limits_never_wait_less_than_a_minute_between_frames() {
        for fps in [1, 24, 60, 120] {
            assert_eq!(CaptureLimits::for_frames(10, fps).between_frames, S(60), "{fps}");
        }
    }

    #[test]
    fn slow_but_steady_long_piece_is_never_stopped() {
        // A 60 s piece at 60 fps that renders at only 2.5 fps: 1440 s in all,
        // far past the old total-time limit, with a frame every 0.4 s.
        let l = CaptureLimits::for_frames(3599, 60);
        let startup = Duration::from_secs(45);
        let step = Duration::from_millis(400);
        assert_eq!(l.exceeded(startup, startup, 0), None);
        for done in 1..=3600u32 {
            let elapsed = startup + step * done;
            assert_eq!(l.exceeded(elapsed, Duration::ZERO, done as usize), None);
            assert_eq!(l.exceeded(elapsed + step, step, done as usize), None);
        }
    }

    #[test]
    fn stopped_when_no_first_frame_within_a_minute() {
        let l = CaptureLimits::for_frames(479, 60);
        assert_eq!(l.exceeded(S(59), S(59), 0), None);
        let why = l.exceeded(S(61), S(61), 0).unwrap();
        assert!(why.contains("no frame") && why.contains("60 s"), "{why}");
    }

    #[test]
    fn stopped_when_frames_stop_arriving() {
        let l = CaptureLimits::for_frames(479, 60);
        assert_eq!(l.exceeded(S(200), S(59), 100), None);
        let why = l.exceeded(S(200), S(61), 100).unwrap();
        assert!(why.contains("no new frame for 60 s"), "{why}");
    }

    #[test]
    fn stopped_at_the_cap_even_while_frames_arrive() {
        let l = CaptureLimits::for_frames(479, 60);
        let why = l.exceeded(S(301), S(0), 470).unwrap();
        assert!(why.contains("300 s"), "{why}");
    }

    #[test]
    fn frame_spec_uses_a_range_when_contiguous() {
        assert_eq!(frame_spec(&[0, 1, 2, 3]), "0..4");
        assert_eq!(frame_spec(&(0..3600).collect::<Vec<_>>()), "0..3600");
        assert_eq!(frame_spec(&[5]), "5");
        assert_eq!(frame_spec(&[0, 30, 60]), "0,30,60");
        assert_eq!(frame_spec(&[0, 1, 3]), "0,1,3");
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
    fn size_mismatch_names_the_display_as_a_likely_cause() {
        let m = size_mismatch(3, (540, 960), (1080, 1920));
        assert!(m.starts_with("frame 3 is 540x960 but the canvas is 1080x1920"), "{m}");
        assert!(m.contains("does not fit on the display"), "{m}");
        assert!(m.contains("1080x1080"), "{m}");
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
