//! Runs the `capture_probe` example in capture mode. Opens a real window, so it is ignored by
//! default: `scripts/cargo test -p kit --test capture -- --ignored`.

use std::path::Path;
use std::process::{Command, Stdio};
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

#[test]
#[ignore = "opens a window"]
fn probe_capture_is_frame_locked_and_exact_size() {
    let status = cargo()
        .args(["build", "-p", "kit", "--example", "capture_probe"])
        .status()
        .expect("cargo build");
    assert!(status.success(), "building capture_probe failed");

    let dir = tempfile::tempdir().unwrap();
    let mut child = cargo()
        .args(["run", "-q", "-p", "kit", "--example", "capture_probe"])
        .env("KIT_CAPTURE_DIR", dir.path())
        .env("KIT_CAPTURE_FRAMES", format!("0..{FRAMES}"))
        .stdin(Stdio::null())
        .spawn()
        .expect("cargo run");
    let start = Instant::now();
    let status = loop {
        if let Some(status) = child.try_wait().unwrap() {
            break status;
        }
        if start.elapsed() > TIMEOUT {
            child.kill().ok();
            child.wait().ok();
            panic!("capture_probe did not exit within {TIMEOUT:?}");
        }
        std::thread::sleep(Duration::from_millis(100));
    };
    assert!(status.success(), "capture_probe exited with {status}");

    let mut names: Vec<String> = std::fs::read_dir(dir.path())
        .unwrap()
        .map(|e| e.unwrap().file_name().into_string().unwrap())
        .collect();
    names.sort();
    let expected: Vec<String> = (0..FRAMES).map(|k| format!("{k:05}.png")).collect();
    assert_eq!(names, expected);

    let frames: Vec<image::GrayImage> =
        expected.iter().map(|n| luma(&dir.path().join(n))).collect();
    for (name, img) in expected.iter().zip(&frames) {
        assert_eq!(img.dimensions(), (WIDTH, HEIGHT), "{name} size");
    }

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
