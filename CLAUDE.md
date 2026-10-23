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
| `scripts/studio new <slug> [--from <piece>]` | Creates `pieces/<YYYY-MM-DD>-<slug>/` with `piece.toml`, `Cargo.toml` (package `wip-<slug>`) and `src/main.rs`. `--from` copies another piece as a variation. |
| `scripts/studio list` | Lists every piece, hung and in progress. |
| `scripts/studio run <piece>` | Builds and opens the live window. It blocks, so run it in the background, and stop the previous run first. A rebuild after an edit takes about 1 s; a new piece about 3 s. |
| `scripts/studio look <piece> [--at 0,1.5,3] [--count 6]` | Renders frames offline. Prints the montage path first, then each frame. A small capture window flashes briefly; that is expected. |
| `scripts/studio hang <piece> [--no-push]` | Records, numbers, commits, pushes and deposits a finished piece. Use it through `/hang`. |
| `scripts/studio doctor` | Checks tools and folders. Run it when something is broken. |
| `scripts/cargo …` | Cargo with the pinned toolchain. |

`<piece>` is a folder name under `pieces/`, its slug, or its catalog number.

To stop a running piece: stop the background task you started, or run
`pkill -f 'target/debug/(wip-|dss[0-9])'`.

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
  frame-locked, so anything else renders differently each time.
- **Seamless loops.** For `form = "loop"`, the state at `t = seconds` must
  equal the state at `t = 0`. Drive periodic motion from `loop_phase`, and make
  every frequency a whole number of cycles per loop.
- Randomness must be seeded, so a capture is repeatable.
- The live window renders at exact canvas pixels. On a Retina display it looks
  half-size. That is intended: the preview matches the recording pixel for
  pixel.
- Load files (images, video, data) by absolute path with your own code, or embed
  them with `include_bytes!`. `studio look` and `studio hang` run the piece's
  binary directly, so Bevy's asset server may not find a piece's `assets/`
  folder. If a piece does use the asset server (shader models load WGSL that
  way), run `studio look` early to confirm the capture sees it.
- Extra crates (`nannou_audio`, `nannou_video`, `image`) go in the piece's own
  `Cargo.toml`. `studio new --from` does not copy the parent's extra
  dependencies yet; copy them by hand.
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
- Commit messages end with a blank line and the session's `Co-Authored-By`
  line.

## Writing

Plain, direct English, in the skills, in commit messages, and in conversation
with Jared. No filler, no hype, no "great question", no exclamation marks. Say
what it is, what it does, or what you think.
