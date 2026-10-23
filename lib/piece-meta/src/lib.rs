//! The `piece.toml` schema shared by `kit` and the `studio` CLI.

use anyhow::{Context, Result, bail};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PieceMeta {
    pub title: String,
    pub slug: String,
    pub created: String,
    pub statement: String,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub series: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub parent: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub catalog: Option<String>,
    #[serde(default)]
    pub live: bool,
    #[serde(default)]
    pub audio: bool,
    pub canvas: Canvas,
    pub time: TimeSpec,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Canvas {
    pub width: u32,
    pub height: u32,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TimeSpec {
    pub fps: u32,
    pub form: Form,
    pub seconds: f32,
    pub still_at: f32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Form {
    Loop,
    Duration,
}

impl PieceMeta {
    pub fn from_toml_str(s: &str) -> Result<PieceMeta> {
        toml::from_str(s).context("invalid piece.toml")
    }

    pub fn to_toml_string(&self) -> String {
        toml::to_string(self).expect("PieceMeta always serializes to TOML")
    }

    /// `round(seconds * fps)`.
    pub fn total_frames(&self) -> u32 {
        (self.time.seconds * self.time.fps as f32).round() as u32
    }

    /// Reasons this piece cannot be hung yet. Empty means ready.
    pub fn readiness_errors(&self) -> Vec<String> {
        let mut errs = Vec::new();
        if self.title.trim().is_empty() {
            errs.push("title is empty".to_string());
        }
        if self.statement.trim().is_empty() {
            errs.push("statement is empty".to_string());
        }
        if self.audio {
            errs.push("audio = true: sound recording is Phase 2".to_string());
        }
        if self.live {
            errs.push("live = true: live web builds are Phase 2".to_string());
        }
        let (w, h) = (self.canvas.width, self.canvas.height);
        if w == 0 || h == 0 || w % 2 != 0 || h % 2 != 0 {
            errs.push(format!(
                "canvas is {w}x{h}: width and height must be even and above zero for video"
            ));
        }
        if self.time.still_at > self.time.seconds {
            errs.push(format!(
                "still_at ({:?}) is past the end ({:?} s)",
                self.time.still_at, self.time.seconds
            ));
        }
        errs
    }
}

/// Slugs match `^[a-z0-9]+(-[a-z0-9]+)*$`, 1 to 40 characters.
pub fn validate_slug(slug: &str) -> Result<()> {
    if slug.is_empty() || slug.len() > 40 {
        bail!("slug must be 1 to 40 characters (got {})", slug.len());
    }
    let ok = slug.split('-').all(|part| {
        !part.is_empty()
            && part
                .bytes()
                .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit())
    });
    if !ok {
        bail!("slug {slug:?} must be lowercase letters and digits separated by single hyphens");
    }
    Ok(())
}

/// `catalog_id("DSS", 1)` is `"DSS0001"`.
pub fn catalog_id(prefix: &str, n: u32) -> String {
    format!("{prefix}{n:04}")
}

/// `hung_folder(12, "tide")` is `"dss0012-tide"`.
pub fn hung_folder(n: u32, slug: &str) -> String {
    format!("{}-{slug}", catalog_id("dss", n))
}

/// `wip_package("tide")` is `"wip-tide"`.
pub fn wip_package(slug: &str) -> String {
    format!("wip-{slug}")
}

#[cfg(test)]
mod tests {
    use super::*;

    const DEFAULT: &str = r#"
title = ""
slug = "drift-field"
created = "2026-10-22"
statement = ""
tags = []
live = false
audio = false

[canvas]
width = 1080
height = 1080

[time]
fps = 60
form = "loop"
seconds = 8.0
still_at = 0.0
"#;

    fn ready() -> PieceMeta {
        let mut m = PieceMeta::from_toml_str(DEFAULT).unwrap();
        m.title = "Drift Field".into();
        m.statement = "Points drift and settle.".into();
        m
    }

    #[test]
    fn parses_default_piece_toml() {
        let m = PieceMeta::from_toml_str(DEFAULT).unwrap();
        assert_eq!(m.series, None);
        assert_eq!(m.parent, None);
        assert_eq!(m.catalog, None);
        assert_eq!(m.total_frames(), 480);
        assert_eq!(m.time.form, Form::Loop);
    }

    #[test]
    fn round_trips() {
        let m = PieceMeta::from_toml_str(DEFAULT).unwrap();
        let again = PieceMeta::from_toml_str(&m.to_toml_string()).unwrap();
        assert_eq!(m, again);
        let mut m2 = ready();
        m2.series = Some("fields".into());
        m2.catalog = Some("DSS0001".into());
        m2.time.form = Form::Duration;
        assert_eq!(m2, PieceMeta::from_toml_str(&m2.to_toml_string()).unwrap());
    }

    #[test]
    fn readiness_errors_name_fields() {
        assert_eq!(ready().readiness_errors(), Vec::<String>::new());
        let has = |m: &PieceMeta, s: &str| m.readiness_errors().iter().any(|e| e.contains(s));
        let mut m = ready();
        m.title.clear();
        assert!(has(&m, "title"));
        let mut m = ready();
        m.statement.clear();
        assert!(has(&m, "statement"));
        let mut m = ready();
        m.audio = true;
        assert!(has(&m, "Phase 2"));
        let mut m = ready();
        m.live = true;
        assert!(has(&m, "Phase 2"));
        let mut m = ready();
        m.time.still_at = 9.0;
        assert!(has(&m, "still_at"));
        for (w, h) in [(1081, 1080), (1080, 1079), (0, 1080)] {
            let mut m = ready();
            m.canvas = Canvas { width: w, height: h };
            assert!(has(&m, "must be even"), "{w}x{h}");
        }
        let mut m = ready();
        m.canvas = Canvas { width: 1920, height: 1080 };
        assert_eq!(m.readiness_errors(), Vec::<String>::new());
    }

    #[test]
    fn slug_rules() {
        assert!(validate_slug("drift-field").is_ok());
        assert!(validate_slug("a1").is_ok());
        for bad in ["Drift", "drift--field", "-x", "x-", ""] {
            assert!(validate_slug(bad).is_err(), "{bad:?}");
        }
        assert!(validate_slug(&"a".repeat(40)).is_ok());
        assert!(validate_slug(&"a".repeat(41)).is_err());
    }

    #[test]
    fn naming() {
        assert_eq!(catalog_id("DSS", 1), "DSS0001");
        assert_eq!(hung_folder(12, "tide"), "dss0012-tide");
        assert_eq!(wip_package("tide"), "wip-tide");
    }
}
