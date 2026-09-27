# WEIGH — STONE 255.52: conditional membership on `extend-type` — ACCEPTED

**Executor: grok via pulsare, commit `bed5a7bcd`.** Weighed by the orchestrator on 2026-09-26. Ruling C.

## Re-run by the orchestrator

| row | result |
|---|---|
| release floor | **6163 passed / 22 skipped** (6155 + 8 rows) at `bed5a7bcd` |
| an **unresolved** element: `(:u::take [])` against the conditional `Mark` edge | refused: *":u::take: parameter #1 expects :u::Mark; got (:wat::core::Vector :- [_])"*. Not admitted silently; consistent with Refuse |
| a `Vector` of `i64` | refused: *"(:wat::core::Vector :- [:wat::core::i64]) is not a :u::Mark: type parameter T is bounded by :u::Mark; got :wat::core::i64"* |
| the IDE's "`generic_edge_targets` is never used" | stale: the function is deleted (`grep` finds only comments) |
| census / delta / clippy | grok's: `no STOP-8` (pre/post pair this time), NEW 2 / RECOVERY 0 same files, clippy rc 0. Not re-run |

## The diff, read

`conditional_edge` / `bound_failure` (`src/check.rs:17357-17400`) take `generic_edge_matches`' bindings and require
`assignable(binding, bound)` for each bounded parameter, on a cloned substitution. The innermost miss is carried
outward, so a nested refusal names the element that failed. A bounded edge is stored **only** as a `GenericEdge`:
no `register_subtype`, no parametric extension. `family_extends` skips any edge with a bound. So all three
argument-blind paths are closed, and unbounded edges (`Spawned`, `Seqable`, 255.48) keep their four-way OR. Both
STOPs held. The misplaced 255.51 doc comment is back on `instantiate`.

The two reds on the first floor (the inlined-wat lint on the new test's rendered-type strings; the `.wat.bad`
fixture that now loads) were kept, named and fixed before the green run, not re-run away.

## Carried

- Two comments still name the deleted `generic_edge_targets` (`src/check.rs:17483`, `:17597`). Fix at the next
  touch.
- At runtime `family_extends` does not treat a conditional member as the surface. That is harmless while such
  surfaces have no methods; a method on a conditional surface must be decided by the checker.
