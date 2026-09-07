# EXPECTATIONS — STONE M: the enum ctor is a map

Written BEFORE the strike. Bars derived from the rule, not from what the orchestrator expects to see.

| # | what | command | expected |
|---|---|---|---|
| 1 | the four probe rows | `cargo nextest run --release -E 'test(probe_arc296_enum_map_ctor)'` | 5 passed, `#[ignore]` = 0 |
| 2 | the control never moved | `git diff -- tests/types/probe_arc296_enum_map_ctor__control.wat` | EMPTY |
| 3 | the fixtures stayed TYPED | each target fixture still calls through a `<- (:wat::core::Option :- […])` / `<- :probe::Box` slot | unchanged (STOP-1) |
| 4 | ⛔ positional REFUSED | `--check` the positional fixture, UNPIPED, read `$?` | non-zero, and the message NAMES the map form |
| 5 | Option got no special case | `git diff -- src/match_arm.rs` | the four `builtin_variant` arms UNCHANGED (STOP-2) |
| 6 | no hand-listed field names | `grep -n '"value"\|"payload"' <the changed src files>` | 0 (STOP-4) |
| 7 | the corpus did NOT move | `git diff --stat -- '*.wat'` | only this stone's 5 fixtures (STOP-5) |
| 8 | the floor | `scripts/floor.sh`, read `^ +Summary` UNPIPED | **⚠ SEE BELOW — a delta, not a total** |
| 9 | clippy | `cargo clippy --release --all-targets --workspace` | 0 errors |

## ⚠ ROW 8 IS A DELTA AND WILL BE LARGE — THAT IS EXPECTED, NOT A CRISIS

Refusing the positional ctor makes **every positional variant construction in the corpus illegal**,
and the corpus has thousands. The floor WILL go red, and the count is the migration's worklist, not
a regression. `docs/SUBSTRATE-AS-TEACHER.md` is the doctrine; FM 15 is the failure mode of panicking
at it.

★ **This is the row where the stone could go wrong quietly.** Two dishonest ways to make it green:
soften the refusal (STOP-3), or migrate the corpus inside this stone (STOP-5). Neither is allowed.
**The honest report is: the four probe rows green, clippy 0, and the floor's red count NAMED with
its top failing categories** — that number is the input to the corpus stone that follows.

If the rider judges the red is NOT explained by positional-ctor sites, that is a finding: say so,
with the arm and the verbatim block, and do not proceed.

## RUNTIME PREDICTION

45-75 min. The mechanism exists (`infer_kwargs_construct_check`); the work is a second arg shape
through it plus the refusal, not new inference.

## TRAP DOORS, NAMED

- The map form ALREADY parses and runs in untyped positions, building `Option<HashMap<…>>`. A row
  that goes green without the typed slot has measured nothing (STOP-1).
- `register_enum_methods` mints unit and payload variants down two DIFFERENT paths
  (`register_unit_variant` vs a function). Both need the map form; a fix to one is half a stone.
- Option/Result reach the checker through `builtin_variant`'s four exceptions, so they may pass or
  fail for reasons a user enum does not share. Row 4 (`option_map`) exists to catch exactly that
  divergence — if user enums go green and Option does not, the general path was not made general.
