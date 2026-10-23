# Lexicon

Words Jared and Claude coin while working, and what each one means in code. This
file is loaded at the start of every session. When Jared uses a term from it,
Claude uses the meaning written here, not a guess.

New terms are proposed at the close of a session and added only when Jared says
yes. A term earns an entry when it was used more than once, or when it named
something precise that would otherwise take a sentence to say.

## Format

Each entry is a third-level heading with the term, followed by:

- **Means:** what it is, in plain words.
- **Code:** what it points to: a function in `lib/`, a pattern in a named
  piece, or "no code yet."
- **Coined:** the date, and the piece it came from.

Example of the shape, not a real entry:

```
### make it breathe

- **Means:** a slow, symmetric swell in a single value (scale, brightness, or
  density) on the loop's period, so the piece inhales and exhales once per loop.
- **Code:** no code yet. First used in `pieces/2026-10-23-tide/src/main.rs`,
  `breath(phase)`.
- **Coined:** 2026-10-23, from `tide`.
```

When a term's code graduates to `lib/`, update its **Code** line.

## Terms

(None yet.)
