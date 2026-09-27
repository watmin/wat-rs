# WEIGH — STONE 255.53: a tuple is a member when each element is (`:..`) — ACCEPTED

**Executor: grok via pulsare, commit `0bc2a4f4c`.** Weighed by the orchestrator on 2026-09-26. Built from the
redraw `f1726a0ad` (the builder ruled the keyword `:..`, Typed Clojure ≥ 1.3.0's spelling).

## Re-run by the orchestrator

| row | result |
|---|---|
| release floor | **6173 passed / 22 skipped** (6163 + 10 rows); the six `ord_tuple_*` tests unchanged (`<` still uses `is_type_orderable` until stone 2) |
| the IDE's two `E0609` errors at `src/types.rs:4807` | stale: that line is a comment in the committed file, and the floor compiled it clean (tenth recurrence) |
| census / delta / clippy | grok's: pre/post census `no STOP-8`, NEW 2 / RECOVERY 0 same files, clippy rc 0. Not re-run |

## The reading

- `:..` is a **keyword marker**, legal only right after the last `extend-type` binder entry. `TypeExpr` is unchanged
  (STOP-2 held): the edge carries `tuple_each: Some("Ts")` and is keyed by `:wat::core::Tuple`.
- Membership checks every slot through 255.52's `bound_failure`. A miss names the **1-based slot**
  (`MembershipBound … got :u::Out slot 2`). The existence-only paths skip a repeated edge.
- **The empty tuple needs no ruling.** The type `(Tuple :- [])` is admitted vacuously, but it has **no values**:
  `(:wat::core::Tuple)` is refused (*"tuple must have at least one element"*), and `()` is not a tuple. A vacuous
  membership of an uninhabited type cannot be exercised. STOP-1 is closed as moot, not decided.

## What this unblocks

Stone 2 (`Orderable`/`Equatable` as declarations) can now delete `is_type_orderable` / `is_type_equatable` outright:
every arm has a declared route (leaves, conditional containers, `:..` tuples, the `Record` root under Q1, Pure enums
under EN-P, newtypes under N-R).
