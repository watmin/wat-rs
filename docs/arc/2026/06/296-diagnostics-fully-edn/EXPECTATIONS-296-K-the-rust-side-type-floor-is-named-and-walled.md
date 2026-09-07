# EXPECTATIONS — 296 K: the Rust-side type floor is named and walled

Written BEFORE the strike.

⚠ **THE WALL IS THE STONE, AND A WALL THAT ADMITS SOMETHING IS HARDER TO TRUST THAN ONE THAT ADMITS
NOTHING.** J's wall could be proved by one sabotage. K's has to be proved in BOTH directions: it must
REFUSE a non-root literal and ADMIT a root one. Rows 5 and 6 are that pair, and neither alone is
evidence.

| # | what | command | expected |
|---|---|---|---|
| 1 | the alias macro exists | `grep -c 'pub fn wat_alias_register_from' crates/wat-source-derive/src/lib.rs` | 1 |
| 2 | the movable aliases are IN WAT | `grep -n 'typealias :wat::holon::BundleResult\|:wat::holon::Holons\|:wat::core::Bytes' wat/holon.wat wat/core.wat` | 3 hits |
| 3 | their Rust literals are gone | `grep -c 'register_builtin(TypeDef::Alias' src/types.rs` | ≤1 — only the named floor (`nil`, if STOP-2 fired) survives |
| 4 | the aggregate literals left are ONLY roots | for each surviving `TypeDef::Aggregate` literal, `Nature::from_root_keyword(name)` | `Some(..)` for every one |
| 5 | ⛔ the wall REFUSES a non-root | add `TypeDef::Aggregate` named `:wat::probe::NotARoot`, run the lint, revert | **RED**, naming the site |
| 6 | ⛔ the wall ADMITS a root | the three category roots are present and the lint is **green** | green — a wall that refuses everything is not this wall |
| 7 | the wall asks the ORACLE, not a list | read the lint | it calls/mirrors `Nature::from_root_keyword`; no hand-list of root names (STOP-1) |
| 8 | zero exemptions added to pass | `grep -c 'allow\|rune:' <the lint>` | 0 (STOP-6) |
| 9 | the floor | `./scripts/floor.sh` unpiped, Summary line | `0 failed` |
| 10 | clippy | `cargo clippy --release --all-targets --workspace` | 0 errors |
| 11 | the named floor is ARGUED | the SCORE | every surviving Rust literal carries a reason distinguishing "impossible in principle" from "not moved yet". A survivor without a reason is the stone failing its own purpose |

## RUNTIME PREDICTION

**45–75 min.** The alias macro is the H-3-shaped piece; three moves are mechanical; the wall's
two-direction proof (rows 5+6) is the care.

## TRAP DOORS

- **Row 6 is the one that gets skipped.** Proving a wall refuses is instinctive; proving it still
  ADMITS what must pass is the half that catches a wall drawn too wide. A wall that refused the
  category roots would go red on the whole build and look like a transcription bug.
- **`:2442` registers in a LOOP** (`format!(":wat::runtime::{}", variant)`). A text-scanning lint may
  or may not see it as a literal. STOP-3 exists because the honest answer might be "the wall is at
  the wrong level", not "add an exemption".
- **`nil` is a plausible bootstrap root and the stone must be able to say so.** Forcing it into wat
  to make row 3 read `0` would be the goal-seeking this arc keeps catching.
- **13 records are ALREADY wat-sourced.** The record half of this class is largely done; do not
  re-derive it, and do not report the 13 as new work.
