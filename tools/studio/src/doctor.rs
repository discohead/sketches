//! `studio doctor`: check the tools and folders the studio depends on.

use std::process::Command;

use crate::workspace::Workspace;

const MIN_CARGO: (u32, u32, u32) = (1, 95, 0);
const WASM_BINDGEN: &str = "0.2.125";
const NANNOU_FIX: &str = "(1024, 768).into()";

/// Parse `"rustc 1.99.1 (58cf8f9c3 2026-10-13)"` (or `cargo ...`) into `(1, 99, 1)`.
pub fn parse_rustc_version(s: &str) -> Option<(u32, u32, u32)> {
    let v = s.split_whitespace().nth(1)?;
    let v = v.split(['-', '+']).next()?;
    let mut it = v.split('.');
    let major = it.next()?.parse().ok()?;
    let minor = it.next()?.parse().ok()?;
    let patch = it.next()?.parse().ok()?;
    if it.next().is_some() {
        return None;
    }
    Some((major, minor, patch))
}

enum Status {
    Ok(String),
    Fail(String),
    Warn(String),
}

fn run(cmd: &mut Command) -> Result<String, String> {
    let out = cmd.output().map_err(|e| format!("could not run: {e}"))?;
    if !out.status.success() {
        let err = String::from_utf8_lossy(&out.stderr);
        return Err(format!("exited with {}: {}", out.status, err.trim()));
    }
    Ok(String::from_utf8_lossy(&out.stdout).trim().to_string())
}

fn check_cargo(ws: &Workspace) -> Status {
    match run(ws.cargo().arg("--version")) {
        Ok(out) => match parse_rustc_version(&out) {
            Some(v) if v >= MIN_CARGO => Status::Ok(out),
            Some(_) => Status::Fail(format!("{out}: need 1.95 or newer")),
            None => Status::Fail(format!("cannot read version from {out:?}")),
        },
        Err(e) => Status::Fail(e),
    }
}

fn check_wasm_target(ws: &Workspace) -> Status {
    match run(ws.tool("rustup").args(["target", "list", "--installed"])) {
        Ok(out) if out.lines().any(|l| l.trim() == "wasm32-unknown-unknown") => {
            Status::Ok("installed".into())
        }
        Ok(_) => Status::Fail("run: rustup target add wasm32-unknown-unknown".into()),
        Err(e) => Status::Fail(e),
    }
}

fn check_wasm_bindgen(ws: &Workspace) -> Status {
    match run(ws.tool("wasm-bindgen").arg("--version")) {
        Ok(out) if out.split_whitespace().nth(1) == Some(WASM_BINDGEN) => Status::Ok(out),
        Ok(out) => Status::Fail(format!("{out}: need {WASM_BINDGEN} (cargo install wasm-bindgen-cli --version {WASM_BINDGEN})")),
        Err(e) => Status::Fail(e),
    }
}

fn check_ffmpeg(ws: &Workspace) -> Status {
    match run(ws.tool("ffmpeg").arg("-version")) {
        Ok(out) => Status::Ok(out.lines().next().unwrap_or("").to_string()),
        Err(e) => Status::Fail(format!("ffmpeg not found on PATH ({e})")),
    }
}

fn check_worktree(ws: &Workspace) -> Status {
    let wt = ws.nannou_worktree();
    if !wt.is_dir() {
        return Status::Fail(format!("{} does not exist", wt.display()));
    }
    let branch = match run(Command::new("git").arg("-C").arg(&wt).args(["rev-parse", "--abbrev-ref", "HEAD"])) {
        Ok(b) => b,
        Err(e) => return Status::Fail(e),
    };
    if branch != "sketches" {
        return Status::Fail(format!("{} is on branch {branch}, need sketches", wt.display()));
    }
    match std::fs::read_to_string(wt.join("nannou/src/app.rs")) {
        Ok(src) if src.contains(NANNOU_FIX) => Status::Ok(format!("{} on sketches, window fix present", wt.display())),
        Ok(_) => Status::Fail(format!("nannou/src/app.rs lacks the line {NANNOU_FIX}")),
        Err(e) => Status::Fail(format!("cannot read nannou/src/app.rs: {e}")),
    }
}

fn check_private(ws: &Workspace) -> Status {
    let p = ws.private_dir();
    if p.is_dir() {
        Status::Ok(p.display().to_string())
    } else {
        Status::Fail(format!("{} does not exist", p.display()))
    }
}

fn check_origin(ws: &Workspace) -> Status {
    match run(Command::new("git").current_dir(&ws.root).args(["remote", "get-url", "origin"])) {
        Ok(url) => Status::Ok(url),
        Err(_) => Status::Warn("no origin remote yet".into()),
    }
}

/// Print one line per check. Returns true if every check passed (warnings are fine).
pub fn run_all(ws: &Workspace) -> bool {
    let checks: [(&str, fn(&Workspace) -> Status); 7] = [
        ("cargo >= 1.95 (rustup)", check_cargo),
        ("wasm32-unknown-unknown target", check_wasm_target),
        ("wasm-bindgen 0.2.125", check_wasm_bindgen),
        ("ffmpeg on PATH", check_ffmpeg),
        ("nannou worktree on branch sketches", check_worktree),
        ("private dir exists", check_private),
        ("git origin remote", check_origin),
    ];
    let mut all_ok = true;
    for (label, check) in checks {
        match check(ws) {
            Status::Ok(d) => println!("ok    {label}: {d}"),
            Status::Warn(d) => println!("warn  {label}: {d}"),
            Status::Fail(d) => {
                all_ok = false;
                println!("FAIL  {label}: {d}");
            }
        }
    }
    all_ok
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rustc_version() {
        assert_eq!(parse_rustc_version("rustc 1.99.1 (58cf8f9c3 2026-10-13)"), Some((1, 99, 1)));
        assert_eq!(parse_rustc_version("cargo 1.95.0 (abc 2026-01-01)"), Some((1, 95, 0)));
        assert_eq!(parse_rustc_version("garbage"), None);
        assert_eq!(parse_rustc_version("rustc one.two.three"), None);
    }
}
