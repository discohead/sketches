# sketches

A daily creative-coding practice repo: small generative pieces written in Rust with Nannou on Bevy 0.19. Each piece lives in its own folder under `pieces/`, and the `studio` tool creates, runs, captures and hangs them. Journal and working notes are kept in a separate private repo.

Building needs the `nannou-sketches` worktree next to this repo (`../nannou-sketches`, branch `sketches`, based on Nannou commit `bffa9887` plus one fix: `WindowResolution` takes integers in Bevy 0.19). Run cargo through `scripts/cargo`, which uses the pinned rustup toolchain.
