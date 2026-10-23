//! The deposit contract: `piece.json` and the media list. See `docs/deposit-spec.md`.

use anyhow::{Result, bail};
use piece_meta::{Form, PieceMeta};

use crate::media::loop_repeats;
use serde::Serialize;

/// Bumped on any breaking change to `piece.json` or the package layout.
pub const SCHEMA: &str = "dss-deposit/0";

/// Social cuts last at least this long; shorter loops repeat.
pub const MIN_CUT_SECONDS: f32 = 15.0;

/// A social cut: file name, aspect label, and pixel size.
#[derive(Debug, Clone, Copy)]
pub struct Cut {
    pub file: &'static str,
    pub aspect: &'static str,
    pub width: u32,
    pub height: u32,
}

pub const CUTS: [Cut; 3] = [
    Cut { file: "feed-4x5.mp4", aspect: "4:5", width: 1080, height: 1350 },
    Cut { file: "reel-9x16.mp4", aspect: "9:16", width: 1080, height: 1920 },
    Cut { file: "wide-16x9.mp4", aspect: "16:9", width: 1920, height: 1080 },
];

pub const STILL: &str = "still.png";
pub const MASTER: &str = "master.mp4";
pub const WEB: &str = "loop.mp4";

#[derive(Debug, Clone, Serialize)]
pub struct Deposit {
    pub schema: String,
    pub catalog: String,
    pub title: String,
    pub slug: String,
    pub date: String,
    pub statement: String,
    pub tags: Vec<String>,
    pub series: Option<String>,
    pub parent: Option<String>,
    pub source: Source,
    pub canvas: Size,
    pub time: Time,
    pub live: bool,
    pub audio: bool,
    pub media: Vec<Media>,
}

#[derive(Debug, Clone, Serialize)]
pub struct Source {
    pub repo: String,
    pub commit: String,
    pub path: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct Size {
    pub width: u32,
    pub height: u32,
}

#[derive(Debug, Clone, Serialize)]
pub struct Time {
    pub fps: u32,
    pub form: Form,
    pub seconds: f32,
}

#[derive(Debug, Clone, Serialize)]
pub struct Media {
    pub file: String,
    pub kind: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub aspect: Option<String>,
    pub width: u32,
    pub height: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub seconds: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fps: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub audio: Option<bool>,
}

/// How many times the cuts repeat the piece: enough to reach 15 s for a loop,
/// once for a fixed duration.
pub fn cut_repeats(meta: &PieceMeta) -> u32 {
    match meta.time.form {
        Form::Loop => loop_repeats(meta.time.seconds, MIN_CUT_SECONDS),
        Form::Duration => 1,
    }
}

fn round3(x: f64) -> f64 {
    (x * 1000.0).round() / 1000.0
}

/// The media list for a piece, in deposit order.
pub fn media_list(meta: &PieceMeta) -> Vec<Media> {
    let (w, h) = (meta.canvas.width, meta.canvas.height);
    let video = |file: &str, kind: &str, aspect: Option<&str>, width, height, seconds| Media {
        file: file.into(),
        kind: kind.into(),
        aspect: aspect.map(Into::into),
        width,
        height,
        seconds: Some(seconds),
        fps: Some(meta.time.fps),
        audio: Some(meta.audio),
    };
    // The rendered length, which can differ slightly from `time.seconds` when
    // `seconds * fps` is not a whole number of frames.
    let frames = f64::from(meta.total_frames());
    let fps = f64::from(meta.time.fps);
    let seconds = round3(frames / fps);
    let cut_seconds = round3(f64::from(cut_repeats(meta)) * frames / fps);
    let mut media = vec![
        Media {
            file: STILL.into(),
            kind: "still".into(),
            aspect: None,
            width: w,
            height: h,
            seconds: None,
            fps: None,
            audio: None,
        },
        video(MASTER, "master", None, w, h, seconds),
        video(WEB, "web", None, w, h, seconds),
    ];
    for c in CUTS {
        media.push(video(c.file, "social", Some(c.aspect), c.width, c.height, cut_seconds));
    }
    media
}

impl Deposit {
    /// `parent` is the parent's catalog id (or `None`); `path` is the piece's
    /// folder relative to the repo root.
    pub fn new(
        meta: &PieceMeta,
        catalog: &str,
        date: &str,
        parent: Option<String>,
        repo: &str,
        commit: &str,
        path: &str,
    ) -> Deposit {
        Deposit {
            schema: SCHEMA.into(),
            catalog: catalog.into(),
            title: meta.title.clone(),
            slug: meta.slug.clone(),
            date: date.into(),
            statement: meta.statement.clone(),
            tags: meta.tags.clone(),
            series: meta.series.clone(),
            parent,
            source: Source { repo: repo.into(), commit: commit.into(), path: path.into() },
            canvas: Size { width: meta.canvas.width, height: meta.canvas.height },
            time: Time { fps: meta.time.fps, form: meta.time.form, seconds: meta.time.seconds },
            live: meta.live,
            audio: meta.audio,
            media: media_list(meta),
        }
    }

    /// Pretty-printed `piece.json`, ending in a newline.
    pub fn to_json(&self) -> String {
        let mut s = serde_json::to_string_pretty(self).expect("Deposit always serializes to JSON");
        s.push('\n');
        s
    }
}

/// The deposit's `source.repo` for an `origin` URL: it must normalize to
/// `https://github.com/<owner>/<repo>` unless `allow_any` is set (tests only).
pub fn source_repo(origin: &str, allow_any: bool) -> Result<String> {
    let url = normalize_repo_url(origin);
    let is_github = url.strip_prefix("https://github.com/").is_some_and(|path| {
        let parts: Vec<&str> = path.split('/').collect();
        parts.len() == 2 && parts.iter().all(|p| !p.is_empty())
    });
    if !is_github && !allow_any {
        bail!(
            "origin is {origin}, which is not a GitHub repo. The deposit links to the code on \
             GitHub, so origin must be https://github.com/<owner>/<repo> or \
             git@github.com:<owner>/<repo>.git"
        );
    }
    Ok(url)
}

/// GitHub remote URLs in any common form become `https://github.com/<owner>/<repo>`.
/// Anything else is returned unchanged.
pub fn normalize_repo_url(url: &str) -> String {
    let url = url.trim();
    let path = if let Some(rest) = url.strip_prefix("git@github.com:") {
        Some(rest)
    } else {
        ["https://", "http://", "ssh://", "git://"]
            .iter()
            .find_map(|scheme| url.strip_prefix(scheme))
            .and_then(|rest| {
                // Drop any `user@` and require the host to be github.com.
                let rest = rest.rsplit_once('@').map_or(rest, |(_, r)| r);
                rest.strip_prefix("github.com/")
            })
    };
    let Some(path) = path else {
        return url.to_string();
    };
    let path = path.trim_end_matches('/');
    let path = path.strip_suffix(".git").unwrap_or(path);
    format!("https://github.com/{path}")
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::Value;

    fn meta() -> PieceMeta {
        let mut m = crate::new::default_meta("tide", "2026-10-22");
        m.title = "Tide".into();
        m.statement = "Water.".into();
        m.tags = vec!["water".into()];
        m
    }

    #[test]
    fn deposit_json_shape() {
        let d = Deposit::new(
            &meta(),
            "DSS0001",
            "2026-10-23",
            None,
            "https://github.com/discohead/sketches",
            "abc123",
            "pieces/dss0001-tide",
        );
        let v: Value = serde_json::from_str(&d.to_json()).unwrap();
        let keys: Vec<&str> = v.as_object().unwrap().keys().map(String::as_str).collect();
        for k in [
            "schema", "catalog", "title", "slug", "date", "statement", "tags", "series",
            "parent", "source", "canvas", "time", "live", "audio", "media",
        ] {
            assert!(keys.contains(&k), "missing {k} in {keys:?}");
        }
        assert_eq!(keys.len(), 15, "{keys:?}");
        assert_eq!(v["schema"], "dss-deposit/0");
        assert_eq!(v["catalog"], "DSS0001");
        assert_eq!(v["date"], "2026-10-23");
        assert_eq!(v["tags"], serde_json::json!(["water"]));
        assert!(v["series"].is_null());
        assert!(v["parent"].is_null());
        assert_eq!(
            v["source"],
            serde_json::json!({"repo": "https://github.com/discohead/sketches", "commit": "abc123", "path": "pieces/dss0001-tide"})
        );
        assert_eq!(v["canvas"], serde_json::json!({"width": 1080, "height": 1080}));
        assert_eq!(v["time"], serde_json::json!({"fps": 60, "form": "loop", "seconds": 8.0}));
        assert_eq!(v["live"], false);
        assert_eq!(v["audio"], false);

        let media = v["media"].as_array().unwrap();
        let files: Vec<&str> = media.iter().map(|m| m["file"].as_str().unwrap()).collect();
        assert_eq!(
            files,
            ["still.png", "master.mp4", "loop.mp4", "feed-4x5.mp4", "reel-9x16.mp4", "wide-16x9.mp4"]
        );
        assert_eq!(
            media[0],
            serde_json::json!({"file": "still.png", "kind": "still", "width": 1080, "height": 1080})
        );
        let video = serde_json::json!({"width": 1080, "height": 1080, "seconds": 8.0, "fps": 60, "audio": false});
        for (m, kind) in [(&media[1], "master"), (&media[2], "web")] {
            let mut want = video.clone();
            want["file"] = m["file"].clone();
            want["kind"] = kind.into();
            assert_eq!(*m, want);
        }
        assert_eq!(
            media[3],
            serde_json::json!({"file": "feed-4x5.mp4", "kind": "social", "aspect": "4:5", "width": 1080, "height": 1350, "seconds": 16.0, "fps": 60, "audio": false})
        );
        assert_eq!(media[4]["aspect"], "9:16");
        assert_eq!((media[4]["width"].as_u64(), media[4]["height"].as_u64()), (Some(1080), Some(1920)));
        assert_eq!(media[5]["aspect"], "16:9");
        assert_eq!((media[5]["width"].as_u64(), media[5]["height"].as_u64()), (Some(1920), Some(1080)));

        let mut with = meta();
        with.series = Some("water".into());
        let d = Deposit::new(&with, "DSS0002", "2026-10-23", Some("DSS0001".into()), "r", "c", "p");
        let v: Value = serde_json::from_str(&d.to_json()).unwrap();
        assert_eq!(v["series"], "water");
        assert_eq!(v["parent"], "DSS0001");
    }

    #[test]
    fn duration_cuts_play_once() {
        let mut m = meta();
        m.time.form = Form::Duration;
        m.time.seconds = 5.0;
        assert_eq!(cut_repeats(&m), 1);
        assert_eq!(media_list(&m)[3].seconds, Some(5.0));
        m.time.form = Form::Loop;
        assert_eq!(cut_repeats(&m), 3);
        assert_eq!(media_list(&m)[3].seconds, Some(15.0));
    }

    #[test]
    fn normalizes_repo_urls() {
        let want = "https://github.com/discohead/sketches";
        for url in [
            "git@github.com:discohead/sketches.git",
            "https://github.com/discohead/sketches.git",
            "https://github.com/discohead/sketches",
            "https://github.com/discohead/sketches/",
            "ssh://git@github.com/discohead/sketches.git",
            "https://jared@github.com/discohead/sketches.git",
        ] {
            assert_eq!(normalize_repo_url(url), want, "{url}");
        }
        assert_eq!(normalize_repo_url("/tmp/bare.git"), "/tmp/bare.git");
    }

    #[test]
    fn source_repo_must_be_github() {
        assert_eq!(
            source_repo("git@github.com:discohead/sketches.git", false).unwrap(),
            "https://github.com/discohead/sketches"
        );
        for bad in [
            "/tmp/bare.git",
            "https://gitlab.com/a/b.git",
            "https://github.com/discohead",
            "https://github.com/a/b/c",
        ] {
            let e = source_repo(bad, false).unwrap_err().to_string();
            assert!(e.contains("not a GitHub repo"), "{bad}: {e}");
        }
        assert_eq!(source_repo("/tmp/bare.git", true).unwrap(), "/tmp/bare.git");
    }

    #[test]
    fn media_seconds_are_rendered_length() {
        let mut m = meta();
        m.time.seconds = 7.3; // 438 frames at 60 fps
        let media = media_list(&m);
        assert_eq!(media[1].seconds, Some(7.3));
        assert_eq!(cut_repeats(&m), 3);
        assert_eq!(media[3].seconds, Some(21.9));
        let d = Deposit::new(&m, "DSS0001", "2026-10-23", None, "r", "c", "p");
        let json = d.to_json();
        assert!(json.contains("\"seconds\": 21.9,"), "{json}");
        assert!(!json.contains("21.900"), "{json}");
        // 8.33 s at 30 fps renders 250 frames: 8.333 s.
        m.time.seconds = 8.33;
        m.time.fps = 30;
        assert_eq!(media_list(&m)[1].seconds, Some(8.333));
    }
}
