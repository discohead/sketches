//! End to end: hang a piece in a throwaway clone, fail the push, fix the remote,
//! and resume. Opens a capture window and encodes video, so it is ignored by default:
//! `scripts/cargo test -p studio --test hang_e2e -- --ignored`

use std::fs;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::process::Command;

use piece_meta::PieceMeta;
use studio::workspace::Workspace;
use studio::{hang, new};

/// Removes the clone when the test ends, pass or fail.
struct RemoveOnDrop(PathBuf);

impl Drop for RemoveOnDrop {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

fn run(dir: &Path, program: &str, args: &[&str]) -> String {
    let out = Command::new(program).current_dir(dir).args(args).output().unwrap();
    assert!(
        out.status.success(),
        "{program} {args:?} failed: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    String::from_utf8_lossy(&out.stdout).trim().to_string()
}

fn git(dir: &Path, args: &[&str]) -> String {
    run(dir, "git", args)
}

fn png_size(path: &Path) -> (u32, u32) {
    let mut head = [0u8; 24];
    fs::File::open(path).unwrap().read_exact(&mut head).unwrap();
    assert_eq!(&head[..8], b"\x89PNG\r\n\x1a\n", "{} is not a PNG", path.display());
    (
        u32::from_be_bytes(head[16..20].try_into().unwrap()),
        u32::from_be_bytes(head[20..24].try_into().unwrap()),
    )
}

#[test]
#[ignore = "opens a capture window and encodes video; run once per round"]
fn hang_resumes_after_push_failure() {
    let main_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..").canonicalize().unwrap();
    // A sibling of the real repo, so `../nannou-sketches` still resolves.
    let clone = main_root
        .parent()
        .unwrap()
        .join(format!(".sketches-e2e-{}", std::process::id()));
    let _cleanup = RemoveOnDrop(clone.clone());
    let remotes = tempfile::tempdir().unwrap();
    // Build into the main target dir so nothing builds cold, and accept the local
    // bare repo as origin. Only this test runs in this binary, so changing the
    // environment is safe.
    unsafe {
        std::env::set_var("CARGO_TARGET_DIR", main_root.join("target"));
        std::env::set_var("STUDIO_ALLOW_ANY_ORIGIN", "1");
    }

    // 1. Clone, and point origin away from the real repo before anything else.
    run(
        main_root.parent().unwrap(),
        "git",
        &["clone", "-q", main_root.to_str().unwrap(), clone.to_str().unwrap()],
    );
    let missing = remotes.path().join("does-not-exist.git");
    git(&clone, &["remote", "set-url", "origin", missing.to_str().unwrap()]);
    assert_eq!(git(&clone, &["remote", "get-url", "origin"]), missing.to_str().unwrap());
    git(&clone, &["config", "user.name", "Studio E2E"]);
    git(&clone, &["config", "user.email", "e2e@example.com"]);
    git(&clone, &["config", "commit.gpgsign", "false"]);
    // hang pushes `main` only; the clone starts on whatever branch the real repo
    // has checked out, so put that commit on a local `main`.
    git(&clone, &["checkout", "-q", "-B", "main"]);
    let branch = git(&clone, &["branch", "--show-current"]);
    assert_eq!(branch, "main");

    // 2. A one-second smoke piece, ready to hang.
    let ws = Workspace::discover(&clone).unwrap();
    let piece = new::create(&ws, "smoke", None, "2026-10-22").unwrap();
    let mut meta = piece.meta.clone();
    meta.title = "Smoke".into();
    meta.statement = "A one-second test loop.".into();
    meta.time.seconds = 1.0;
    meta.time.still_at = 0.5;
    fs::write(piece.dir.join("piece.toml"), meta.to_toml_string()).unwrap();

    // 3. The push fails; everything before it is done.
    let err = hang::run(&ws, "smoke", hang::HangOpts { push: true }).unwrap_err().to_string();
    assert!(err.contains("push failed"), "{err}");
    assert!(err.contains("studio hang DSS0001"), "{err}");
    assert!(clone.join("pieces/dss0001-smoke").is_dir());
    assert!(!clone.join("pieces/2026-10-22-smoke").exists());
    assert_eq!(git(&clone, &["log", "-1", "--format=%s"]), "hang: DSS0001 Smoke");
    assert_eq!(
        git(&clone, &["status", "--porcelain", "--", "pieces", "Cargo.lock"]),
        "",
        "the hang commit left changes behind"
    );
    let lock = fs::read_to_string(clone.join("Cargo.lock")).unwrap();
    assert!(lock.contains("name = \"dss0001-smoke\""));
    assert!(!lock.contains("wip-smoke"));
    let outbox = clone.join("outbox");
    assert!(outbox.join(".DSS0001.partial").is_dir());
    assert!(!outbox.join("DSS0001").exists());

    // 4. Fix the remote and run again: same number, deposit appears.
    let bare = remotes.path().join("origin.git");
    git(remotes.path(), &["init", "-q", "--bare", "-b", &branch, bare.to_str().unwrap()]);
    git(&clone, &["remote", "set-url", "origin", bare.to_str().unwrap()]);
    let dir = hang::run(&ws, "DSS0001", hang::HangOpts { push: true }).unwrap();
    assert_eq!(dir, outbox.join("DSS0001"));
    assert!(!outbox.join("DSS0002").exists());
    assert!(!outbox.join(".DSS0001.partial").exists());
    let hung: Vec<_> = fs::read_dir(clone.join("pieces"))
        .unwrap()
        .map(|e| e.unwrap().file_name().to_string_lossy().into_owned())
        .filter(|n| n.starts_with("dss"))
        .collect();
    assert_eq!(hung, ["dss0001-smoke"]);
    let hang_commits = git(&clone, &["log", "--format=%s"])
        .lines()
        .filter(|l| l.starts_with("hang:"))
        .count();
    assert_eq!(hang_commits, 1);
    let on_disk = PieceMeta::from_toml_str(
        &fs::read_to_string(clone.join("pieces/dss0001-smoke/piece.toml")).unwrap(),
    )
    .unwrap();
    assert_eq!(on_disk.catalog.as_deref(), Some("DSS0001"));

    let json: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(dir.join("piece.json")).unwrap()).unwrap();
    assert_eq!(json["catalog"], "DSS0001");
    assert_eq!(json["schema"], "dss-deposit/0");
    assert_eq!(json["title"], "Smoke");
    let bare_head = git(&bare, &["rev-parse", "HEAD"]);
    assert_eq!(json["source"]["commit"], bare_head.as_str());
    assert_eq!(git(&clone, &["rev-parse", "HEAD"]), bare_head);
    assert_eq!(json["source"]["path"], "pieces/dss0001-smoke");
    let media = json["media"].as_array().unwrap();
    assert_eq!(media.len(), 6);
    for m in media {
        let f = dir.join(m["file"].as_str().unwrap());
        assert!(f.is_file() && fs::metadata(&f).unwrap().len() > 0, "{}", f.display());
    }
    assert_eq!(png_size(&dir.join("still.png")), (1080, 1080));
    for (file, w, h) in [
        ("master.mp4", 1080, 1080),
        ("loop.mp4", 1080, 1080),
        ("feed-4x5.mp4", 1080, 1350),
        ("reel-9x16.mp4", 1080, 1920),
        ("wide-16x9.mp4", 1920, 1080),
    ] {
        let probe = run(
            &dir,
            "ffprobe",
            &[
                "-v", "error", "-select_streams", "v:0", "-count_frames",
                "-show_entries", "stream=width,height,nb_read_frames", "-of", "csv=p=0", file,
            ],
        );
        let frames = if file.starts_with("master") || file.starts_with("loop") { 60 } else { 900 };
        assert_eq!(probe, format!("{w},{h},{frames}"), "{file}");
        let color = run(
            &dir,
            "ffprobe",
            &[
                "-v", "error", "-select_streams", "v:0",
                "-show_entries", "stream=color_space,color_transfer,color_primaries",
                "-of", "default=nw=1", file,
            ],
        );
        for tag in ["color_space=bt709", "color_transfer=bt709", "color_primaries=bt709"] {
            assert!(color.lines().any(|l| l == tag), "{file}: {color}");
        }
    }
    assert_eq!(media[1]["seconds"], 1.0);
    assert_eq!(media[3]["seconds"], 15.0);
    assert!(!clone.join(".studio/hang/DSS0001").exists(), "frames left behind");

    // 5. Never overwrite a deposit.
    let err = hang::run(&ws, "smoke", hang::HangOpts { push: true }).unwrap_err().to_string();
    assert!(err.contains("already exists"), "{err}");
}
