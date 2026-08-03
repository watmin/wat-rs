# EXPECTATIONS — slice one of the rete `where` vocabulary

Written BEFORE the strike, so the result cannot move the goalposts. Scored after the
orchestrator's OWN re-run, never the rider's report.

## Scorecard

| # | what | the command that checks it | expected |
|---|---|---|---|
| 1 | builds clean | `cargo build --release --all-targets` | exit 0, **zero** warnings |
| 2 | clippy clean | `cargo clippy --release --all-targets` | exit 0, **zero** warnings |
| 3 | ★ the admission test ADMITS | new `tests/rete/` case: a rete-module head | admitted |
| 4 | ★ the admission test REFUSES the engine API | same test: bare `:wat::rete::fire-rules` | refused |
| 5 | ★ the admission test REFUSES core | same test: a `:wat::core::` head | refused |
| 6 | ★ composition survives, BY A RUN | a user `defn` over the four ops, classified | admissible, transitively |
| 7 | the four ops dispatch | a run calling each of the three ops outside a rule | correct values |
| 8 | `:undefined` is mandatory | omit it on `:wat::rete::i64::+` | expansion fails, `kwargs-lower: missing argument :undefined` |
| 9 | the fallback FIRES | `(:wat::rete::i64::+ i64::MAX 1 :undefined -1)` | `-1`, no raise |
| 10 | ⛔ the corpus is UNMOVED | `./wat-scripts/perf/grid/check-where-shapes.sh` | 9 pairs, 99 forms, all agreeing |
| 11 | the fence is still UNARMED | read `wat/rete.wat:661` | `(and is-pure is-det)` — **unchanged** |
| 12 | rete suite green | `cargo test --release --test rete` | no new failures |
| 13 | repo lints green | `cargo test --release --test lint` | no new failures |
| 14 | ★ op #5 is ONE ROW | inspect the diff | no rete op named in more than one file |
| 15 | whole floor | orchestrator's own `cargo nextest run --release` | Summary ≥ the pre-strike baseline, 0 failed |

Rows 3-6 and 14 are load-bearing. **Rows 3-5 together, not separately** — an admission test that
only refuses is the vacuous-gate class (R59; `91bbb8cd`'s 11 gates; R62's empty rejection column).
Row 14 is the stone's actual contract; the four ops are its demonstration.

## Baseline, measured before the spawn

Floor at HEAD, orchestrator's own `--release` run — recorded here so row 15 has something to
compare against rather than a remembered number.

- Reflection target: **73 passed / 0 failed / 11 ignored** (after the probe cleanup).
- Whole floor: **recorded at spawn time in the SCORE** — the pre-strike weigh was running as this
  was written; the SCORE carries the number, not this file's memory of it.

## Runtime prediction

**35-55 minutes**, Mode A, if the `:4829` refactor is a contained move.
**Upper bound 75 minutes**; wakeup scheduled at 2×.

The band is wide on purpose: three of the four ops are mechanical (a table, two registration loops,
a dispatch arm, an inference-arm mirror), and the fourth carries the one unmeasured thing.

## Trap-doors named in advance

1. **The `:4829` refactor is the only real unknown.** The probe closed *which path* by a run; it did
   **not** measure *how big*. If moving the inline arm onto `arith_i64_i64_inner`'s kernel touches
   more than the arithmetic arms, STOP-3 fires and ops #1/#3/#4 land without #2. **That is a
   partial success, not a failure** — and it is the outcome I would bet on if the band is exceeded.
2. **`and`'s inference arm may not mirror cleanly.** `check.rs:4230` handles `and`/`or` together;
   a rete twin may want its own arm rather than a widened guard. If the clean edit is a widened
   guard, that is fine — it is a head-table form, not a structural one. If it starts reaching into
   `classify_expr`'s structural arms, STOP-4 fires.
3. **The table may want to live somewhere `purity.rs` cannot see.** `vocabulary.rs` under
   `src/rete/` is the drawn home; if `check.rs`/`runtime.rs` cannot reach it without a visibility
   change that widens more than `pub(crate)`, report rather than widening.
4. **A rete-namespaced name may trip a reserved-prefix or namespacing gate** at registration.
   `:wat::` is reserved (`resolve/reserved.rs`) and the namespacing wall armed at `b18888f8`. This
   is *expected to be fine* — the registration is substrate-side, not user-side — but it is
   unproven, and it is the most likely place for an early surprise.

## What a Mode B looks like

Ops #1/#3/#4 land green, #2 blocked on a measured `:4829` number, corpus unmoved. That is a
**reportable, committable** outcome: the table exists, the admission test discriminates, and the
fallback class has a number attached to its blocker instead of a guess.

## What would make this a failure

The corpus moving (row 10), the fence arming (row 11), an op named twice (row 14), or an admission
test that only demonstrates refusals (rows 3-5 as a set).
