# SCORE — type-check `accumulate`'s `:from` clauses

Executed 2026-09-10, HEAD before `cde291e42`, landed at `6ddccec63`.

## EXPECTATIONS, row by row

| what | expected | real result |
|---|---|---|
| ⭐ a legal `accumulate` compiles and fires TODAY, before any source change | **it runs**; STOP-2 fires only if it cannot be built | **PASS.** All 7 `probe_arc278_accumulate_from_types` tests except the gap ran green pre-fix: `plain_bind_compiles_and_fires`, `typed_constraint_compiles_and_fires`, `earlier_bound_join_var_compiles_and_fires` all fired natively (`fire-rules`, not the oracle) with checked derived values before `src/rete/validate/mod.rs` was touched. STOP-2 does **not** fire — `accumulate` is live, not merely declared. |
| the gap closes | ill-typed predicate in `:from` → refused, naming the type mismatch | **PASS.** `from_inner_type_error_is_refused` — pre-fix it FAILED (bug reproduced: `string::=` over an i64-bound field inside `:from` compiled clean); post-fix it PASSES (`ConstraintTypeMismatch`). |
| the inline twin is unchanged | still `ConstraintTypeMismatch` | **PASS**, unaffected by the strike, both before and after. |
| ⛔ the over-rejection guard holds | positive fixture, after the change, still compiles and fires | **PASS**, with one correction along the way (see "What the brief got wrong" below): all 4 rows of `probe_arc278_accumulate_from_types.wat` compile (`legal_accumulate_still_compiles`); 3 of the 4 legal shapes (plain bind, well-typed constraint, earlier-bound join var) are proven to actually *fire* with checked derived values, not merely compile. |
| the retired claim is recorded | `:296` comment rewritten, not deleted | **PASS** — see `git diff` in the commit; the arm's comment now names the retired claim, its source (`DESIGN-rete-defrule-wall.md` design call 3), and states the new contract. |
| ⛔ MUTATION — the check is load-bearing | revert to `validate_fact_type_head_only` in live source → refusal test REDs | **PASS.** Reinstated the deleted function, swapped the live arm back, ran: only `from_inner_type_error_is_refused` REDs (all 6 others stay green — the check is precisely load-bearing for the gap it closes, no more). Restored; re-verified green. |
| ⛔ MUTATION — the guard is load-bearing | make the arm refuse every operand it cannot type → positive fixture REDs | **PASS, with a correction.** First attempt (mutating `check_constraint_head`'s `ComputedNotDerivableHere` arm to push an error) left the not-knowable row green — because that row, copied from the fence's row 3 (`i64::+ ?v 1 :undefined 0`), does **not** actually resolve to `ComputedNotDerivableHere` (see below). Rebuilt row 4 with `cond` (the same construction D10's `nk1` uses). Re-ran the identical mutation: `legal_accumulate_still_compiles` AND `not_knowable_operand_still_compiles` both RED. Restored; re-verified green. |
| the sibling arms are untouched | `not`/`exists`/`or`/`and`/`:then` all still refuse | **PASS**, driven directly: `probe_arc278_fence_interior_types` (3/3), `probe_arc278_D10_then_field_types` (5/5), `probe_arc278_8a_accumulate_oracle` (5/5), `probe_arc278_8b_accumulate_native_differential` (5/5), `probe_arc278_8i_accumulator_folds` (10/10) — all green post-fix. |
| scope held | no change to `reachability.rs`, `typing.rs`, `clause.rs`, `wat-scripts/` | **PARTIAL.** `reachability.rs`, `clause.rs`, `wat-scripts/` untouched. `typing.rs` **was** touched — 18 lines removed — but only to delete `validate_fact_type_head_only`, which became genuinely dead code (its one caller was the arm the fix replaced) and would have failed the clippy `-D warnings` gate as `dead_code` if left in. No check *logic* in `typing.rs` changed; this is a forced, zero-behavior consequence of the one-line `mod.rs` edit, not scope creep on the check itself. |
| floor | 0 failed, ≥ 5516 + new tests | **PASS on the second run** (see below): `Summary [ 464.410s] 5523 tests run: 5523 passed (2 slow), 19 skipped`, 0 FAIL lines in `.floor/2026-09-10T18-24-52Z/clean.log`. |
| clippy | rc=0 | **PASS**, `cargo clippy --all-targets --release -- -D warnings`, clean both times it was run (pre- and post-floor-fix). |

## The floor RED, reported honestly

The floor was run once and went RED before the final green:

- **Summary** (`.floor/2026-09-10T18-16-21Z/clean.log`): `Summary [ 464.273s] 5523 tests run: 5522 passed (1 slow), 1 failed, 19 skipped`
- **Failing test**: `wat::lint rete_citation_resolves::every_backticked_name_in_a_rete_comment_resolves`
- **The exact arm**: the gate's own report named the unresolved citation precisely — `src/rete/validate/mod.rs:300  \`check_constraint_types\``. The real function is `check_constraint_head`; my first-draft rewritten comment cited a name that has never existed. This is not the class of failure the wat-rs CLAUDE.md's "no known flakes" doctrine is warning about — it is a deterministic, single-character-away typo in my own newly-written text, with the gate naming the exact site and the exact wrong string. I did not re-run blind: I read the whole arm (captured in `.floor/2026-09-10T18-16-21Z/ARM.txt`), confirmed by grep that `check_constraint_types` does not exist and `check_constraint_head` does, corrected the citation, and re-ran the full floor once. The second run is the one reported as the strike's result: `Summary [ 464.410s] 5523 tests run: 5523 passed (2 slow), 19 skipped`, 0 FAIL.

## The open question — answered

`Design call 3`'s comment ("`:from`'s inner gets fact-type-HEAD validation only... out of scope") traces to `docs/arc/2026/06/294-holon-returns-to-vsa/DESIGN-rete-defrule-wall.md`, "The three design calls," item 3:

> "`where`/`accumulate` depth = shape + alpha refs, not interiors (v1 scope)... It does NOT deep-walk a `where` predicate's arbitrary pure expr... or an accumulate reducer's body. If interior validation is wanted, it's a **named follow-on**, not a half-wall on *this* class... ← the one scope bound; flag if you want it wider."

This is a **stated v1 scope bound, explicitly flagged there as narrowable** — not an architectural necessity. It is the *same shape and, largely, the same words* as the `Where` arm's justification, which turned out to be a genuine hole (cured earlier the same day at `6f1d93451`) — not the shape of `reachability.rs`'s exclusion, which carries a specific technical argument (`Cell` cannot render a differential-against-a-wrapped-control shape) and stays correctly untouched here. So: **the reason does not still hold as a reason to leave `:from`'s clauses unchecked** — it was a v1 boundary the same design doc invited widening on, and this strike is exactly that named follow-on for the `:from` half (not the reducer body, which stays out of scope, unmeasured, per DESIGN.md).

## What the brief got wrong

1. **The `.wat.bad` sibling this strike modeled itself on (`probe_arc278_fence_interior_types.wat` row 3) does not test what its own comment claims.** `(:wat::rete::core::i64::+ ?v 1 :undefined 0)` was described there as "NOT KNOWABLE... a computed rete form." Driven directly (twice — once via a live mutation of `ComputedNotDerivableHere`, once by reading `vocabulary.rs`): `i64::+`'s row declares `ret: Ret::Is(ParamType::I64)`, a REAL, knowable return type (`Fallback`-class, still total), so `resolve_operand_type` returns `Resolved("i64")`, never `ComputedNotDerivableHere`. My first not-knowable row, copied from that precedent, inherited the same defect — the load-bearing mutation exposed it directly (stayed green under an active over-rejection mutation). Replaced with `cond` — the same `Form`-class construction D10's `nk1` fixture already uses correctly on the `:then` side — and the mutation now reds it as required. **This is worth flagging upstream**: the fence's row 3 is not exercising the guard it claims to.
2. **"Compiles and fires" ran into a genuine, out-of-scope native-engine defect.** `acc::count` combined with an *unused* extra bind (`?v <- :value`) present in `:from`'s inner but not consumed by the acc-form derives a wrong count natively — 3 real readings produce `n=1`, reproducible, independent of any constraint (`acc::sum ?v` with the identical two binds is unaffected: 10+20+30 sums correctly, filtered sums correct too). This is a `src/rete/kernel/fire/acc.rs` / `fire/pass/accumulate.rs` question, not a validator one — DESIGN.md explicitly excludes the reducer/engine body, so it is reported here, not fixed. It cost a round of fixture rework (rows 2 and 3 rebuilt on `acc::sum` instead of `acc::count`) and is the reason "anchor a new measurement before trusting it" mattered here: the first draft's exact-count assertions were wrong, caught only by driving a standalone diagnostic `.wat` script before trusting the Rust test's own failure.
3. **The runtime prediction (60-90 minutes) was low** given the two corrections above plus the floor RED — the actual work involved two rounds of fixture reconstruction from empirical, driven measurement rather than reasoning off the type-checker's source alone.

## Line counts (final diff, `6ddccec63`)

```
 src/rete/validate/mod.rs                            |  11 +-   (net: +8/-3, comment rewrite + one-arm swap)
 src/rete/validate/typing.rs                         |  18 --   (dead-code removal, forced)
 tests/rete/probe_arc278_accumulate_from_types.rs    | 236 ++   (new)
 tests/rete/probe_arc278_accumulate_from_types.wat   |  69 ++   (new)
 tests/rete/probe_arc278_accumulate_from_types_from.wat.bad   |  22 ++   (new)
 tests/rete/probe_arc278_accumulate_from_types_inline.wat.bad |  13 ++   (new)
 6 files changed, 348 insertions(+), 21 deletions(-)
```

Source change: 11 lines touched in `mod.rs` (one arm's call + a rewritten comment), 18 lines deleted in `typing.rs` (dead code). The positive corpus + refusal fixtures are 340 of the 348 inserted lines — the deliverable was the corpus, as the brief said it would be.

## Both mutations

- **Check-load-bearing** (revert arm to `validate_fact_type_head_only`): `from_inner_type_error_is_refused` REDs; all 6 siblings stay green. Restored, re-verified green.
- **Guard-load-bearing** (make `check_constraint_head`'s `ComputedNotDerivableHere` arm refuse): `legal_accumulate_still_compiles` AND `not_knowable_operand_still_compiles` both RED (first attempt, with the fence-copied row 4, incorrectly stayed green — corrected per "what the brief got wrong" item 1 before this result). Restored, re-verified green.

## Floor, verbatim

```
Summary [ 464.410s] 5523 tests run: 5523 passed (2 slow), 19 skipped
```

FAIL lines in `.floor/2026-09-10T18-24-52Z/clean.log`: **0**.

clippy: `cargo clippy --all-targets --release -- -D warnings` → rc=0, no warnings.
