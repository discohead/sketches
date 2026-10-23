# Sketches

Jared's daily art practice: audiovisual pieces in Rust with Nannou on Bevy 0.19.
Jared directs. Claude writes all of the code. Start an evening with `/begin`;
finish a piece with `/hang`.

The design is in `docs/specs/2026-10-22-practice-system-design.md`. The
aesthetic is in `docs/aesthetic.md`, which builds on
`/Users/jaredmcfarland/Developer/writings/.claude/skills/patch-bay/aesthetic.md`.
The shared vocabulary is in `docs/lexicon.md`. The deposit contract is in
`docs/deposit-spec.md`.

## Commands

Run everything from the repo root. Use `scripts/cargo` for every cargo call,
never bare `cargo`: Homebrew's cargo comes first on `PATH` and cannot build
Bevy 0.19.

| Command | What it does |
| --- | --- |
| `scripts/studio new <slug> [--from <piece>]` | Creates `pieces/<YYYY-MM-DD>-<slug>/` with `piece.toml`, `Cargo.toml` (package `wip-<slug>`) and `src/main.rs`. `--from <piece>` makes a variation: it copies the parent's folder (`src/`, `assets/`, other files; not `target/` or dotfiles), keeps its `Cargo.toml` dependencies, and writes a fresh `piece.toml` that records `parent` and keeps its canvas, time, `live` and `audio`. |
| `scripts/studio list` | Lists every piece, hung and in progress. |
| `scripts/studio run <piece>` | Builds and opens the live window. It blocks, so run it in the background, and stop the previous run first. A rebuild after an edit takes about 1 s; a new piece about 3 s. |
| `scripts/studio look <piece> [--at 0,1.5,3] [--count 6]` | Renders frames offline. Prints the montage path first, then each frame. A small capture window flashes briefly; that is expected. |
| `scripts/studio hang <piece> [--no-push]` | Records, numbers, commits, pushes and deposits a finished piece. Use it through `/hang`. |
| `scripts/studio doctor` | Checks tools and folders. Run it when something is broken. |
| `scripts/cargo …` | Cargo with the pinned toolchain. |

`<piece>` is a folder name under `pieces/`, its slug, or its catalog number.

Every folder under `pieces/`, dotted ones included, must hold a `Cargo.toml`:
cargo's `pieces/*` member glob takes them all, and one without a manifest makes
every cargo command fail with "failed to load manifest for workspace member".
`scripts/studio` names the folder before running cargo. Remove it if it is empty
or left over.

To stop a running piece, run:

```sh
for p in $(pgrep -f 'target/debug/studio run '); do pkill -P "$p"; done
```

It kills only the windows that `studio run` started, so a `look` or `hang`
capture in progress is left alone. Never stop a piece any broader way while a
`look` or `hang` is running.

## Writing a piece

- `main` is `kit::launch(kit::piece!(), nannou::app(model).update(update));`.
- Create the window with `kit::window!(app).view(view).build();`. It returns
  Nannou's full window builder, so key and mouse handlers chain before
  `.build()`. Never create a window another way.
- `kit::loop_phase(app)` is the phase of the loop in `[0, 1)`.
- **Capture-safe rule.** A piece's look must be a function of `app.time()`,
  `app.elapsed_frames()` or `kit::loop_phase(app)`. A piece that simulates must
  step by `app.time_delta()`, never a fixed amount per update. Capture runs a
  variable number of warm-up updates with time held at 0, and recordings are
  frame-locked, so anything else renders differently each time. Discrete steps
  (a cellular automaton generation, a new grain) advance when
  `app.elapsed_frames()` changes, never once per update.
- **Seamless loops.** For `form = "loop"`, the state at `t = seconds` must
  equal the state at `t = 0`. Drive periodic motion from `loop_phase`, and make
  every frequency a whole number of cycles per loop.
- Randomness must be seeded, so a capture is repeatable.
- The live window renders at exact canvas pixels. On a Retina display it looks
  half-size. That is intended: the preview matches the recording pixel for
  pixel.
- **The canvas must fit on the display at exact pixels.** The live and capture
  windows are canvas-sized, and macOS won't make a window larger than the
  screen, so a canvas that doesn't fit records at the wrong size and `look` and
  `hang` fail. Default to 1080 × 1080 and keep it. For portrait (1080 × 1920),
  4K or anything larger than 1080 on a side, ask Jared first and check the
  screen: `system_profiler SPDisplaysDataType | rg 'Resolution|Looks like'`.
  The canvas in points (pixels ÷ 2 on Retina) plus about 60 for the title and
  menu bars must fit inside "UI Looks like" of the display the window opens on
  (with several displays, ask which). 1080 × 1920 does not fit on a 14"
  MacBook Pro.
- **Private materials stay private.** Materials are read at runtime by
  absolute path from `/Users/jaredmcfarland/Developer/sketches-private/materials/`
  and never copied under `pieces/`. `studio hang` commits everything in the
  piece folder and pushes it publicly. Use `include_bytes!` or a piece
  `assets/` folder only for files that can be public (WGSL, data you made). A
  hung piece that reads private materials won't rebuild on another machine;
  that is accepted. The code it publishes names the file by absolute path, so
  give material files neutral names (`grain-01.jpg`, not a person or a place).
- Assets load relative to the piece folder (`pieces/<folder>/assets/`) in
  `run`, `look` and `hang` alike. Run `studio look` early on any piece that
  loads files.
- Extra crates come from the workspace. Add the lines you need to the piece's
  own `Cargo.toml` under `[dependencies]`:

  ```toml
  nannou_audio = { workspace = true }
  nannou_video = { workspace = true }
  image = { workspace = true, features = ["jpeg"] }
  ```

  The workspace `image` is PNG-only; add `"jpeg"` (or another format) in the
  piece. `studio new --from` keeps the parent's dependencies.
- Nannou's examples for this version are in
  `/Users/jaredmcfarland/Developer/nannou-sketches/examples/`. Read the
  matching one before using a medium for the first time.

`piece.toml` fields:

- top level: `title`, `slug`, `created`, `statement`, `tags`, `series`,
  `parent`, `catalog` (set by hang; never set it by hand), `live`, `audio`
- `[canvas]`: `width`, `height` (default 1080 × 1080)
- `[time]`: `fps`, `form` (`"loop"` or `"duration"`), `seconds`, `still_at`

`audio = true` and `live = true` are refused by `hang` in Phase 1. Sound can be
monitored live with `nannou_audio`; recording it is Phase 2. A sound piece hangs
silent with `audio = false`.

## Look before critiquing

Never judge a piece from its code. Run `scripts/studio look <piece>`, Read the
montage first, and open single frames only when you need detail. Judge what you
see against `docs/aesthetic.md`: name clichés out loud, including in your own
suggestions, and say plainly when something works.

## Art first

Jared's rule: art first. Tooling is built only after the piece is hung.

- While a piece is in progress, the code it needs lives in the piece, even if
  rough or repeated. No refactors, no library work, no generalizing.
- Do not edit `lib/`, `templates/` or `tools/` while a piece is in progress.
- When something looks worth extracting, add it to the harvest list at
  `/Users/jaredmcfarland/Developer/sketches-private/journal/harvest.md` and
  keep going.
- If Jared pulls toward tooling mid-piece, name it once, kindly. He decides.
- Something graduates to `lib/` or `templates/` after it has appeared in two or
  three hung pieces.

## Hung pieces are frozen

A hung piece (`pieces/dss####-<slug>/`) is never edited again: not its code, not
its `piece.toml`. A variation is a new piece made with
`scripts/studio new <slug> --from <piece>`. Catalog numbers (DSS0001, …) are
assigned only by `studio hang`.

## Private material

The journal (session notes, threads, harvest list) and `materials/` (Jared's
raw images, video, textures and audio) live in
`/Users/jaredmcfarland/Developer/sketches-private`. This repo is public. Never
copy, commit or quote private material into it. A piece may read a file from
`materials/` at runtime; the file itself never comes here.

## Git

- Nothing pushes except `scripts/studio hang`, and only after Jared says to
  hang the piece.
- Commit only paths you changed, with an explicit pathspec.
- Every commit, in this repo and in the private one, ends with a blank line
  and `Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>`.

## Writing

Plain, direct English, in the skills, in commit messages, and in conversation
with Jared. No filler, no hype, no "great question", no exclamation marks. Say
what it is, what it does, or what you think.
