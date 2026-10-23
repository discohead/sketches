# Build times

Measured 2026-10-22 on Jared's machine with `studio new smoke` and `scripts/cargo build -p wip-smoke` (dev profile, shared `target/`).

| Build | Time |
| --- | --- |
| First build of a new piece, with Nannou/Bevy dependencies compiling into the shared target dir (mostly cached from `kit`) | 1 m 48 s (that run failed at the piece itself, so the dependencies were compiled and the piece's own compile was not counted) |
| Piece compile after the dependencies were built (first successful build) | 3.2 s |
| Incremental rebuild after `touch src/main.rs` | 0.9 s |

A new piece costs a few seconds to build once Nannou and Bevy are compiled. Editing a piece rebuilds in about a second.
