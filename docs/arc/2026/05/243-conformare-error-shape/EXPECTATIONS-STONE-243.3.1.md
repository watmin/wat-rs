# EXPECTATIONS — Stone 243.3.1 — src/check/ home + CheckEnv borrow redesign

Independent scorecard. Companion to `BRIEF-STONE-243.3.1.md` + `DESIGN-STONE-243.3.1.md`.

## Structural scorecard (12 rows)

| # | Claim | Verification | Expected |
|---|---|---|---|
| 1 | `src/check/` home minted | `test -d src/check && test -f src/check/mod.rs && echo OK` | OK |
| 2 | `src/check.rs` flat file GONE | `test ! -f src/check.rs && echo GONE` | GONE |
| 3 | CheckEnv carved to env.rs | `test -f src/check/env.rs && grep -q "struct CheckEnv" src/check/env.rs && echo OK` | OK |
| 4 | mod.rs re-exports CheckEnv | `grep -nE "pub use env::CheckEnv" src/check/mod.rs` | 1 match |
| 5 | CheckEnv carries lifetime | `grep -nE "struct CheckEnv<'a>" src/check/env.rs` | 1 match |
| 6 | `types` field borrows | `grep -nE "types: &'a TypeEnv" src/check/env.rs` | 1 match |
| 7 | `binding_metadata` field borrows | `grep -nE "binding_metadata: Option<&'a" src/check/env.rs` | 1 match |
| 8 | NO `Arc<TypeEnv>` field in CheckEnv | `grep -nE "types: Arc<TypeEnv>" src/check/env.rs` | 0 matches |
| 9 | NO binding_metadata deep-clone | `grep -nE "binding_metadata.clone\(\)" src/check/` | 0 matches |
| 10 | NO `Arc::new(types.clone())` in check | `grep -rnE "Arc::new\(types.clone\(\)\)" src/check/` | 0 matches |
| 11 | `with_builtins()` (zero-arg owned ctor) removed | `grep -nE "pub fn with_builtins\(\)" src/check/env.rs` | 0 matches |
| 12 | 6 owned fields UNCHANGED (not borrowed) | `grep -cE "(schemes\|unit_variant_types\|defined_values\|defined_value_spans\|redef_allowed\|defclause_registrations):" src/check/env.rs` | 6 (none gained `&'a`) |

## Gate scorecard (must hold)

| Gate | Command | Expected |
|---|---|---|
| Lib | `cargo test --release --lib -p wat 2>&1 \| tail -1` | ≥ 890 / 0 |
| function | `cargo test --release --test function 2>&1 \| tail -1` | 8 / 0 |
| probe TypeError (no regress) | `cargo test --release --test probe_arc243_stone3_typeerror_pattern_a 2>&1 \| tail -1` | 3 / 0 |
| **probe CheckEnv (now passes)** | `cargo test --release --test probe_arc243_stone3_1_checkenv_borrow 2>&1 \| tail -1` | **3 / 0** |
| arc112 (stays green) | `cargo test --release --test arc112_slice2b_process_send_recv 2>&1 \| tail -1` | 1 / 0 |
| :restricted-to behavioral ×4 | `cargo test --release --test wat_arc198_slice2_stone_1_inventory_wiring` (+ stone_2, stone_3, def_restricted) | each green |
| workspace build | `cargo build --release --tests --workspace` | exit 0 |
| clippy | `cargo clippy --release 2>&1 \| grep -cE "^warning:"` | ≤ 894 |

## The FM 2-bis flip (the load-bearing assertion)

| State | probe_arc243_stone3_1 | Verified |
|---|---|---|
| PRE-stone (HEAD `2eaa2ad7`) | FAILS to compile — `E0107 CheckEnv takes 0 lifetime args` ×3 + `E0308 expected Arc<TypeEnv> found &TypeEnv` ×2 | ✓ (disconfirmed) |
| POST-stone | COMPILES + PASSES 3/0 | (this stone delivers) |

If the probe does NOT flip from fail-compile to pass, the redesign did not land structurally — REJECTION regardless of other gates.

## Trap-door outcomes (sonnet's return must name each)

| # | Trap-door | Acceptable outcomes |
|---|---|---|
| T1 | `with_builtins()` can't borrow stack-local | "Removed; 3 standalone sites reshaped to bind-then-borrow" (the ONLY acceptable outcome — keeping it via leak/static is a STOP) |
| T2 | freeze.rs:329 clone fate | (a) "KILLED — FrozenWorld takes ownership of `types`; `.clone()` dropped" OR (b) "KEPT-honest — FrozenWorld persists types beyond check's borrow; clone is the persistence boundary, not duplication" — BOTH acceptable; verdict must be borrow-checker-derived + stated |
| T3 | CheckEnv escapes to heap | "Confirmed no escape; lifetime stayed in check_program frame" (if escape found → STOP, surface) |
| T4 | over-borrowing owned fields | "Only types + binding_metadata borrowed; 6 owned fields unchanged" |
| T5 | confusing borrow error | "None — cascade was verbose-mechanical" OR a surfaced pivot with the verbatim confusing error (NOT forced through) |

## Calibration

**Predicted band: 90-180 min Mode A.** Carve (mechanical) + 2-field borrow reshape (small) + 4 constructors + ~72-site lifetime cascade (verbose, borrow-checker-guided) + T2 investigation. If < 60 min: suspect the cascade was under-done (did all ~72 sites get the lifetime?) — verify count. If > 180 min: likely hit a confusing error that should have been a pivot — check for forcing.

**Time-box (orchestrator ScheduleWakeup): 180 min.** Beyond → TaskStop + assess whether a confusing-error pivot was missed.

## Pre-spawn baseline (HEAD `2eaa2ad7`)
- Lib: 890 / 0
- function: 8 / 0
- probe_arc243_stone3 (TypeError): 3 / 0
- probe_arc243_stone3_1 (CheckEnv): FAILS to compile (disconfirmed — intended)
- arc112_slice2b: 1 / 0
- clippy: 894
- CheckEnv: non-generic struct, owns `types: Arc<TypeEnv>` + `binding_metadata: Arc<HashMap>` (deep-cloned)

## What completion looks like

### Structural (all 12 rows green)
+ home minted, flat file gone, CheckEnv carved to env.rs with lifetime, 2 fields borrow, 0 clones, with_builtins() removed, 6 owned fields untouched.

### Behavioral (all gates + FM 2-bis flip)
+ probe flips fail-compile → 3/0; :restricted-to tests stay green (read-through works through the borrow); no regressions.

### Then (orchestrator-direct, post-strike)
- vigilia REMARKABLE bar on `src/check/` (8 spells; L1+L2=0) — the home is now grimoire-enforced
- SCORE-STONE-243.3.1.md authored
- atomic commit Stone 243.3.1
- 243.3.1 closes → wind up: Stone 243.3 SCORE Phase B + close (spawn-block satisfied)

### Calibration record
Actual runtime vs 90-180 band; T2 verdict; cascade site count; any pivots.
