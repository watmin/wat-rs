# SCORE — Stone 254.T (build-tooling): build.rs test-mod auto-discovery

**A tooling stone off the comms-integration spine, retrofitted into the build
log.** Landed during arc 254 (the active arc) as a pre-stone that de-risks the
make-channel cascade: with the mod-list auto-generated, the cascade can add/remove
test files freely with zero manual step to forget mid-strike.

**SHIPPED** — `f06f9228` (pushed). Scored against my own build + discovery run.

## The trigger

A freshly-added probe (`probe_arc254_make_channel.rs`) wasn't found by `cargo
test` — `running 0 tests` — until I manually ran `gen-test-mods.sh`. The grouped
integration-test mod lists were hand-maintained by a bash script you had to
*remember* to run; the `--check` green-gate caught drift only at gate time,
leaving a dev-loop window where a new test was silently un-compiled. The builder:
*"how do we make that gen-tests thing ALWAYS run? something that can never be
forgotten? i want to exist in the cargo ecosystem completely."*

## ⚰️ Ripped from existence

- `scripts/gen-test-mods.sh` — the hand-run generator (deleted).
- The `// BEGIN/END GENERATED MODS` committed mod-lists in all 6 group `mod.rs`
  (`collection`, `comms`, `function`, `macros`, `nursery`, `types`) — replaced by
  one-line `include!` stubs.
- green-gate's `--check` gate (was 1/4) — drift is now *unrepresentable*, so there
  is nothing to check; green-gate is 3 gates (gate 1 `cargo build --tests` runs
  build.rs, so auto-discovery is inherently exercised).

## 🔥 Raised

- **`build.rs`** (pure Rust, cargo-native): auto-scans every `tests/<group>/` with
  a `mod.rs`, generates its module list into `OUT_DIR` on **every** build (absolute
  `#[path]` for robust resolution regardless of `include!` site;
  `rerun-if-changed` per group dir). Drop a `.rs` into a group → compiled + run on
  the next `cargo test`. The failure class (a silently-forgotten test) is
  annihilated by construction, not guarded by a gate.
- `tests/nursery/probe_build_rs_autodiscovery.rs` — the FM-2-bis: an empty-bodied
  test (no-panic-IS-the-proof, named honestly — not a sentinel) that was never
  added to any mod list. Its appearing in the run report is the proof.

## Why OUT_DIR, not in-place rewrite (the rejected grok shape)

A `build.rs` that rewrites tracked source (`grok`'s `sh ./prebuild.sh` form)
dirties git every time a file is added/removed AND can need **two builds to
settle** — cargo fingerprints `mod.rs` before `build.rs` runs — so it might not
even reliably close the window it exists to close. Generating into `OUT_DIR`
(per-build, gitignored) dodges both: no source mutation, no fingerprint lag, and
no committed mod-list to drift (✅✅✅).

## Verification (PASS)

| # | check | result |
|---|---|---|
| 1 | all 6 group test binaries compile via include!/OUT_DIR | **PASS** (`cargo build --release --tests -p wat`, exit 0) |
| 2 | `super::` cross-ref survives (`function/stone18a_errors` → `super::stone18a::try_startup`) | **PASS** (sibling mods declared at root; resolution unchanged) |
| 3 | new file auto-discovered, zero manual step | **PASS** (`probe_build_rs_autodiscovery` ran + passed; make-channel probe's 3 tests listed) |
| 4 | discovery parity | **PASS** (834 nursery tests discovered) |
| 5 | no stray `gen-test-mods` refs in live docs | **PASS** (INTENTIONS.md updated; only historical CLIFFNOTES blocks + green-gate's own history note remain) |

## Forward path — wat hosts its own toolchain

This is the *always-runs floor* for the named benchmark (`docs/INTENTIONS.md` §"The
benchmark not yet passed"). The wat migration follows: once wat has fs syscalls
(`readdir`), the list-computation moves to a wat program run as a separately
distributed, prebuilt **stage0 wat binary** used to bootstrap the build (the rustc
pattern) — NOT called from within this crate's own `build.rs` (circular: the binary
is what's being built). Builder's decision, recorded. Gated on wat fs syscalls.
