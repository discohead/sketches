# Deposit spec

Schema version: `dss-deposit/0`

A deposit is the package `studio hang` produces for one finished piece. It is
the only thing this repo hands to the website. The site reads deposits; it never
reads the repo, and this repo never reads the site. If you are building the site,
this document is the whole contract.

## Where deposits go

Deposits are written under the deposit directory set in `studio.toml`:

```toml
[deposit]
dir = "outbox"
```

The default is `outbox/` at the repo root. It is not checked into git. Later the
destination may become a bucket or an upload endpoint; the package itself does not
change.

## The package

One directory per piece, named by its catalog number:

```
outbox/
  DSS0001/
    piece.json       metadata, described below
    still.png        one frame at full canvas size
    master.mp4       archival video at canvas size
    loop.mp4         web video at canvas size
    feed-4x5.mp4     Instagram feed cut, 1080×1350
    reel-9x16.mp4    Instagram Reels cut, 1080×1920
    wide-16x9.mp4    Vimeo and YouTube cut, 1920×1080
```

Catalog numbers are the prefix `DSS` plus four digits: `DSS0001`, `DSS0002`, and
so on. Numbers are given out in order when a piece is hung and are never reused.

Treat `piece.json`'s `media` list as the source of truth for which files exist.
Do not assume the list above is complete in later schema versions.

## The media

Every video is H.264 in an MP4 container, converted from RGB with the BT.709
matrix to limited ("TV") range, and tagged BT.709 for primaries, transfer and
matrix. Play it as ordinary HD video; no colour management is needed.

- **still.png**: one frame, chosen by the piece (`still_at` seconds in), at the
  native canvas size.
- **master.mp4**: the archive copy. H.264 (libx264, `-preset slow -crf 10`),
  4:4:4 chroma, no audio in this schema version. Native canvas size. Plays the
  piece once: a loop that repeats seamlessly when played back to back, or the
  full fixed duration.
- **loop.mp4**: the copy to show on the web. H.264 (`-crf 18`), 4:2:0 chroma,
  `faststart` so it begins playing before it finishes downloading. Native canvas
  size, same length as the master. It is named `loop.mp4` even for a
  fixed-duration piece.
- **Social cuts**: the same settings as `loop.mp4`, at a fixed size. The piece is
  scaled to fit inside the frame without cropping, centred, and matted on black.
  A looping piece is repeated until the cut lasts at least 15 seconds (an 8-second
  loop plays twice, 16 seconds in all). A fixed-duration piece plays once.

## piece.json

UTF-8 JSON. Every top-level key is always present; where a value can be absent,
it is `null`, never a missing key. Inside `media`, the fields depend on the
entry's `kind`: a still has no `seconds`, `fps` or `audio`, and only social cuts
have `aspect`. The tables below say which fields each kind has.

```json
{
  "schema": "dss-deposit/0",
  "catalog": "DSS0001",
  "title": "Drift Field",
  "slug": "drift-field",
  "date": "2026-10-23",
  "statement": "Points drift and settle.",
  "tags": ["field", "particles"],
  "series": null,
  "parent": null,
  "source": {
    "repo": "https://github.com/discohead/sketches",
    "commit": "3f1c2e9…",
    "path": "pieces/dss0001-drift-field"
  },
  "canvas": { "width": 1080, "height": 1080 },
  "time": { "fps": 60, "form": "loop", "seconds": 8.0 },
  "live": false,
  "audio": false,
  "media": [
    { "file": "still.png", "kind": "still", "width": 1080, "height": 1080 },
    { "file": "master.mp4", "kind": "master", "width": 1080, "height": 1080, "seconds": 8.0, "fps": 60, "audio": false },
    { "file": "loop.mp4", "kind": "web", "width": 1080, "height": 1080, "seconds": 8.0, "fps": 60, "audio": false },
    { "file": "feed-4x5.mp4", "kind": "social", "aspect": "4:5", "width": 1080, "height": 1350, "seconds": 16.0, "fps": 60, "audio": false },
    { "file": "reel-9x16.mp4", "kind": "social", "aspect": "9:16", "width": 1080, "height": 1920, "seconds": 16.0, "fps": 60, "audio": false },
    { "file": "wide-16x9.mp4", "kind": "social", "aspect": "16:9", "width": 1920, "height": 1080, "seconds": 16.0, "fps": 60, "audio": false }
  ]
}
```

### Top-level fields

| Field | Type | Meaning |
|---|---|---|
| `schema` | string | The schema version, `dss-deposit/<n>`. Check it before reading anything else. |
| `catalog` | string | The catalog number, `DSS` plus four digits. Same as the directory name. |
| `title` | string | The piece's title. Never empty. |
| `slug` | string | Lowercase letters and digits joined by single hyphens, at most 40 characters. Safe for URLs. Unique among hung pieces in practice, but use `catalog` as the key. |
| `date` | string | The day the piece was hung, `YYYY-MM-DD`. |
| `statement` | string | A short artist's statement. Never empty. May contain line breaks. |
| `tags` | array of strings | Free-form tags. May be empty. |
| `series` | string or `null` | The name of the series the piece belongs to, or `null`. |
| `parent` | string or `null` | The catalog number of the piece this one was forked from, or `null` if it has no parent or the parent was never hung. |
| `source` | object | Where the code lives. See below. |
| `canvas` | object | The native size in pixels: `width` and `height`, both integers. |
| `time` | object | How the piece runs in time. See below. |
| `live` | boolean | Whether a live, in-browser build is included. Always `false` in this version. |
| `audio` | boolean | Whether the piece has sound. Always `false` in this version. |
| `media` | array | The files in the package. See below. |

### `source`

| Field | Type | Meaning |
|---|---|---|
| `repo` | string | The public repo, normally `https://github.com/<owner>/<repo>`. |
| `commit` | string | The full git commit sha that contains the piece as hung. |
| `path` | string | The piece's folder inside the repo, `pieces/<catalog in lower case>-<slug>`. |

`<repo>/tree/<commit>/<path>` is a link to the exact code that made the piece.

### `time`

| Field | Type | Meaning |
|---|---|---|
| `fps` | integer | Frames per second of every video in the package. |
| `form` | string | `"loop"`: the piece repeats seamlessly, so play the videos on loop. `"duration"`: the piece has a beginning and an end. |
| `seconds` | number | The length of one pass: one loop, or the whole duration. |

### `media` entries

Every entry has these fields:

| Field | Type | Meaning |
|---|---|---|
| `file` | string | The file name inside the package directory. |
| `kind` | string | `"still"`, `"master"`, `"web"` or `"social"`. |
| `width` | integer | Width in pixels. |
| `height` | integer | Height in pixels. |

Video entries (`master`, `web`, `social`) also have:

| Field | Type | Meaning |
|---|---|---|
| `seconds` | number | Length of this file: its frame count divided by `fps`, rounded to 3 decimals. This can differ from `time.seconds` by part of a frame. For social cuts it is the repeated length, so it can be longer than `time.seconds`. |
| `fps` | integer | Frames per second. |
| `audio` | boolean | Whether the file has a sound track. |

Social entries also have:

| Field | Type | Meaning |
|---|---|---|
| `aspect` | string | The aspect ratio as `"<w>:<h>"`: `"4:5"`, `"9:16"` or `"16:9"`. |

Entries appear in this order: still, master, web, then the social cuts. A reader
should find a file by `kind` and `aspect`, not by its position.

## Versioning

The `schema` value changes whenever a change could break a reader that follows
this document. That includes removing or renaming a field, changing a field's
type or meaning, changing a file's name, or allowing `null` where it was not
allowed before. Each such change bumps the number: `dss-deposit/0` becomes
`dss-deposit/1`.

Adding a new field, a new `kind`, or a new file does not bump the version. Readers
must ignore fields and media they do not know.

A deposit keeps the schema version it was written with. Old deposits are not
rewritten when the version changes, so a site should either support every version
it finds or skip ones it does not know.

## Atomicity

A deposit appears all at once. `studio hang` writes every file into a hidden
directory, `.DSS0001.partial/`, and renames it to `DSS0001/` only after every file
is complete and `piece.json` is written last. A directory whose name is a catalog
number is always complete. Ignore anything whose name starts with a dot.

A failed hang leaves a `.partial` directory behind, and so does a hang run with
`--no-push`, which commits without pushing and waits to be re-run. Re-running the
hang replaces it.

## Immutability

A deposit directory is never changed after it appears. `studio hang` refuses to
run if the deposit already exists. To change a hung piece, make a new piece
(usually forked from the old one, which records it as the `parent`) and hang that
under the next number.
