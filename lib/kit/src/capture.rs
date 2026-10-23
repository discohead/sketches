//! Frame-locked capture driven by `KIT_CAPTURE_DIR` and `KIT_CAPTURE_FRAMES`.
//!
//! Bevy increments `FrameCount` in `Last`, so during update `n` it reads `n`, and nannou's
//! `elapsed_frames()` is `n - 1` (saturating). Updates 0 and 1 therefore both report frame 0.
//! Bevy's first time update has zero delta, so time is held for update 1 as well and then steps
//! by `1/fps`. That keeps `app.time() == elapsed_frames() / fps` on every update from 1 on, and
//! frame `k` is captured on update `k + 1`.

use std::path::PathBuf;
use std::time::Duration;

use anyhow::{Context, Result, bail};
use bevy::app::{App, AppExit, First, Plugin, PostUpdate};
use bevy::diagnostic::FrameCount;
use bevy::ecs::prelude::*;
use bevy::render::view::screenshot::{Screenshot, save_to_disk};
use bevy::time::{TimeSystems, TimeUpdateStrategy};

pub(crate) const DIR_VAR: &str = "KIT_CAPTURE_DIR";
pub(crate) const FRAMES_VAR: &str = "KIT_CAPTURE_FRAMES";

/// Updates to keep running after the last capture, so pending screenshots are written.
const EXIT_AFTER: u64 = 10;

pub(crate) struct CaptureConfig {
    dir: PathBuf,
    frames: Vec<u64>,
}

impl CaptureConfig {
    /// `None` when neither variable is set. Creates the directory.
    pub(crate) fn from_env() -> Result<Option<Self>> {
        let dir = std::env::var_os(DIR_VAR);
        let frames = std::env::var_os(FRAMES_VAR);
        let (dir, frames) = match (dir, frames) {
            (None, None) => return Ok(None),
            (Some(dir), Some(frames)) => (PathBuf::from(dir), frames),
            _ => bail!("set both {DIR_VAR} and {FRAMES_VAR}, or neither"),
        };
        let frames = frames
            .to_str()
            .with_context(|| format!("{FRAMES_VAR} is not valid UTF-8"))?;
        let frames = parse_frames(frames).with_context(|| format!("bad {FRAMES_VAR}"))?;
        std::fs::create_dir_all(&dir)
            .with_context(|| format!("cannot create {}", dir.display()))?;
        Ok(Some(CaptureConfig { dir, frames }))
    }
}

/// Parses `a..b` (half-open) or a comma list `0,30,60`. Sorted, deduplicated.
pub(crate) fn parse_frames(spec: &str) -> Result<Vec<u64>> {
    let spec = spec.trim();
    let mut frames = if let Some((a, b)) = spec.split_once("..") {
        let a: u64 = a
            .trim()
            .parse()
            .with_context(|| format!("bad range start {a:?}"))?;
        let b: u64 = b
            .trim()
            .parse()
            .with_context(|| format!("bad range end {b:?}"))?;
        if a >= b {
            bail!("empty range {spec:?}");
        }
        (a..b).collect::<Vec<_>>()
    } else {
        spec.split(',')
            .map(|s| {
                let s = s.trim();
                s.parse::<u64>().with_context(|| format!("bad frame {s:?}"))
            })
            .collect::<Result<Vec<_>>>()?
    };
    frames.sort_unstable();
    frames.dedup();
    Ok(frames)
}

/// The time step for the update during which `FrameCount` reads `frame_count`.
fn time_step(frame_count: u32, dt: Duration) -> Duration {
    if frame_count <= 1 { Duration::ZERO } else { dt }
}

/// `elapsed_frames()` for the update during which `FrameCount` reads `frame_count`, or `None`
/// for update 0, which repeats frame 0 at the same time as update 1.
fn captured_frame(frame_count: u32) -> Option<u64> {
    (frame_count as u64).checked_sub(1)
}

pub(crate) struct CapturePlugin {
    dir: PathBuf,
    frames: Vec<u64>,
    dt: Duration,
}

impl CapturePlugin {
    pub(crate) fn new(config: CaptureConfig, fps: u32) -> Self {
        assert!(fps > 0, "fps must be positive");
        CapturePlugin {
            dir: config.dir,
            frames: config.frames,
            dt: Duration::from_secs_f64(1.0 / fps as f64),
        }
    }
}

#[derive(Resource)]
struct Capture {
    dir: PathBuf,
    frames: Vec<u64>,
    dt: Duration,
}

impl Plugin for CapturePlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(TimeUpdateStrategy::ManualDuration(Duration::ZERO))
            .insert_resource(Capture {
                dir: self.dir.clone(),
                frames: self.frames.clone(),
                dt: self.dt,
            })
            .add_systems(First, lock_time.before(TimeSystems))
            .add_systems(PostUpdate, capture_frame);
    }
}

fn lock_time(
    capture: Res<Capture>,
    frame_count: Res<FrameCount>,
    mut strategy: ResMut<TimeUpdateStrategy>,
) {
    *strategy = TimeUpdateStrategy::ManualDuration(time_step(frame_count.0, capture.dt));
}

fn capture_frame(
    mut commands: Commands,
    capture: Res<Capture>,
    frame_count: Res<FrameCount>,
    mut exit: MessageWriter<AppExit>,
) {
    let Some(frame) = captured_frame(frame_count.0) else {
        return;
    };
    if capture.frames.binary_search(&frame).is_ok() {
        let path = capture.dir.join(format!("{frame:05}.png"));
        commands
            .spawn(Screenshot::primary_window())
            .observe(save_to_disk(path));
    }
    let last = capture.frames.last().copied().unwrap_or(0);
    if frame >= last + EXIT_AFTER {
        exit.write(AppExit::Success);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_frames_range() {
        assert_eq!(parse_frames("0..4").unwrap(), vec![0, 1, 2, 3]);
    }

    #[test]
    fn parse_frames_list() {
        assert_eq!(parse_frames("30,0,30").unwrap(), vec![0, 30]);
    }

    #[test]
    fn parse_frames_rejects() {
        for bad in ["", "a..b", "5..2"] {
            assert!(parse_frames(bad).is_err(), "{bad:?} should be rejected");
        }
    }

    #[test]
    fn time_matches_elapsed_frames() {
        let dt = Duration::from_millis(10);
        let mut t = Duration::ZERO;
        // Update 0 is Bevy's first time update: zero delta whatever the strategy.
        for n in 1..100u32 {
            t += time_step(n, dt);
            assert_eq!(t, dt * captured_frame(n).unwrap() as u32, "update {n}");
        }
        assert_eq!(captured_frame(0), None);
    }
}
