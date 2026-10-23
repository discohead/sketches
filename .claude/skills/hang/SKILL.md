---
name: hang
description: Frame, title, record and deposit a finished piece in the sketches repo. Chooses the loop or duration and the still with studio look, writes the title and one-sentence statement with Jared, runs the listening test, then runs studio hang, which numbers, commits, pushes and deposits the piece. Use when Jared types /hang, says "hang it", "let's frame it", "ship it", "it's done", or when /begin steers a session toward finishing.
---

# Hang

Hanging finishes a piece: it gets its frame, its title, its statement, a catalog
number, a recording, and a public commit. This is the moment the practice
exists for. Do it carefully and without ceremony.

Read `CLAUDE.md` if you haven't this session. You need the piece's name (its
folder under `pieces/` or its slug). If more than one piece is in progress,
ask which.

Keep Jared in the loop at each step, in a line or two. He decides the title,
the statement and the frame; you propose.

## 1. Frame it

Stop the live run first, with the command in `CLAUDE.md`:
`for p in $(pgrep -f 'target/debug/studio run '); do pkill -P "$p"; done`. It
touches only `studio run` windows. Never stop pieces any broader way while a
`look` or `hang` is running.

Read `pieces/<folder>/piece.toml` and decide the `[time]` table with Jared:

- **`form`**: `"loop"` if the piece repeats seamlessly, `"duration"` if it has
  a beginning and an end, or it doesn't repeat.
- **`seconds`**: the loop length, or the duration. For a loop, every periodic
  motion must complete a whole number of cycles in it. Short loops (4–12 s)
  are normal; social cuts repeat a loop to reach 15 s.
- **`still_at`**: the moment that stands for the piece as a single image. It
  must be no later than `seconds`.

Use `scripts/studio look` to choose:

- `scripts/studio look <piece> --count 8` for an overview. Read the montage.
- `scripts/studio look <piece> --at <t1>,<t2>,<t3>` around candidate stills.
  Pick the frame with the clearest value structure and the idea most visible.
- For a loop, check the seam: `--at 0,<seconds>` must give identical frames.
  If they differ, the loop is not seamless; fix it in the piece before going
  on (a frequency that isn't a whole number of cycles is the usual cause).

## 2. Title and statement

Write them with Jared, in plain English.

- **Title.** Short. Label culture, not poetry: a noun, a technique, a place, a
  number. Offer two or three options at most, then take his. Lowercase is
  fine if he likes it.
- **Statement.** One plain sentence that says what the piece is or does. No
  adjectives doing the work ("mesmerizing", "hypnotic", "exploring"). Good:
  "Two line grids, one turning a fraction of a degree, beat against each
  other." Bad: "A hypnotic exploration of interference and perception."
- **Tags.** Three to five lowercase words: the medium and the technique
  (`2d`, `shader`, `moire`, `polyrhythm`). Plain, findable.
- **Series.** If the piece belongs to one Jared has named, set `series`.

## 3. The listening test

Ask the questions in `docs/aesthetic.md` ("The listening test") and answer them
honestly, in a few lines:

1. One idea, carried all the way through?
2. Would it hold up next to a Raster-Noton or Mille Plateaux release?
3. Finished: seamless loop, clean edges, nothing placeholder?
4. Does it hold as a still and in motion?
5. Anything from the cliché list? If so, is it the subject?
6. Can the statement be said in one plain sentence?

If an answer is no, say which and what would fix it. Then let Jared choose: fix
it now if it's quick, or hang it as it is. Hanging an imperfect piece is fine.
Not hanging is the failure this practice exists to prevent.

## 4. Update `piece.toml`

Edit `title`, `statement`, `tags`, `series`, and the `[time]` table. Leave
`catalog` alone; `hang` sets it. Check that `audio = false` and `live = false`:
both are refused in Phase 1. A piece that played sound live hangs silent.

Show Jared the final values in a short block.

## 5. Ask

Ask: "Hang it?" His yes is the approval to commit and push the piece publicly.
Nothing else in the practice pushes. If he says no or not yet, go back to
whichever step he wants.

## 6. Run `studio hang`

```sh
scripts/studio hang <piece>
```

It renders every frame and encodes several videos. Capture costs about 5 s per
second of piece, so an 8 s loop takes a minute or two. For pieces up to about
20 s, run it in the foreground with a 10-minute timeout. For longer pieces, run
it as a background command and wait for it to finish (a 60 s piece spends about
300 s on capture alone). It refuses an unready piece and lists
the reasons. Otherwise it:

1. Assigns the next catalog number (DSS0001, …).
2. Renders every frame and encodes the still, the master, the web loop, and the
   social cuts.
3. Sets `catalog`, renames the folder to `pieces/dss####-<slug>/` and the
   package to `dss####-<slug>`, and checks that it builds.
4. Commits the piece and pushes it.
5. Writes `piece.json` and moves the finished package to `outbox/DSS####/`.

If it fails, show Jared the error in a line, fix the cause, and run it again.
It resumes where it stopped. After the rename, use the catalog number:
`scripts/studio hang DSS0001`.

- **A readiness error** (empty title or statement, `audio` or `live` set,
  `still_at` past the end): fix `piece.toml` and re-run.
- **No `origin` remote**: `hang` checks for it before rendering, because the
  deposit's source link needs it, and `--no-push` does not get around it. Stop
  and tell Jared. Creating the GitHub repo is a separate step (Task 8) that
  needs his go-ahead; don't create it or add a remote yourself.
- **"push failed"**: the commit exists locally. Show the error. If the cause
  is something Jared has to do (authentication, no network), tell him, and
  re-run once it's fixed.
- **The deposit already exists**: never overwrite or delete a deposit. Tell
  Jared.
- **A render or encode error**: read it, fix the piece if the piece is the
  cause, and re-run.

Use `--no-push` only if Jared asks. Then the commit stays local and the
deposit's source link won't resolve until it is pushed; say so.

After a successful hang the piece is frozen. Never edit it again. Variations
start with `scripts/studio new <slug> --from DSS####`.

## 7. Report the deposit

`studio hang` prints the deposit folder as its last line. List it
(`ls -la outbox/DSS####/`) and tell Jared what is in
it, briefly: `piece.json`, `still.png`, `master.mp4`, `loop.mp4`, and the
social cuts `feed-4x5.mp4`, `reel-9x16.mp4`, `wide-16x9.mp4`. Read `still.png`
once to confirm it is the frame you chose. Give the catalog number, the title,
and the commit.

## 8. Write it down

Add a line to today's session note in
`/Users/jaredmcfarland/Developer/sketches-private/journal/sessions/<YYYY-MM-DD>.md`:

```markdown
- Hung DSS0003 *moire* (pieces/dss0003-moire), outbox/DSS0003. Took the seed / Jared's idea.
```

If the piece came from a thread in `journal/threads.md`, remove or update that
thread.

## 9. Mark the moment

One or two plain lines. It's a release on the label: say its number and what it
is. If it's a milestone (the first piece, the tenth, the first of a series),
say that. No hype, no exclamation marks.

> DSS0003 is hung: *moire*, two grids beating. Third in the catalog.

## 10. Then

If time remains, the harvest list is open: this is when tooling is allowed.
Offer the top item from the private `journal/harvest.md`, or a variation of the
piece just hung, or the close. If the session is ending, go back to `/begin`'s
close step.
