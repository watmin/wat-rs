# EXPECTATIONS — STONE O

Written BEFORE the strike. Bars derived from the rule.

| # | what | command | expected |
|---|---|---|---|
| 1 | the 5 probe rows | `cargo nextest run --release -E 'test(probe_arc296_keys_on_aggregates)'` | 5 passed, `#[ignore]` = 0 |
| 2 | ⛔ it RUNS, not just checks | `./target/release/wat <each fixture>` UNPIPED | EXIT=0 and prints `"3"` for all four kinds (STOP-4) |
| 3 | the guard is not a nature list | read `src/check.rs:12725` | an aggregate test, not `matches!(… Struct \| Record \| …)` (STOP-1) |
| 4 | Peer was RULED, not admitted silently | the SCORE | states whether a Peer can be destructured and why (STOP-2) |
| 5 | the message stopped saying "struct" | `--check` a non-aggregate target, e.g. an i64 | message names what it actually requires (STOP-3) |
| 6 | 257.2's probe untouched | `git diff -- tests/wat_lang/probe_arc257_keys_destructure.wat` | EMPTY (STOP-5) |
| 7 | binder-first unaffected | rows 1-2 | still green |
| 8 | the floor | **ORCHESTRATOR**, `^ +Summary` UNPIPED | **5238 passed, 0 failed** — this stone widens an acceptance; it must not move any count |
| 9 | clippy | `cargo clippy --release --all-targets --workspace` | 0 errors |

## ⚠ ROW 8 IS DIFFERENT FROM EVERY STONE IN THIS ARC

M and N were REFUSALS: they made illegal what had been legal, so a large red floor was correct and
expected. **Stone O is the opposite — it makes legal what had been refused.** Nothing that passes
today may stop passing. A red floor here is not a worklist; it is a regression, and it means the
widening admitted something it should not have.

★ The most likely such regression is exactly STOP-2: a `Peer` or some other non-aggregate slipping
through a predicate that was widened by deleting a condition rather than by stating the right one.

## RUNTIME PREDICTION

20-40 min. One predicate, one message, plus whatever the runtime side needs. The probe is written.

## TRAP DOORS, NAMED

- **The checker and the runtime may gate separately.** We measured `--check`. Row 2 exists because a
  green check with a dying runtime is worse than an honest refusal.
- **`defholon` and `:wat::holon::defrecord` may not share `TypeDef::Aggregate`.** If either is a
  different `TypeDef` arm, "is this an aggregate?" needs to mean the right thing for it — measure
  rather than assume the four kinds are one shape internally.
- **A widened predicate is easiest to get wrong by DELETION.** Removing `if a.nature == Struct`
  admits every `TypeDef::Aggregate` — which may be correct — but state that it is the rule, do not
  arrive at it by subtraction.
