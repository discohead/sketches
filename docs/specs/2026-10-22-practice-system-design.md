# Sketches: a daily Nannou practice

Design document, 2026-10-22. Written from a brainstorming session with Jared.
Anything marked **(Claude's call)** was not decided by Jared and can be changed.
Everything else is his.

## 1. What this is and why

Jared wants a daily art practice: an hour most evenings, sometimes two, making
audiovisual pieces with Nannou (the Rust creative coding library, now built on
Bevy) through Claude Code. Jared directs. Claude writes all of the code.

The system is the environment for that practice. It is three things at once:

1. **A collaborator.** A `/begin` session that acts as an artistic guide,
   thought partner, mentor, and friend. It offers a seed of inspiration and is
   just as happy to follow an idea Jared brings. It works out when to pull him
   along and when to follow.
2. **A performance coach.** It cares that pieces get finished and shared. Jared
   has a pattern he named himself: he built his Curves engine five times and
   shipped little art with it. His corrective is "art first, ship it." The coach
   exists to make finishing the normal outcome of an evening.
3. **A workshop that grows.** Jared and Claude are engineers too. Over time they
   build their own library, templates, and vocabulary out of the practice, so
   the work gets faster and the language between them gets richer. Tooling is
   earned by repetition and only built after art is hung (section 9).

The end of the line is a **deposit**: a finished piece, packaged the same way
every time, dropped somewhere that causes it to appear on `datspace.studio`. The
site itself is out of scope. It is a black box to this system.

## 2. Goals, non-goals, anti-goals

**Goals**

- Most evenings end with a piece hung: recorded, titled, described, deposited.
- Sessions start fast. A first image is on screen within minutes of `/begin`.
- The work has range: 2D, 3D, shaders, images, video, textures, sampling and
  processing of raw material, synthesis, rendering, and sound when the piece
  calls for it.
- Pieces form families: variations of earlier pieces, and named series that
  explore a rich vein.
- A personal toolkit and lexicon grow out of real repetition.
- Every hung piece keeps running and keeps rebuilding as the toolkit changes.

**Non-goals**

- Building or hosting `datspace.studio`. This system only produces deposits.
- Posting to Instagram, Vimeo, or YouTube. The system produces files cut for
  those formats; posting is manual for now.
- A general-purpose creative coding framework for others.

**Anti-goals** (failure even if everything technically works)

- **Cliché output.** Trippy, psychedelic, rainbow, screensaver, cheesy,
  amateur, low-brow. Jared wants serious work in the spirit of Ryoji Ikeda,
  Alva Noto, Curtis Roads, Mark Fell, Autechre, Raster-Noton, and Mille
  Plateaux. Note: the rainbow-dots spike sketch made during design is an
  example of what not to make.
- **Getting trapped in simple 2D.** 2D is welcome. It must not become the
  default by inertia.
- **Instrument-building in place of art.** A session that ends with a better
  library and no hung piece is a failed session, unless it was a planned
  tooling evening after a hang.
- **Abstraction ahead of the art.** No framework on day two.

## 3. The aesthetic

Jared's sensibility is already written down for his writing site:
`/Users/jaredmcfarland/Developer/writings/.claude/skills/patch-bay/aesthetic.md`
(and `briefs.md` beside it). Read it. Its one-line target there is "precision
with warmth." The core facts about Jared (musician first, his influences, film
post-production and color grading, label and catalog culture, plain English,
Tahoe, "art first, ship it") apply here unchanged.

Requirements:

- **Share the core; do not fork it.** The sketches system must draw on the same
  "who Jared is" material as patch-bay so the two cannot drift apart. It adds
  its own layer for image, motion, and sound. How to share it (a shared file
  both reference, a symlink, a small common skill) is the builder's call. Do not
  modify the writings repo without asking Jared.
- **The sketches layer** covers what the writing site does not: motion, time,
  grading moving images, 3D, shaders, sampling and processing source material,
  sound, and composition for a frame. It should state that monochrome and
  restraint are normal and color has to earn its place; that value comes before
  hue (colorist's order); that structure and system carry the work, not effects.
  It also needs an explicit list of clichés to avoid on sight (for example: hue
  cycling, rainbow palettes, kaleidoscopes, generic "audio-reactive" bars,
  bloom on everything, particle fountains, glow as a crutch). The list should
  grow from Jared's reactions.
- **Seeds come from the lineage.** Concepts and techniques, not "draw something
  cool": granular time, quantization, interference and beating, data as
  material, test signals, error as form, stochastic and Xenakis-style processes,
  grids and their violation, sampling a frame like sampling a sound. A bank of
  briefs, like patch-bay's `briefs.md`, is a good model.
- **A listening test for pieces**, adapted from patch-bay's: one idea carried all
  the way through; would it hold up next to a Raster-Noton release; is it
  finished; can its statement be said in one plain sentence.
- **Critique is part of the guide's job.** When Claude looks at frames it judges
  them against the aesthetic, names clichés out loud (including in its own
  suggestions), and says when something is working.
- **The document grows** as Jared reacts to pieces. A stated like or dislike
  goes into it, marked stated or inferred, the way patch-bay's file does.

## 4. The session

### The arc of an evening

About an hour, sometimes two, usually in the evening. Rough pacing **(Claude's
call)**:

| Phase | Time | What happens |
| --- | --- | --- |
| Arrive | ~5 min | `/begin`. Claude recalls recent pieces, open threads, the harvest list, and where the last session stopped. It offers a seed, or follows Jared's idea. |
| Explore and design | ~10–15 min | Discuss the idea and settle an approach. Get a first image on screen early instead of planning at length. |
| Make | ~30 min | Claude writes the code; Jared directs. Claude captures and looks at frames alongside him. |
| Frame and hang | ~10 min | Choose the loop or still, title it, write the statement, record, deposit. Build WASM if the piece is live. |
| Close | ~2 min | A note for next time; harvest candidates; any new lexicon word. |

The coach watches the clock. Around 45 minutes in it starts steering toward
framing, so the evening ends on a finished piece rather than a half-built one.
Two-hour evenings are where tooling happens, after the hang.

### The persona

`/begin` teaches Claude to be guide, thought partner, collaborator, mentor,
friend, and coach, and to read when to pull and when to follow. In practice:

- **Pull** when Jared is unsure, tired, or circling: offer a concrete seed, a
  constraint, a provocation, a reference, a technique he hasn't used.
- **Follow** when Jared arrives with an idea or is in flow: build what he
  describes, stay quiet, ask only what's needed.
- **Push past the first idea** when the first idea is the obvious one.
- **Coach** without nagging: name the time, name the drift toward tooling, help
  pick the frame, help write the statement. Celebrate a hang.
- Plain, direct English. No AI filler, no hype. Jared strips that from his own
  writing.

### The tired night

When Jared has about 20 minutes in him, the session starts from an existing
piece and explores variations. `/begin` should be able to offer "pieces with
room left in them" as a seed. This is also how series are born (section 7).

### Seeing the work

Claude must be able to see what it made, not just compile it. It runs the
sketch, captures frames at chosen times, and looks at them. To keep context
small it should look at a montage or contact sheet first and open single frames
only for detail (the same practice as patch-bay's `montage.png`). Jared watches
the live window at the same time.

## 5. The workspace

A new git repo at `/Users/jaredmcfarland/Developer/sketches`, a sibling of the
Nannou clone at `/Users/jaredmcfarland/Developer/nannou`.

**Pinning Nannou (Claude's call).** The clone's `master` tracks upstream, which
moves fast. Sketches must not build against whatever happens to be checked out.
Create a dedicated git worktree of the clone (for example
`/Users/jaredmcfarland/Developer/nannou-sketches` on a local `sketches` branch)
at a known commit, carrying the WASM fix from section 12. Pieces depend on it by
path. Jared's own clone stays free for following upstream. Moving sketches to a
newer Nannou is a deliberate step: rebase the branch, run the build check, fix
what breaks.

**Hung pieces are frozen (Claude's call).** Once hung, a piece keeps its own
code. When something graduates to `lib/`, old pieces keep their inline copies;
`lib/` serves new pieces only. The deposit is the archival record of how a
piece looks, so the build check only has to prove that every hung piece still
compiles, not that it renders identically.

Layout **(Claude's call; the builder may refine)**:

```
sketches/
  .claude/              skills, settings, CLAUDE.md for the practice
  docs/
    specs/              this document
    deposit-spec.md     the deposit contract (section 10)
    aesthetic.md        the sketches layer of the aesthetic (section 3)
    lexicon.md          the shared vocabulary (section 9)
  pieces/
    dss-0001-<slug>/    one crate per piece
  lib/                  the shared library crate(s), empty at first
  templates/            starting points for new pieces, minimal at first
  outbox/               default deposit location, git-ignored (section 10)

sketches-private/       private sibling (see "The repo is public" below)
  journal/              session notes, threads, harvest list (section 11)
  materials/            Jared's raw images, video, textures, audio (section 8)
```

- **One crate per piece, in one Cargo workspace.** Pieces are isolated, so a
  later change never breaks an earlier piece's code, and they share one
  `target/` so builds stay warm. Every hung piece must still build; the builder
  should provide a check that proves it.
- **Variations are full pieces** with their own folders, never branches or
  hidden parameters of another piece.
- **Fast iteration is a requirement.** The edit-to-image loop must be quick
  enough for directing in conversation. The spike measured a cold native build
  of Nannou at about 3.5 minutes and did not measure incremental rebuilds. The
  builder should measure, then choose the loop: rebuild-and-relaunch, Bevy's
  hot patching, a live-reloaded parameter file, or a combination. Keep
  dependencies optimized in dev builds (the spike used `opt-level = 3` for
  dependencies and `1` for the piece).
- **The repo is public (Jared's decision).** So personal material does not go
  in it: the session journal, open threads, Jared's raw reactions, and
  `materials/` (which may include footage or samples Jared doesn't own) live in
  a separate private location and are ignored by the public repo. A private
  sibling repo (for example `/Users/jaredmcfarland/Developer/sketches-private`)
  is **Claude's call**; the exact mechanism is the builder's. The lexicon, the
  sketches aesthetic layer, templates, `lib/`, and pieces are public. Creating
  the GitHub repo is an outward-facing step; confirm with Jared before doing it.
- Claude Code configuration is **project-local in this repo** rather than a
  separately installed plugin **(Claude's call)**. It can be promoted to a
  plugin later if it proves useful elsewhere.

### Skills

At minimum **(Claude's call on all but `/begin`)**:

- `/begin`: the session: persona, seeds, the arc, the coach, recall.
- `/hang`: framing, recording, metadata, deposit, optional WASM build, git
  commit of the piece. Mechanical and repeatable, so separate from `/begin`.
- `/harvest`: a tooling session that works from the harvest list (section 9).

The builder may add more if the work calls for them, but keep the surface small.

## 6. Media range

The toolkit should grow along all of these lines over time, so no single medium
becomes the default:

- 2D drawing (Nannou's `draw` API).
- 3D (Nannou on Bevy: cameras, meshes, lights).
- Shaders: Nannou's shader models, ISF, raw wgpu, compute.
- Images, video, and textures as input: loading, slicing, granulating,
  re-quantizing, feedback, and post-processing chains. Jared post-processes raw
  material the way he samples audio, and wants that on equal footing with
  synthesis and rendering.
- Sound (section 8).

Nothing here needs to be built upfront. It grows piece by piece under the
tooling rule.

## 7. Pieces, variations, series

- **A piece** has a catalog number, a slug, a title, a date, a statement, and a
  state (in progress or hung).
- **Catalog numbers** follow label culture: sequential, zero-padded, like
  releases. Prefix `DSS` (for datspace.studio) is **Claude's call** and easy to
  change before the first hang.
- **Numbers are assigned at hang time, never at creation (Claude's call)**, so
  the public catalog has no gaps. A piece in progress lives under a dated
  working name (for example `pieces/2026-10-23-<slug>/`) and is renamed to its
  catalog folder when hung. Unfinished pieces keep their working names, stay
  available as seeds, and are never deposited.
- **A variation** is a new piece forked from an existing one. It starts as a
  copy of the parent's code and records its parent.
- **A series** is a named group of pieces exploring one vein. Claude may notice
  one forming ("fourth variation on this; is it a series?"). Jared names it.
  A piece may belong to a series and have a parent at the same time.
- The shared core of a series is often the first good harvest candidate.

## 8. Sound and materials

**Sound** is part of the practice when a piece calls for it. It is not
required. The model is Ikeda and Alva Noto: audiovisual work where sound and
image come from the same source. Jared's principle from Curves: "the same math
is a sound, a shape, a motion, and a beat, and you just choose which face to
render."

- When a piece has sound, image and audio are driven by **one shared signal and
  one clock**, not synced after the fact.
- Live monitoring while working uses real-time audio (Nannou's `nannou_audio`,
  built on cpal).
- **Recording renders audio offline** by sample index from the same clock as
  the frame-locked video, then muxes it with the video. This gives exact sync.
  **(Claude's call.)**
- **Risk:** `nannou_audio` says nothing about the browser. Sound in live WASM
  pieces may need a separate Web Audio path. Investigate early; do not assume it.
  **If it doesn't work (Claude's call):** the live version of a sound piece
  ships silent, and the video master carries the sound. A browser audio path
  goes on the harvest list. It is tooling, so it waits for a hang like any other
  tooling.

**Materials** are Jared's raw images, video, textures, and audio, kept in
`materials/` and tagged so they can be found and reused like a sample library.
Large binary files probably should not live in git as-is; how to store and
track them (Git LFS, an ignored folder with a manifest, something else) is the
builder's call.

## 9. The toolkit and the lexicon

**The rule (Jared's): art first. Tooling is built only after the piece is hung.**

- While a piece is in progress, code the piece needs lives in the piece, even if
  it is rough or repeated. No refactors, no library work, no generalizing.
- When Claude sees something worth extracting, it adds it to the **harvest
  list** and keeps going.
- After the hang, on a longer evening or a dedicated tooling session
  (`/harvest`), the harvest list is the menu.
- Something graduates to `lib/` or `templates/` after it has appeared in two or
  three hung pieces **(Claude's call on the threshold)**.
- If Jared pulls toward tooling mid-piece, the coach names it once, kindly, and
  Jared decides.

**The lexicon** is a running document of words Jared and Claude coin ("the drift
field", "make it breathe") and what each means in code, usually pointing at a
library item. It is loaded into every session. When Jared uses a term, Claude
uses the lexicon's meaning. New terms are proposed at the close of a session
and added when Jared agrees.

**The first library item** is known already: **frame-locked time** (section
12). It is infrastructure every recording needs, so it is the one exception to
the "two or three pieces" threshold and should exist before the first hang.

## 10. The deposit

The system's job ends at the deposit. Everything behind it belongs to the site.
Jared was explicit: the site is a black box. A `datspace` repo already exists
next door with its own schema. Do not read from it or shape the contract around
it. Design the deposit contract on its own terms; the site adapts to it.

- A **written contract** at `docs/deposit-spec.md`, versioned, so the site can
  be built against it later by anyone, including another Claude session, with
  neither side knowing the other's internals.
- The **default destination** is a local `outbox/` folder. The destination must
  be configurable so it can later become a bucket or an upload endpoint without
  changing pieces.
- **One package per piece**, named by catalog number. Contents:
  - Metadata: catalog number, title, slug, date, statement, tags, series,
    parent, media list (file, kind, width, height, duration, fps, has audio),
    live flag, source reference (repo path and commit), schema version.
  - A still at full resolution.
  - A video master as the durable archive: a seamless loop or a fixed duration,
    with audio if the piece has sound. Choose a codec and quality that will
    hold up as an archive (builder's call; very high-quality H.264/H.265 or a
    ProRes master are both reasonable).
  - Social cuts for Instagram (feed 4:5, reels 9:16), Vimeo and YouTube (16:9).
  - If the piece is live: the WASM build and its HTML entry, compressed.
- **Aspect ratios:** a piece declares its native canvas. The master is recorded
  at that canvas. A social cut at another aspect is re-rendered from the code
  only if the piece supports it; otherwise it is matted on black. Naive
  cropping is not acceptable. **(Claude's call.)**
- **Output size must not depend on the display.** The spike's 720-point window
  captured at 1440×1440 because of the Retina display. Recordings need explicit
  pixel dimensions.
- Hanging also commits the piece to git and pushes it, so the source reference
  in the metadata is a public GitHub URL at that commit. Jared saying "hang it"
  is the approval for that push; nothing else pushes without asking.

## 11. Memory across sessions

The mentor needs continuity: what has been hung, which themes keep coming back,
which threads were left open, what is on the harvest list, what Jared reacted
to.

Keep this **as plain files** (session notes, open threads, the harvest list)
in the private location from section 5, plus the public aesthetic document and
lexicon, so it is versioned and readable by Jared **(Claude's call)**. `/begin` reads it at the
start, and the close writes to it. Keep what `/begin` loads small; summarize
old journal entries rather than loading them all.

## 12. Verified technical facts (from the spike)

A throwaway spike on 2026-10-22 proved the hard parts. Builders can rely on
these.

**Toolchain**

- Bevy 0.19 requires Rust 1.95 or newer. Rustup's stable toolchain was updated
  to 1.99.1. The `wasm32-unknown-unknown` target and `wasm-bindgen-cli` 0.2.125
  (matching Nannou's `Cargo.lock`) are installed.
- **Trap:** Homebrew's `rustc` 1.94 comes before `~/.cargo/bin` on `PATH`, so
  plain `cargo` fails to build Nannou. The sketches repo must pin rustup's
  toolchain (for example with `rust-toolchain.toml` and a `PATH` guard in the
  skills or a wrapper). Do not change Jared's global shell setup without asking.
- `ffmpeg` is installed (Homebrew).

**Nannou API facts**

- `nannou::sketch(..)` has no `add_plugin`; use `nannou::app(model)` when a
  Bevy plugin is needed.
- Nannou does not re-export the whole `bevy` crate. Depend on
  `bevy = { version = "0.19.0", default-features = false }` directly to reach
  things like `bevy::time::TimeUpdateStrategy`.
- `app.main_window().save_screenshot(path)` writes a PNG once the frame renders.
- `RunMode::loop_once()` / `loop_ntimes(n)` exist (new in this clone).
- `app.time()` is wall-clock elapsed time.

**Recording**

- Inserting `TimeUpdateStrategy::ManualDuration(1/60 s)` through a plugin makes
  `app.time()` advance exactly one frame per update, whatever the render speed.
- With that, 240 frames captured with `save_screenshot` and encoded with
  `ffmpeg` gave a seamless 4-second loop: the difference between the last and
  first frame matched every other consecutive pair, with no dropped or doubled
  frames. A 720×720 window captured at 1440×1440 on the Retina display.

**WASM**

- Works after three fixes:
  1. **Nannou bug:** `nannou/src/app.rs` line 311 has
     `resolution: (1024.0, 768.0).into()`, which does not compile on wasm32
     against Bevy 0.19. It must be `(1024, 768)` (Bevy 0.19's
     `WindowResolution` converts only from integers). **Claude's call:** carry the fix on a local
     branch of the Nannou clone. Whether to open an upstream PR is Jared's
     decision.
  2. Pieces must create their window with `.primary()`. Without it Nannou
     panics on wasm with "Non-primary windows are not supported on wasm."
  3. `getrandom` needs its browser backend: depend on `getrandom` 0.4 and 0.3
     with the `wasm_js` feature for the wasm32 target, and set
     `--cfg getrandom_backend="wasm_js"` in `.cargo/config.toml`.
- Build: `cargo build --target wasm32-unknown-unknown`, then
  `wasm-bindgen --target web`. It ran at 60 fps on WebGPU in headless Chrome
  and matched the native render.
- Size with a dedicated profile (`opt-level = "z"`, fat LTO, one codegen unit,
  `strip`, `panic = "abort"`): 19.5 MB raw, 5.8 MB gzip, 4.1 MB brotli. That
  build took about 4 minutes. Trimming unused Bevy features or `wasm-opt` may
  shrink it; not tried.
- Headless Chrome's `--screenshot` with virtual time shows a blank WebGPU
  canvas. Driving Chrome over the DevTools protocol in real time works.
- Jared said large WASM builds are acceptable. **Live is opt-in per piece.**
  The slow build happens only at hang time for pieces marked live.

**Prior art**

Almost nobody ships Nannou to the browser. Artists share videos and link source
(for example altunenes/rusty_art). Nannou's WASM tracking issue (#657) is still
open, and the community web templates target pre-Bevy Nannou. Live WASM pieces
from Bevy-based Nannou would be early.

## 13. Left to the builder

- Exact skill wording, file formats, and folder names beyond those above.
- How the aesthetic core is shared with patch-bay (without editing the writings
  repo unless Jared agrees).
- The iteration loop (hot patching, relaunch, live parameters).
- Codecs, bitrates, and exact social-cut resolutions.
- How the coach tracks session time.
- How materials are stored.
- How much of the first piece's scaffolding is a template on day one. Keep it
  minimal; templates are supposed to be earned.
- The order of work within the phases below.

## 14. Order of work

**The first evening is tonight (Jared's decision).** So the first build is the
thinnest path that lets Jared start, make, and hang a piece end to end. The rest
follows on later evenings. The phase split below is **Claude's call**.

**Phase 1: tonight, before the session**

- Toolchain pin and `PATH` guard; the pinned Nannou worktree with the fix.
- The Cargo workspace with warm dev-build settings, and a pre-built cache so
  the first compile of the night is not a 3-minute wait.
- Frame-locked time and frame capture, plus a montage so Claude can see frames.
- A fast enough iteration loop. The simplest one that works tonight;
  refinement can wait.
- `/begin` with the persona, the arc, the coach, the art-first rule, and seeds
  drawn from the lineage.
- A first version of the sketches aesthetic layer, linked to patch-bay's
  `aesthetic.md`, including the cliché list.
- `/hang`: still, video master, social cuts (matting on black is enough for
  now), metadata, `deposit-spec.md` v0, deposit to `outbox/`, commit and push.
- The private location with a journal and an empty harvest list, and an empty
  lexicon.
- The public GitHub repo, created with Jared's go-ahead.

**Phase 2: later evenings, as the work calls for it**

- Live WASM at hang time (proven in the spike; slow to build).
- Offline sound rendering and muxing, when the first sound piece appears.
- Re-rendered social cuts for pieces that support other aspects.
- `/harvest` and the build check over hung pieces.
- Materials library storage and tagging.
- Measuring and improving the iteration loop (hot patching, live parameters).
