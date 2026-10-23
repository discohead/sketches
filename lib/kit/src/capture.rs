//! Frame-locked capture driven by `KIT_CAPTURE_DIR` and `KIT_CAPTURE_FRAMES`.
//!
//! Bevy increments `FrameCount` in `Last`, so during update `n` it reads `n`, and nannou's
//! `elapsed_frames()` is `n - 1` (saturating). Updates 0 and 1 therefore both report frame 0.
//! Bevy's first time update has zero delta, so time is held for update 1 as well and then steps
//! by `1/fps`. That keeps `app.time() == elapsed_frames() / fps` on every update from 1 on, and
//! frame `k` is captured on update `k + 1`.
//!
//! On macOS the window's surface can take a few frames to produce its first swapchain texture.
//! Until then the camera renders nothing, yet a screenshot still reads back its (zeroed) target,
//! so a frame captured then is silently black. Capture therefore waits for the render world to
//! acquire the primary window's swapchain once, holding `FrameCount` at 1 (frame 0, time 0)
//! meanwhile, so the warm-up looks to the piece like more repeats of frame 0. Every captured
//! frame is also checked in the render world; one rendered without a swapchain fails the run.
//!
//! A run succeeds only when every requested frame has been written. Any write failure, or a
//! frame still missing 120 updates and 10 s after the last one was requested, fails the run.

use std::ffi::OsString;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use anyhow::{Context, Result, anyhow, bail};
use bevy::app::{App, AppExit, First, Last, Plugin, PostUpdate};
use bevy::diagnostic::{FrameCount, update_frame_count};
use bevy::ecs::prelude::*;
use bevy::log::{error, info};
use bevy::render::extract_resource::{ExtractResource, ExtractResourcePlugin};
use bevy::render::view::ExtractedWindows;
use bevy::render::view::screenshot::{Screenshot, ScreenshotCaptured};
use bevy::render::{Render, RenderApp, RenderSystems};
use bevy::time::{TimeSystems, TimeUpdateStrategy};

pub(crate) const DIR_VAR: &str = "KIT_CAPTURE_DIR";
pub(crate) const FRAMES_VAR: &str = "KIT_CAPTURE_FRAMES";

/// A missing frame fails the run once both of these have passed since the last request. The
/// wall-clock limit also bounds the wait for the window to become ready.
const DEADLINE_UPDATES: u64 = 120;
const DEADLINE_WALL: Duration = Duration::from_secs(10);

static FAILED: AtomicBool = AtomicBool::new(false);

/// Whether the capture run failed. Nannou's `run` drops Bevy's `AppExit` code, so `launch`
/// checks this after `run` returns.
pub(crate) fn failed() -> bool {
    FAILED.load(Ordering::SeqCst)
}

pub(crate) struct CaptureConfig {
    dir: PathBuf,
    frames: Vec<u64>,
}

impl CaptureConfig {
    /// `None` when neither variable is set.
    pub(crate) fn from_env() -> Result<Option<Self>> {
        Self::new(std::env::var_os(DIR_VAR), std::env::var_os(FRAMES_VAR))
    }

    /// Validates the two variables' values. Creates the directory and checks it is writable.
    fn new(dir: Option<OsString>, frames: Option<OsString>) -> Result<Option<Self>> {
        let (dir, frames) = match (dir, frames) {
            (None, None) => return Ok(None),
            (Some(dir), Some(frames)) => (dir, frames),
            _ => bail!("set both {DIR_VAR} and {FRAMES_VAR}, or neither"),
        };
        if dir.is_empty() {
            bail!("{DIR_VAR} is empty");
        }
        let dir = PathBuf::from(dir);
        let frames = frames
            .to_str()
            .with_context(|| format!("{FRAMES_VAR} is not valid UTF-8"))?;
        let frames = parse_frames(frames).with_context(|| format!("bad {FRAMES_VAR}"))?;
        std::fs::create_dir_all(&dir)
            .with_context(|| format!("cannot create {}", dir.display()))?;
        let probe = dir.join(".kit-write-probe");
        std::fs::write(&probe, b"")
            .and_then(|()| std::fs::remove_file(&probe))
            .with_context(|| format!("{} is not writable", dir.display()))?;
        Ok(Some(CaptureConfig { dir, frames }))
    }
}

/// Parses `a..b` (half-open) or a comma list `0,30,60`. Sorted, deduplicated, never empty.
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

#[derive(Debug, PartialEq, Eq)]
enum Verdict {
    Continue,
    Success,
    Failed(String),
}

#[derive(Default)]
struct Progress {
    saved: usize,
    failed: usize,
}

/// `updates_past_last` and `wall_past_last` are `None` until the last frame is requested.
fn verdict(
    progress: &Progress,
    requested: usize,
    updates_past_last: Option<u64>,
    wall_past_last: Option<Duration>,
) -> Verdict {
    if progress.failed > 0 {
        return Verdict::Failed(format!(
            "{} of {requested} frames failed to save",
            progress.failed
        ));
    }
    if progress.saved >= requested {
        return Verdict::Success;
    }
    match (updates_past_last, wall_past_last) {
        (Some(updates), Some(wall)) if updates >= DEADLINE_UPDATES && wall >= DEADLINE_WALL => {
            Verdict::Failed(format!(
                "timed out: saved {} of {requested} frames",
                progress.saved
            ))
        }
        _ => Verdict::Continue,
    }
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
    progress: Progress,
    /// Set on the first update at which the window has rendered; frame 0 is captured then.
    started: bool,
    warmup_since: Option<Instant>,
    last_requested_at: Option<Instant>,
    done: bool,
}

/// Shared by the main and render worlds.
#[derive(Resource, Clone, Default)]
struct WindowStatus(Arc<WindowStatusInner>);

#[derive(Default)]
struct WindowStatusInner {
    /// The primary window has had a swapchain texture at least once.
    ready: AtomicBool,
    /// Captured frames that were rendered without a swapchain texture.
    unrendered: Mutex<Vec<u64>>,
}

/// The frame whose screenshot was requested this update, if any. Extracted to the render world.
#[derive(Resource, Clone, Default, ExtractResource)]
struct CaptureRequest(Option<u64>);

impl Plugin for CapturePlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(TimeUpdateStrategy::ManualDuration(Duration::ZERO))
            .insert_resource(Capture {
                dir: self.dir.clone(),
                frames: self.frames.clone(),
                dt: self.dt,
                progress: Progress::default(),
                started: false,
                warmup_since: None,
                last_requested_at: None,
                done: false,
            })
            .insert_resource(WindowStatus::default())
            .init_resource::<CaptureRequest>()
            .add_plugins(ExtractResourcePlugin::<CaptureRequest>::default())
            .add_systems(First, lock_time.before(TimeSystems))
            .add_systems(PostUpdate, capture_frame)
            .add_systems(Last, hold_frame_count.after(update_frame_count));

        let status = app.world().resource::<WindowStatus>().clone();
        app.sub_app_mut(RenderApp)
            .insert_resource(status)
            .add_systems(
                Render,
                check_window
                    .after(RenderSystems::PrepareViews)
                    .before(RenderSystems::Render),
            );
    }
}

fn lock_time(
    capture: Res<Capture>,
    frame_count: Res<FrameCount>,
    mut strategy: ResMut<TimeUpdateStrategy>,
) {
    *strategy = TimeUpdateStrategy::ManualDuration(time_step(frame_count.0, capture.dt));
}

/// Until capture starts, keeps every update at `FrameCount` 1: frame 0, time 0.
fn hold_frame_count(capture: Res<Capture>, mut frame_count: ResMut<FrameCount>) {
    if !capture.started && frame_count.0 > 1 {
        frame_count.0 = 1;
    }
}

/// Render world: records whether the primary window can be rendered to this frame.
fn check_window(
    windows: Res<ExtractedWindows>,
    request: Res<CaptureRequest>,
    status: Res<WindowStatus>,
) {
    let renderable = windows
        .primary
        .and_then(|entity| windows.windows.get(&entity))
        .is_some_and(|window| window.swap_chain_texture_view.is_some());
    if renderable {
        status.0.ready.store(true, Ordering::SeqCst);
    } else if let Some(frame) = request.0 {
        status.0.unrendered.lock().unwrap().push(frame);
    }
}

fn capture_frame(
    mut commands: Commands,
    mut capture: ResMut<Capture>,
    mut request: ResMut<CaptureRequest>,
    status: Res<WindowStatus>,
    frame_count: Res<FrameCount>,
    mut exit: MessageWriter<AppExit>,
) {
    request.0 = None;
    if capture.done {
        return;
    }
    let Some(frame) = captured_frame(frame_count.0) else {
        return;
    };
    if !capture.started {
        if status.0.ready.load(Ordering::SeqCst) {
            capture.started = true;
            if let Some(since) = capture.warmup_since {
                info!(
                    "kit capture: waited {:?} for the window to render",
                    since.elapsed()
                );
            }
        } else {
            let since = *capture.warmup_since.get_or_insert_with(Instant::now);
            if since.elapsed() >= DEADLINE_WALL {
                error!("kit capture failed: the window never became ready to render");
                FAILED.store(true, Ordering::SeqCst);
                capture.done = true;
                exit.write(AppExit::error());
            }
            return;
        }
    }
    for unrendered in std::mem::take(&mut *status.0.unrendered.lock().unwrap()) {
        error!("kit capture: frame {unrendered} was rendered before the window had a surface");
        capture.progress.failed += 1;
    }
    let last = *capture
        .frames
        .last()
        .expect("parse_frames never returns an empty list");
    if capture.frames.binary_search(&frame).is_ok() {
        let path = capture.dir.join(format!("{frame:05}.png"));
        commands
            .spawn(Screenshot::primary_window())
            .observe(save_frame(path));
        request.0 = Some(frame);
        if frame == last {
            capture.last_requested_at = Some(Instant::now());
        }
    }

    let wall_past_last = capture.last_requested_at.map(|t| t.elapsed());
    match verdict(
        &capture.progress,
        capture.frames.len(),
        frame.checked_sub(last),
        wall_past_last,
    ) {
        Verdict::Continue => {}
        Verdict::Success => {
            info!(
                "kit capture: saved {} frames to {}",
                capture.frames.len(),
                capture.dir.display()
            );
            capture.done = true;
            exit.write(AppExit::Success);
        }
        Verdict::Failed(reason) => {
            error!("kit capture failed: {reason}");
            FAILED.store(true, Ordering::SeqCst);
            capture.done = true;
            exit.write(AppExit::error());
        }
    }
}

/// Writes one captured frame as PNG and records the outcome in [`Capture`].
fn save_frame(path: PathBuf) -> impl FnMut(On<ScreenshotCaptured>, ResMut<Capture>) {
    move |captured, mut capture| {
        let result = captured
            .image
            .clone()
            .try_into_dynamic()
            .map_err(|e| anyhow!("cannot convert screenshot: {e:?}"))
            .and_then(|img| {
                img.to_rgb8()
                    .save_with_format(&path, image::ImageFormat::Png)
                    .with_context(|| format!("cannot write {}", path.display()))
            });
        match result {
            Ok(()) => capture.progress.saved += 1,
            Err(e) => {
                error!("kit capture: {e:#}");
                capture.progress.failed += 1;
            }
        }
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

    fn config(
        dir: Option<&std::path::Path>,
        frames: Option<&str>,
    ) -> Result<Option<CaptureConfig>> {
        CaptureConfig::new(dir.map(Into::into), frames.map(Into::into))
    }

    #[test]
    fn config_absent_is_none() {
        assert!(config(None, None).unwrap().is_none());
    }

    #[test]
    fn config_rejects_half_set_empty_and_unwritable() {
        let tmp = tempfile::tempdir().unwrap();
        assert!(config(Some(tmp.path()), None).is_err());
        assert!(config(None, Some("0..3")).is_err());
        let err = config(Some(std::path::Path::new("")), Some("0..3"))
            .err()
            .unwrap();
        assert!(format!("{err:#}").contains("empty"), "{err:#}");

        use std::os::unix::fs::PermissionsExt;
        let ro = tmp.path().join("ro");
        std::fs::create_dir(&ro).unwrap();
        std::fs::set_permissions(&ro, std::fs::Permissions::from_mode(0o555)).unwrap();
        let err = config(Some(&ro), Some("0..3")).err().unwrap();
        assert!(format!("{err:#}").contains("not writable"), "{err:#}");
        assert_eq!(std::fs::read_dir(&ro).unwrap().count(), 0);
    }

    #[test]
    fn config_creates_dir() {
        let tmp = tempfile::tempdir().unwrap();
        let dir = tmp.path().join("a/b");
        let c = config(Some(&dir), Some("2,1")).unwrap().unwrap();
        assert_eq!(c.frames, vec![1, 2]);
        assert!(dir.is_dir());
        assert_eq!(std::fs::read_dir(&dir).unwrap().count(), 0);
    }

    #[test]
    fn verdict_rules() {
        let p = |saved, failed| Progress { saved, failed };
        let s = Duration::from_secs(1);
        // Still capturing or waiting on readbacks.
        assert_eq!(verdict(&p(0, 0), 3, None, None), Verdict::Continue);
        assert_eq!(verdict(&p(2, 0), 3, Some(5), Some(s)), Verdict::Continue);
        // All saved.
        assert_eq!(verdict(&p(3, 0), 3, Some(1), Some(s)), Verdict::Success);
        // Any write failure fails at once.
        assert!(matches!(
            verdict(&p(1, 1), 3, None, None),
            Verdict::Failed(_)
        ));
        // Deadline needs both 120 updates and 10 s past the last frame.
        assert_eq!(verdict(&p(2, 0), 3, Some(120), Some(s)), Verdict::Continue);
        assert_eq!(
            verdict(&p(2, 0), 3, Some(5), Some(s * 10)),
            Verdict::Continue
        );
        assert!(matches!(
            verdict(&p(2, 0), 3, Some(120), Some(s * 10)),
            Verdict::Failed(_)
        ));
    }
}
