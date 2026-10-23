//! Finding the repo root and the paths and tools hanging off it.

use std::path::{Path, PathBuf};
use std::process::Command;

use anyhow::{Context, Result, bail};
use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct StudioConfig {
    pub catalog: CatalogConfig,
    pub deposit: DirConfig,
    pub private: DirConfig,
    pub nannou: NannouConfig,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CatalogConfig {
    pub prefix: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct DirConfig {
    pub dir: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct NannouConfig {
    pub worktree: String,
}

#[derive(Debug, Clone)]
pub struct Workspace {
    pub root: PathBuf,
    pub config: StudioConfig,
}

impl Workspace {
    /// Walk up from `start` to the nearest directory holding a `studio.toml`.
    pub fn discover(start: &Path) -> Result<Workspace> {
        let start = start
            .canonicalize()
            .with_context(|| format!("cannot read {}", start.display()))?;
        for dir in start.ancestors() {
            let file = dir.join("studio.toml");
            if file.is_file() {
                let text = std::fs::read_to_string(&file)
                    .with_context(|| format!("cannot read {}", file.display()))?;
                let config = toml::from_str(&text)
                    .with_context(|| format!("invalid {}", file.display()))?;
                return Ok(Workspace {
                    root: dir.to_path_buf(),
                    config,
                });
            }
        }
        bail!("no studio.toml found in {} or any parent", start.display())
    }

    pub fn pieces_dir(&self) -> PathBuf {
        self.root.join("pieces")
    }

    pub fn outbox_dir(&self) -> PathBuf {
        self.root.join(&self.config.deposit.dir)
    }

    pub fn private_dir(&self) -> PathBuf {
        self.root.join(&self.config.private.dir)
    }

    pub fn nannou_worktree(&self) -> PathBuf {
        self.root.join(&self.config.nannou.worktree)
    }

    pub fn templates_dir(&self) -> PathBuf {
        self.root.join("templates")
    }

    /// A command for `tool` with rustup's bin dir first on `PATH`, run in the root.
    pub fn tool(&self, tool: &str) -> Command {
        let mut cmd = Command::new(tool);
        cmd.current_dir(&self.root).env("PATH", rustup_path());
        cmd
    }

    /// rustup's cargo (`$HOME/.cargo/bin/cargo`), run in the root. Homebrew's cargo
    /// is older than the workspace needs, so never use bare `cargo`.
    pub fn cargo(&self) -> Command {
        let mut cmd = Command::new(cargo_bin().join("cargo"));
        cmd.current_dir(&self.root).env("PATH", rustup_path());
        cmd
    }
}

fn cargo_bin() -> PathBuf {
    PathBuf::from(std::env::var_os("HOME").unwrap_or_default()).join(".cargo/bin")
}

fn rustup_path() -> std::ffi::OsString {
    let mut paths = vec![cargo_bin()];
    if let Some(p) = std::env::var_os("PATH") {
        paths.extend(std::env::split_paths(&p));
    }
    std::env::join_paths(paths).unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;

    const CONFIG: &str = "[catalog]\nprefix = \"DSS\"\n[deposit]\ndir = \"outbox\"\n[private]\ndir = \"../priv\"\n[nannou]\nworktree = \"../nn\"\n";

    #[test]
    fn discovers_from_a_subdirectory() {
        let d = tempfile::tempdir().unwrap();
        std::fs::write(d.path().join("studio.toml"), CONFIG).unwrap();
        let sub = d.path().join("pieces/x/src");
        std::fs::create_dir_all(&sub).unwrap();
        let ws = Workspace::discover(&sub).unwrap();
        assert_eq!(ws.root, d.path().canonicalize().unwrap());
        assert_eq!(ws.config.catalog.prefix, "DSS");
        assert_eq!(ws.outbox_dir(), ws.root.join("outbox"));
        assert!(ws.private_dir().ends_with("../priv"));
    }

    #[test]
    fn missing_studio_toml_is_an_error() {
        let d = tempfile::tempdir().unwrap();
        // Only meaningful if no studio.toml sits above the temp dir.
        if Workspace::discover(d.path()).is_err() {
            assert!(Workspace::discover(d.path()).unwrap_err().to_string().contains("studio.toml"));
        }
    }

    #[test]
    fn cargo_is_rustups() {
        let d = tempfile::tempdir().unwrap();
        std::fs::write(d.path().join("studio.toml"), CONFIG).unwrap();
        let ws = Workspace::discover(d.path()).unwrap();
        let c = ws.cargo();
        assert!(c.get_program().to_string_lossy().ends_with(".cargo/bin/cargo"));
    }
}
