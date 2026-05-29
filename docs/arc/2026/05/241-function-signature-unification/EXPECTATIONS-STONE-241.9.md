# EXPECTATIONS — Stone 241.9 — defenum HARD CUT

Independent scorecard. No vigilia (legacy flat substrate per D6). SCORE-green commit. Upper bound 150 min — slightly above 241.8's 120 to absorb the `parse_field` retirement audit.

## Phase A — Scorecard (11 rows)

| Row | Claim | Verification | Expected |
|---|---|---|---|
| 1 | Probe contracts 01-04 PASS (defenum success paths) | `cargo test ... contract_0[1-4]` | 4/0 |
| 2 | Probe contract 05 PASS (empty `{}` metadata rejected) | `cargo test ... contract_05` | 1/0 |
| 3 | Probe contracts 06-07 PASS (legacy enum HARD CUT rejected, unit + tagged forms) | `cargo test ... contract_0[67]` | 2/0 |
| 4 | Probe contract 08 PASS (defenum registers usable variant constructors) | `cargo test ... contract_08` | 1/0 |
| 5 | Probe whole-suite 8/8 | `cargo test --release --test probe_arc241_stone9_defenum` | 8/0 |
| 6 | Stone 241.8 probe preserved 8/8 | `cargo test --release --test probe_arc241_stone8_defstruct` | 8/0 |
| 7 | Stone 241.7 probe preserved | `cargo test --release --test probe_arc241_stone7_metadata_of_reflection` | counts preserved |
| 8 | Stone 241.1-241.6 probes + arc 237/238 probes preserved | each | counts preserved |
| 9 | Lib baseline (post-cascade-migration) | `cargo test --release --lib -p wat` | 834 PASS / 0 FAIL |
| 10 | Workspace test-build clean | `cargo build --release --tests --workspace` | exit 0 |
| 11 | Clippy delta ≤ 0 | `cargo clippy --release 2>&1 \| grep -cE "^warning:"` | ≤ 902 |

## Structural verification (7 rows)

| Verification | Command | Expected |
|---|---|---|
| `parse_defenum` present | `grep -n "fn parse_defenum" src/types.rs` | 1 match |
| `parse_enum` DELETED | `grep -n "^fn parse_enum\b\\|fn parse_enum(" src/types.rs` | 0 matches |
| `parse_enum_variant` DELETED | `grep -n "fn parse_enum_variant" src/types.rs` | 0 matches |
| Legacy classify arm DELETED | `grep -n '":wat::core::enum"' src/types.rs` | 0 matches |
| `:wat::core::defenum` arm ADDED | `grep -n ':wat::core::defenum' src/types.rs` | ≥ 1 match |
| `check.rs` HARD-CUT rejection arm ADDED | `grep -n '":wat::core::enum"' src/check.rs` | ≥ 1 match (the rejection) |
| Variant Vector routes through canonical | `grep -n "parse_argspec_triples" src/types.rs` | ≥ 2 matches (defstruct + defenum) |

## Prediction: 60–120 min Mode A

Cascade size (~25-35 declaration sites) similar to 241.8's 33. Substrate mint mechanical given DESIGN locks; one-token look-ahead variant grammar is more discriminator-logic than 241.8's struct (which had 2-arg vs 3-arg arity); roughly comparable per-site cost.

Per `docs/SUBSTRATE-AS-TEACHER.md`: fail-count IS the progress meter. Sonnet should:
- Initial fail-count after substrate change: ~20-50 (each legacy enum site = 1+ test failures)
- After mechanical migration: 0
- Per failure: ~30-90 seconds (read error → identify site → convert pair-form to argspec / drop-in head rename → re-run)

## Trap-door risks

| # | Risk | Detection | Resolution |
|---|---|---|---|
| **T1** | Cascade migration time-out (>150 min) | wall clock | STOP-3; orchestrator decides re-spawn (β-style) |
| **T2** | Pattern B (tagged variant) field-name parser hits trap-door similar to 241.8 T-fd | sonnet surfaces honest delta | Document; FORM-COLLAPSE-NOTES note matching 241.8's pattern |
| **T3** | Tests that EXPLICITLY test legacy enum behavior (`wat_user_enums.rs`, `probe_let_splice_enum.rs`, etc.) | grep + test inventory | Migrate the TESTS to defenum equivalent; preserve test INTENT |
| **T4** | `parse_field` becomes orphaned after parse_enum_variant deletion | `grep -n "parse_field" src/` post-S2 | DELETE per HARD CUT (D8); if any callers remain → KEEP + document |
| **T5** | Wat source files (`wat-tests/*.wat`, `wat/kernel/services/*.wat`, crate `wat/*.wat`) with `:wat::core::enum` references in constructor/match positions (NOT declarations) | per-file inspection (declarations live at top level; constructors are `(:T::V ...)`) | Per Stone 241.8 precedent: constructors + match patterns DO NOT migrate; only DECLARATION sites do |
| **T6** | Sonnet introduces an alias arm (e.g., `:wat::core::enum` → "defenum" routing) | grep | STOP-10 (HARD CUT violation); re-brief |
| **T7** | Per-variant metadata storage tempting to schema-extend EnumDef | sonnet surfaces extension proposal | STOP-6 (scope creep); silent generic storage per D5 |

## Pre-spawn baseline checks

1. Stone 241.9 probe at HEAD = 4/8 PASS (FM 2-bis verified — C05/C06/C07/C08 disconfirm cleanly; C01-C04 weakly pass via no-op).
2. Lib at HEAD = 834 PASS / 0 FAIL.
3. All Stone 241.x probes at current counts.
4. Clippy at HEAD = ~883 (per Stone 241.8 SCORE; gate is 902).

## What completion looks like

### Phase A — SCORE scorecard + structural verification

After sonnet returns Mode A:
- 11/11 scorecard verifies locally
- 7/7 structural rows verify locally
- SCORE doc written with cascade audit + honest deltas + `parse_field` audit verdict + PHASE 3 advances inscription

### Phase B — NOT cast (legacy flat substrate; no namespaced home minted)

### Phase C — Commit + push

- Atomic commit covers: `src/types.rs`, `src/check.rs`, `src/runtime.rs` (if needed), `src/special_forms.rs` (if registry entry exists), `src/freeze.rs` (if mutation/declaration arms exist), `src/closure_extract.rs` (if dispatch entry exists), ~25-35 migration target files, SCORE doc
- Push to origin
- Stone 241.10 (`define ⇒ defn` HARD CUT) opens next

## Calibration history reference

| Stone | Class | Predicted | Actual |
|---|---|---|---|
| 241.6 | Metadata-map storage | 25-45 min | ~28.8 min (within band) |
| 241.7 | Reflection verb | 15-30 min | ~19.4 min (within band) |
| 241.8 | defstruct HARD CUT + 33-site cascade | 60-120 min | ~41 min (UNDER band) |
| **241.9 (this)** | **defenum HARD CUT + ~25-35-site cascade + parse_field audit** | **60-120 min** | **TBD** |

241.8's cascade ran UNDER band (~41 min vs 60-120 predicted). 241.9 likely similar — same cascade pattern; per-site work mechanical; substrate-as-teacher loop tight.

## What this unblocks

**Stone 241.10** — `define ⇒ defn` HARD CUT (Phase 3 third stone).
**Stone 241.11** — INSCRIPTION closes the arc.
**Arc 237.8b** — reopens after 241.11 per `feedback_no_regression_until_arc_done`.
