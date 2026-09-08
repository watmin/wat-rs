# EXPECTATIONS — STONE M2

Written BEFORE the strike. Bars derived from the rule, not from what the orchestrator expects to see.

| # | what | command | expected |
|---|---|---|---|
| 1 | ⛔ the toolchain RUNS again | `./target/release/wat <any corpus .wat>; echo $?` UNPIPED | **not 3** |
| 2 | ⛔ fixture-local errors at zero | for every worklist path: `--check` and count errors whose `:file` is that path | **0**, all 288 |
| 3 | the codemod exists and is recorded | `ls wat-scripts/fixes/positional-ctor-to-map.wat` | present, committed |
| 4 | ⛔ it ASKED, it did not observe | `grep -c "type-of\|eval-with-defs!" <the codemod>` | **> 0**; and `grep -c "defrecord\|defstruct"` as a field-map source → 0 (STOP-1) |
| 5 | idempotent | re-run the codemod over the same paths | 0 further changes |
| 6 | generated enums migrated | `grep -c "Cache::Reply\|Journal::Reply\|Store::Reply" ` in migrated files shows MAP form | map form, not positional |
| 7 | the probe's bar is fixed | `grep -n "fn check" -A 12 tests/types/probe_arc296_enum_map_ctor.rs` | counts fixture-local errors, no bare exit code |
| 8 | the five probe rows | `cargo nextest run --release -E 'test(probe_arc296_enum_map_ctor)'` | 5 passed, `#[ignore]` = 0 |
| 9 | `src/` unchanged vs M | `git diff HEAD -- src/` | EMPTY (STOP-3) |
| 10 | spellings untouched | `git diff -- '*.wat' \| grep -c "Option::None\|Option::Some"` | 0 new (STOP-6) |
| 11 | the floor | **ORCHESTRATOR.** `scripts/floor.sh`, `^ +Summary` UNPIPED | see below |
| 12 | clippy | `cargo clippy --release --all-targets --workspace` | 0 errors |

## ⚠ ROW 11 IS THE ONE THAT DECIDES THE STONE

The floor went `2447 passed / 2773 failed / 18 timed out` at M. **M2's floor should return to the
5,2xx-passed band with 0 failed and 0 timed out** — the timeouts especially, because they were
spawn-tests whose child died at startup, and nothing but a working toolchain fixes them.

★ **If the floor is green EXCEPT for a residue, the residue is the finding** — name it with the arm
and the verbatim block, do not fold it into "migration noise." Two shapes are expected and are NOT
the same thing:

- **goldens that pin a stdlib `wat/*.wat` line.** The migration moves lines in `service.wat` and
  friends, and `109/NOTE-a-golden-that-pins-a-stdlib-line.md` already records this class: RECAPTURE,
  KEEP PINNING, do NOT extend the normaliser.
- **anything else.** A finding. Report it.

## RUNTIME PREDICTION

90-150 min. The codemod is a sibling of an existing one, but the reflection half (extract decls →
`eval-with-defs!` → `type-of` → two-level enum→variant→fields map) is real work, and the dance costs
two release builds.

## TRAP DOORS, NAMED

- **The 32 generated enums are the whole risk.** A codemod that "works" on a dry run of 3 files may
  have sampled only hand-written ones. **Dry-run at least one file whose enum is generated** —
  `wat/cache.wat` (Cache::Op / Cache::Reply) or `wat/telemetry/journal.wat` (Journal::Reply).
- **`wat/fix.wat` rewrites its own source** (67 sites). Safe during the run — the executing copy is
  frozen into the binary — but it means step 5's rebuild is the first time the new `fix.wat` is
  compiled. If step 5 fails to build, suspect that file first.
- **A variant head is three segments** (`:ns::E::V`) where a record head is two. An accessor is
  `:ns::E/field` and a type reference sits after `<-` or inside `:- [...]`. The codemod must
  discriminate all four; `positional-to-kwargs.wat` gates on head + arg-count for the same reason.
- **Declaration ORDER, not alphabetical.** `type-of` answers in declaration order and that is
  load-bearing — 296 L's SCORE calls it "the row that matters." A map built from an unordered
  source will look right and bind wrong.
- **Partial success is failure here.** Unlike the arm campaign, where a red count was a progress
  meter, this tree does not RUN until the last site moves. There is no useful intermediate state.
