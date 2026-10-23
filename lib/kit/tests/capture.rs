//! Runs the `capture_probe` example in capture mode. Opens a real window, so it is ignored by
//! default: `scripts/cargo test -p kit --test capture -- --ignored`.

use std::path::Path;
use std::process::{Command, ExitStatus, Stdio};
use std::sync::Mutex;
use std::time::{Duration, Instant};

const FRAMES: usize = 60;
const WIDTH: u32 = 540;
const HEIGHT: u32 = 360;
const TIMEOUT: Duration = Duration::from_secs(120);

/// rustup's cargo, with its proxies first on PATH (Homebrew's cargo cannot build Bevy 0.19).
fn cargo() -> Command {
    let home = std::env::var("HOME").expect("HOME");
    let bin = Path::new(&home).join(".cargo/bin");
    let path = std::env::var_os("PATH").unwrap_or_default();
    let path =
        std::env::join_paths(std::iter::once(bin.clone()).chain(std::env::split_paths(&path)))
            .unwrap();
    let mut cmd = Command::new(bin.join("cargo"));
    cmd.env("PATH", path)
        .current_dir(env!("CARGO_MANIFEST_DIR"));
    cmd
}

fn luma(path: &Path) -> image::GrayImage {
    image::open(path)
        .unwrap_or_else(|e| panic!("{}: {e}", path.display()))
        .to_luma8()
}

fn mean_abs_diff(a: &image::GrayImage, b: &image::GrayImage) -> f64 {
    let sum: u64 = a
        .as_raw()
        .iter()
        .zip(b.as_raw())
        .map(|(x, y)| x.abs_diff(*y) as u64)
        .sum();
    sum as f64 / a.as_raw().len() as f64
}

/// Builds the probe once, then runs it in capture mode and returns its exit status. Runs are
/// serialized so only one window is open at a time.
fn run_probe(dir: &Path, frames: &str) -> ExitStatus {
    static LOCK: Mutex<bool> = Mutex::new(false);
    let mut built = LOCK.lock().unwrap_or_else(|e| e.into_inner());
    if !*built {
        let status = cargo()
            .args(["build", "-p", "kit", "--example", "capture_probe"])
            .status()
            .expect("cargo build");
        assert!(status.success(), "building capture_probe failed");
        *built = true;
    }

    let mut child = cargo()
        .args(["run", "-q", "-p", "kit", "--example", "capture_probe"])
        .env("KIT_CAPTURE_DIR", dir)
        .env("KIT_CAPTURE_FRAMES", frames)
        .stdin(Stdio::null())
        .spawn()
        .expect("cargo run");
    let start = Instant::now();
    loop {
        if let Some(status) = child.try_wait().unwrap() {
            return status;
        }
        if start.elapsed() > TIMEOUT {
            child.kill().ok();
            child.wait().ok();
            panic!("capture_probe did not exit within {TIMEOUT:?}");
        }
        std::thread::sleep(Duration::from_millis(100));
    }
}

fn file_names(dir: &Path) -> Vec<String> {
    let mut names: Vec<String> = std::fs::read_dir(dir)
        .unwrap()
        .map(|e| e.unwrap().file_name().into_string().unwrap())
        .collect();
    names.sort();
    names
}

/// Centre of the bright pixels, in the probe's coordinates: origin at the canvas centre, y up.
fn disc_centre(img: &image::GrayImage) -> (f64, f64) {
    let (w, h) = img.dimensions();
    let (mut sx, mut sy, mut n) = (0.0, 0.0, 0.0);
    for (x, y, p) in img.enumerate_pixels() {
        if p.0[0] > 128 {
            sx += x as f64;
            sy += y as f64;
            n += 1.0;
        }
    }
    assert!(n > 0.0, "no disc in frame");
    (
        sx / n + 0.5 - w as f64 / 2.0,
        h as f64 / 2.0 - (sy / n + 0.5),
    )
}

fn assert_near(actual: (f64, f64), expected: (f64, f64), what: &str) {
    let d = ((actual.0 - expected.0).powi(2) + (actual.1 - expected.1).powi(2)).sqrt();
    assert!(
        d <= 2.0,
        "{what}: disc centre {actual:?}, expected {expected:?}"
    );
}

#[test]
#[ignore = "opens a window"]
fn probe_capture_is_frame_locked_and_exact_size() {
    let dir = tempfile::tempdir().unwrap();
    let status = run_probe(dir.path(), &format!("0..{FRAMES}"));
    assert!(status.success(), "capture_probe exited with {status}");

    let expected: Vec<String> = (0..FRAMES).map(|k| format!("{k:05}.png")).collect();
    assert_eq!(file_names(dir.path()), expected);

    let frames: Vec<image::GrayImage> =
        expected.iter().map(|n| luma(&dir.path().join(n))).collect();
    for (name, img) in expected.iter().zip(&frames) {
        assert_eq!(img.dimensions(), (WIDTH, HEIGHT), "{name} size");
    }

    // Absolute alignment: frame k is at phase k / 60 on a 140 px orbit.
    assert_near(disc_centre(&frames[0]), (140.0, 0.0), "frame 0");
    assert_near(disc_centre(&frames[15]), (0.0, 140.0), "frame 15");

    let steps: Vec<f64> = frames
        .windows(2)
        .map(|w| mean_abs_diff(&w[0], &w[1]))
        .collect();
    for (k, d) in steps.iter().enumerate() {
        assert!(
            *d > 0.0,
            "frames {k} and {} are identical (doubled frame)",
            k + 1
        );
    }
    let mean = steps.iter().sum::<f64>() / steps.len() as f64;
    let seam = mean_abs_diff(&frames[FRAMES - 1], &frames[0]);
    eprintln!(
        "seam d(59->0) = {seam:.4}, mean d(k->k+1) = {mean:.4}, ratio = {:.4}",
        seam / mean
    );
    eprintln!("steps: {steps:.3?}");
    assert!(
        (seam - mean).abs() <= 0.25 * mean,
        "seam d(59->0) = {seam} is not within 25% of mean step {mean}"
    );
}

#[test]
#[ignore = "opens a window"]
fn unwritable_dir_fails_before_capture() {
    use std::os::unix::fs::PermissionsExt;
    let tmp = tempfile::tempdir().unwrap();
    let dir = tmp.path().join("ro");
    std::fs::create_dir(&dir).unwrap();
    std::fs::set_permissions(&dir, std::fs::Permissions::from_mode(0o555)).unwrap();

    let status = run_probe(&dir, "0..3");
    std::fs::set_permissions(&dir, std::fs::Permissions::from_mode(0o755)).unwrap();
    assert!(!status.success(), "expected a non-zero exit, got {status}");
    assert_eq!(file_names(&dir), Vec::<String>::new());
}

#[test]
#[ignore = "opens a window"]
fn frame_write_failure_exits_nonzero() {
    // The directory is writable, so the up-front check passes, but frame 1's path is taken by
    // a directory, so its PNG cannot be written.
    let dir = tempfile::tempdir().unwrap();
    std::fs::create_dir(dir.path().join("00001.png")).unwrap();

    let status = run_probe(dir.path(), "0..3");
    assert!(!status.success(), "expected a non-zero exit, got {status}");
}
