# CLAUDE.md — wat-rs

Conventions specific to working in `wat-rs` (the Rust-hosted `wat` language). Loaded in addition
to the parent `holon/CLAUDE.md`.

## Scratch `.wat` files → `wat-scripts/scratch-pad/`, NOT the session scratchpad

Throwaway / reconnaissance / scratch **`.wat`** programs go in **`wat-scripts/scratch-pad/**/*.wat`**,
NOT the ephemeral session scratchpad (`/tmp/.../scratchpad`). This OVERRIDES the default
"temp files → session scratchpad" for `.wat` specifically.

Rationale: a scratch `.wat` is a durable, loadable reference, and the
`every_wat_scripts_file_loads` gate (`tests/lint/wat_scripts_fixes_load.rs`) parses +
type-checks **every** `.wat` under `wat-scripts/` (recursively, incl. `scratch-pad/`) on the
current runtime — so a scratch program that rots goes RED and cannot become a graveyard that
reads like live code. All wat stays correct, always. Scratch here therefore obeys the current
substrate rules (delete it if it's truly dead; otherwise it conforms). Non-`.wat` temp files
(logs, patches, data) still use the session scratchpad.

## The test floor is weighed in RELEASE

The zero-failure floor is **`cargo nextest run --release`** (~4189/0). Plain `cargo nextest run`
(debug) surfaces `debug_assert!`s and timing flakes (double-fork service tests, `sigterm`,
`pdeathsig`, `lifeline_orphan`) that are NOT release failures. Read the Summary line — never a
piped exit code (`cargo nextest ... | tail` returns `tail`'s exit, not nextest's). A green→red
flip between two runs, or a `debug_assert!` panic, is a mode/timing signal first, not a regression.
