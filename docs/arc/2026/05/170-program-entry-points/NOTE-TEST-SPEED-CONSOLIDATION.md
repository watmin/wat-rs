# NOTE — test-speed consolidation: the 170-CLOSURE deliverable (DEFERRED — tackle hard, not piecemeal)

**Status: CAPTURED 2026-06-25 (during the arc-291 marathon).** Builder: *"the tackle is 170's closure — don't
pollute the docs/ root… i want to tackle this hard but it's not a thing we can focus on now."* The test build/run
is slow (~3.5 min full clean build, builder's ref; he tabs away while it grinds). This is the corrected home —
the **probe→home consolidation is gated by 170 AND proves 170's closure**, so it belongs to 170, not a standalone
root doc. Pairs `FOLLOWUPS-TEST-BINARY-LEAK.md` (the leak forensics) + `SCORE-SLICE-D-LEAK-ZERO-VERIFICATION.md`.
This note holds the whole plan ready so the focused run re-derives nothing.

## The mechanism (grounded 2026-06-24)
Every top-level `tests/*.rs` is its own crate = its own binary = its own LINK step. We have **250** of them
(**166 `probe_*.rs`**), each statically linking the entire `wat` crate (32k-line `runtime.rs`, 21k `check.rs`).
We're not compiling tests 250× — we're **linking the monolith 250×.** The lib *compile* is link-tool-immune
(same every build); the **link** is the lever.

## Lever 3 — probe→home consolidation (THE 170-CLOSURE PIECE; the reason this note lives here)
Collapse the **166 loose `probe_*.rs`** into home-grouped binaries (mirroring the proven `tests/comms/`,
`tests/collection/`, `tests/macros/` dirs — already module-grouped, NOT separate binaries). Kills ~165 redundant
link steps AND completes the homes migration (the loose probes are the unmigrated tail — speed win = migration
completion, one move).

**⛔ HARD-BLOCKED on 170 (the execve global leak).** Today separate binaries = separate **processes** = the leak
stays quarantined per file. One home-grouped binary = one process's shared globals → the leak surfaces as
cross-test interference and flake. **So annihilating the execve leak (170's core) is the structural prerequisite.**

**Why it's 170's CLOSURE, not just blocked-on-170:** consolidation-staying-green is the **strongest possible proof
170 actually closed the leak.** Per-binary isolation *masks* a residual leak (fresh process per test); one process
exposes it loudly. So the re-org is a harder stability gate than the soak (#207) — green-in-one-binary == the leak
is genuinely dead, not hidden. It IS the closure verification. `substrate-forces-idealized-state`, on the harness.

## Levers 1+2 — mold + debuginfo (INDEPENDENT quick wins; NOT 170-bound; do anytime a build is between strikes)
Pure link-speed, zero isolation change, zero risk — available now, no 170 dependency. Deferred only because the
builder wants the test-speed work tackled as one focused effort, not piecemealed mid-arc.
- **mold** (`2.30.0` installed at `/usr/bin/mold`; target `x86_64-unknown-linux-gnu`; `gcc`/`cc` present). Append
  to `.cargo/config.toml` (do NOT clobber `[env] RUST_MIN_STACK="8388608"`):
  ```toml
  [target.x86_64-unknown-linux-gnu]
  rustflags = ["-C", "link-arg=-fuse-ld=mold"]
  ```
  Fallback if the gcc-driver path balks: `["-C","linker=clang","-C","link-arg=-fuse-ld=mold"]` or `mold -run cargo …`.
- **debuginfo trim** (helps the DEV full gate only — `--release` already has `debug=0`). Workspace-root `Cargo.toml`:
  ```toml
  [profile.dev]
  debug = "line-tables-only"   # NOT 0 — keeps line-numbered panic backtraces we lean on
  ```
- **Prove it, don't assert it:** baseline `cargo clean -p wat && time cargo test --release -p wat --no-run`
  (~3.5 min ref) → apply → re-run → the delta is the win. (The lib *compile* is identical in both runs, so the
  delta is pure link savings.) Own atomic commit.

## Sequencing
- **Levers 1+2 (mold + debuginfo):** one timed commit, whenever a build sits between strikes (verification needs a
  clean build that can't race a running suite — so it's the next keystroke on a stable tree, never mid-sonnet).
- **Lever 3 (consolidation):** part of 170's CLOSURE — both the big speed win and the leak-proof + migration completion.

## Pairs
`FOLLOWUPS-TEST-BINARY-LEAK.md` (the leak forensics) · `SCORE-SLICE-D-LEAK-ZERO-VERIFICATION.md` (leak-zero verify) ·
#207 (stability-100 soak — consolidation is a stronger gate) · `project_test_floor_is_execve_global_leak` (memory) ·
the homes migration (`VIGILATUM.md` / `OP-PLACEMENT.md`).
