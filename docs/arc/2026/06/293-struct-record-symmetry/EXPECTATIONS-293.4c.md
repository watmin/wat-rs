# EXPECTATIONS — 293.4c: `extend-type` as the foreign-accessor adapter

Independent scorecard, fixed BEFORE the strike. Scored by the orchestrator's own re-run.

| # | what | command | expected |
|---|---|---|---|
| 1 | the 293.4c probe flips GREEN (un-ignored) | `cargo nextest run --release -E 'test(extend_type_teaches_a_foreign_type_to_satisfy_a_surface)'` | PASS |
| 2 | a FOREIGN type satisfies + dispatches via extend-type | the probe's `(:t::probe)` → `42` (the adapter's `:wat::core::String/tag` impl, dispatched on a String receiver) | 42 |
| 3 | COLLISION = DuplicateDefine (negative) | a `_dup_bad.wat`: two `extend-type` of the same `:<T>/<m>` (or extend vs real defn) → startup `Err` | rejected (DuplicateDefine) |
| 4 | a NON-extended foreign type is REJECTED (negative) | a foreign type with no `:<T>/tag`, passed where `:t::Tagged` is required → check `Err` | rejected (not satisfied) |
| 5 | protocol `extend-type` un-regressed | `cargo nextest run --release -E 'binary(function)'` (+ any defprotocol/extend tests) | green (arc-232 path untouched) |
| 6 | 293.4a + 293.4b un-regressed | `cargo nextest run --release -E 'test(method_member_surface_parses) + test(surface_method_dispatches)'` | both PASS |
| 7 | acceptance demo stays RED (untouched) | `grep -n '#\[ignore' tests/types/probe_arc293_acceptance_demo.rs` | still `#[ignore]`'d |
| 8 | whole workspace green | `cargo nextest run --release` | `4090 passed / 0 failed / N skipped` (floor 0) |

## Independent prediction
- **Runtime:** 35–60 min. Three coordinated pieces (extend-register / non-aggregate satisfaction / general receiver
  extraction), each small, but they must agree — the same "satisfies iff `:<T>/<m>` resolves" rule in check AND runtime.
- **Load-bearing rows:** #2 (foreign dispatch works) + #3 + #4 (the negatives prove it's a real adapter, not always-true)
  + #5/#6 (no regression to protocols or 293.4a/b).

## Trap-door risks (named)
- **Satisfaction goes always-true (STOP-3).** Dropping the Aggregate gate could make every type satisfy every
  method-only surface. #4 catches it — a non-extended type MUST be rejected. Weigh #4 against the disk.
- **The `type_name()` FQDN ≠ extend key (STOP-2).** Generalizing the dispatcher to `type_name()` must not silently
  mis-map a value whose `type_name` differs from the `:<T>` the user wrote. The probe uses String (unambiguous); if the
  executor needs other types, it surfaces the mapping, not guesses.
- **Collision not enforced.** If extend-type silently overwrites an existing `:<T>/<m>`, #3 fails. DuplicateDefine must fire.

## What "done" means
#1, #2, #3, #4, #8 green by the orchestrator's own re-run; #5/#6 confirm no regression; #7 confirms the demo untouched;
satisfaction is a REAL check (the negatives reject). Then commit on green.
