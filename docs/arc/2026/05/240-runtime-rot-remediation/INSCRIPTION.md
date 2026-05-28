# Arc 240 — INSCRIPTION (runtime-rot remediation)

**Closed 2026-05-27.** Spawned from arc 239's STOP-and-report: the first full
`cargo test --workspace` (after 239 made the test-build compile) surfaced 69
runtime failures hidden behind the old `--lib`-only metric — ~6 root causes,
none from 237.7a. Arc 240 fixed the consumer-`.wat` drift + the clean substrate
gaps, and attributed every non-drift failure to the open arc that owns it.

## What shipped

- **240.1** (`2fdd0f6f`) — substrate gaps, check-side, `src/check.rs` only:
  - **B**: `:wat::core::first`/`second`/`third` gained a `wat::core::List` arm
    (→ `Option<T>`); the substrate taught us the briefed one-arm fix was
    incomplete — `List/of` had no TypeScheme (returned a fresh var), so it was
    completed with `List/of` constructor inference + `List/conj` + a `rest` arm.
    Runtime already handled all of these.
  - **C**: `:wat::holon::Bundle` `other`-branch now `reduce`s the inferred type,
    so the `:wat::holon::Holons` alias (= `Vector<HolonAST>`) unfolds structurally.
- **240.2** (`0b786b7b`) — stale-test `probe_arc216_stone5c::probe_12`: replaced
  the `Bundle/children` assertion (invalid post arc-228 classifier-wrap) with a
  `to-holon → from-holon` round-trip, mirroring stone4's precedent.
- **240.3a** (`7d5fbcbd`) — `WorkUnitLog.wat` recipe exemplar, FM-2-bis proven.
- **240.3b** (`97c6ace8`) — consumer-`.wat` drift sweep. **wat-telemetry 36/0**;
  wat-telemetry-sqlite drift cleared (5/6; the 6th-group blocked on a non-drift
  bug, scoped out below). The **FOUR-element recipe** (arc 215/225/230 drift):
  1. `(:wat::holon::Atom <value>)` → `(:wat::holon::to-holon <value>)`
  2. `(:wat::holon::Atom <watast>)` → `(:wat::holon::from-wat <watast>)`
  3. `(:wat::core::atom-value <h>)` → `(:wat::holon::from-holon <h>)`
  4. `(:wat::core::HashMap :Tag)` (1-arg tuple-alias) → 2 separate type args.

## Out of arc 240's scope — tracked + marked on owning open arcs

These are NOT deferrals-to-the-void; each is affirmatively owned by an OPEN arc
that is actively reworking the code in question, with a KNOWN-BROKEN marker:

- **lru / holon-lru `HolonKey.wat`** (3 tests) — the lru wat-tests are arc 119's
  in-flight `#208` consumer-sweep; arc 130 (`#226`) is actively reshaping the
  `:wat::lru::LocalCache` substrate. Tracked: arc 119 + arc 130 KNOWN-BROKEN.
- **wat-cli fork/exit + ambient-stdio** (~3 + cli residual) — the spawn/fork/
  stdio machinery arc 170 is actively reshaping. Tracked: arc 170 KNOWN-BROKEN.
- **lifeline 1/100 orphan** — the `spawn_lifelined`/Pidfd primitive, arc 213
  (`#373` pidfd cascade). Tracked: arc 213 KNOWN-BROKEN.
- **sqlite log-daemon cursor `::`-keyword decode** (6 `reader` tests) — the sqlite
  log sink is the auto-spawned `Service` daemon in arc 170's spawn/Service rework
  scope (user direction 2026-05-27). The specific defect is decode-side
  (`decode_notag_holon`); fix it when correcting the daemon, or re-home to arc
  219b if 170's rework doesn't touch row-decode. Tracked: arc 170 KNOWN-BROKEN +
  FINDING-sqlite-reader-bugs.md.

## Green-gate (momentary restriction)

The routine gate is now `cargo test --lib` + `cargo build --tests --workspace`
(compile-only). The full `cargo test --workspace` RUN is held off — it leaks
processes (ambient-stdio/fork/lifeline) arc 170 fixes — **until 170 closes the
leaks, at which point the full integ suite returns as a gate.** Memory:
`feedback_green_gate_lib_and_build`. The `--tests --workspace` build gate itself
ships as arc 239's `#566` closure (the visibility-gap class-fix), not here.

## Verification (FM-9, independent re-run)

`cargo test --lib -p wat` ≥834/0 · `wat_arc220_list` 23/0 · `wat_bundle_capacity`
7/0 · `probe_arc216_stone5c` 12/0 · `wat-telemetry` 36/0 · `cargo build --tests
--workspace` 0 errors. Scope held: `src/check.rs` + tests/`.wat` only; no
holon-rs; no namespace renames.

## Doctrine carried forward

- The FM-2-bis recipe-proof on WorkUnitLog grew the recipe from 2 → 4 elements
  (the WatAST `from-wat` + `atom-value→from-holon` sites a uniform sweep would
  have broken) — earning the right to brief the bulk sweep.
- arc 109 NOTE filed: reconsider `atomize`/`materialize` vs the
  `to-holon`/`from-holon`/`from-wat` family (challenge intueri later).

Cross-ref: DESIGN.md (the ledger); arc 239 (parent; span-rot compile sweep).
