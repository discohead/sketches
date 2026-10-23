use std::fs;
use std::path::Path;

use piece_meta::PieceMeta;
use studio::workspace::Workspace;
use studio::{new, pieces};

fn copy_dir(from: &Path, to: &Path) {
    fs::create_dir_all(to).unwrap();
    for e in fs::read_dir(from).unwrap() {
        let e = e.unwrap();
        let dest = to.join(e.file_name());
        if e.file_type().unwrap().is_dir() {
            copy_dir(&e.path(), &dest);
        } else {
            fs::copy(e.path(), dest).unwrap();
        }
    }
}

fn temp_ws() -> (tempfile::TempDir, Workspace) {
    let dir = tempfile::tempdir().unwrap();
    fs::write(
        dir.path().join("studio.toml"),
        "[catalog]\nprefix = \"DSS\"\n[deposit]\ndir = \"outbox\"\n[private]\ndir = \"../private\"\n[nannou]\nworktree = \"../nannou\"\n",
    )
    .unwrap();
    let templates = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../templates");
    copy_dir(&templates, &dir.path().join("templates"));
    fs::create_dir_all(dir.path().join("pieces")).unwrap();
    let ws = Workspace::discover(dir.path()).unwrap();
    (dir, ws)
}

#[test]
fn new_creates_piece() {
    let (_d, ws) = temp_ws();
    let p = new::create(&ws, "tide", None, "2026-10-22").unwrap();
    let dir = ws.pieces_dir().join("2026-10-22-tide");
    assert_eq!(p.dir, dir);
    assert_eq!(p.folder, "2026-10-22-tide");
    assert_eq!(p.package, "wip-tide");
    for f in ["Cargo.toml", "piece.toml", "src/main.rs"] {
        assert!(dir.join(f).is_file(), "{f}");
    }
    let cargo = fs::read_to_string(dir.join("Cargo.toml")).unwrap();
    assert!(cargo.contains("name = \"wip-tide\""));
    let meta = PieceMeta::from_toml_str(&fs::read_to_string(dir.join("piece.toml")).unwrap()).unwrap();
    assert_eq!(meta.slug, "tide");
    assert_eq!(meta.created, "2026-10-22");
    let main = fs::read_to_string(dir.join("src/main.rs")).unwrap();
    assert!(main.contains("kit::launch(kit::piece!()"));
    assert!(main.contains("kit::window!(app)"));
}

#[test]
fn new_from_sets_parent() {
    let (_d, ws) = temp_ws();
    let parent = new::create(&ws, "tide", None, "2026-10-22").unwrap();
    let mut m = parent.meta.clone();
    m.title = "Tide".into();
    m.statement = "Water.".into();
    m.series = Some("water".into());
    m.catalog = Some("DSS0001".into());
    m.canvas.width = 800;
    m.time.fps = 30;
    fs::write(parent.dir.join("piece.toml"), m.to_toml_string()).unwrap();
    fs::write(parent.dir.join("src/main.rs"), "// custom\n").unwrap();
    fs::write(parent.dir.join("src/extra.rs"), "// extra\n").unwrap();

    let c = new::create(&ws, "ebb", Some("tide"), "2026-10-23").unwrap();
    assert_eq!(c.meta.parent.as_deref(), Some("2026-10-22-tide"));
    assert_eq!(c.meta.title, "");
    assert_eq!(c.meta.statement, "");
    assert_eq!(c.meta.series, None);
    assert_eq!(c.meta.catalog, None);
    assert_eq!(c.meta.canvas.width, 800);
    assert_eq!(c.meta.time.fps, 30);
    assert_eq!(c.meta.slug, "ebb");
    assert_eq!(fs::read_to_string(c.dir.join("src/main.rs")).unwrap(), "// custom\n");
    assert!(c.dir.join("src/extra.rs").is_file());
    assert_eq!(pieces::all(&ws).unwrap().len(), 2);
}

#[test]
fn new_rejects_duplicates_and_bad_slugs() {
    let (_d, ws) = temp_ws();
    new::create(&ws, "tide", None, "2026-10-22").unwrap();
    // same folder
    assert!(new::create(&ws, "tide", None, "2026-10-22").is_err());
    // same package on another day
    let e = new::create(&ws, "tide", None, "2026-10-23").unwrap_err();
    assert!(e.to_string().contains("wip-tide"), "{e}");
    for bad in ["Tide", "a--b", "", "-x"] {
        assert!(new::create(&ws, bad, None, "2026-10-22").is_err(), "{bad:?}");
    }
    assert!(new::create(&ws, "x", Some("nope"), "2026-10-22").is_err());
}
