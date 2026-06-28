# EXPECTATIONS — 293.4b: the generated surface dispatcher

Independent scorecard, fixed BEFORE the strike. Scored by the orchestrator's own re-run.

| # | what | command | expected |
|---|---|---|---|
| 1 | the 293.4b probe flips GREEN (un-ignored) | `cargo nextest run --release -E 'test(surface_method_dispatches_by_runtime_type)'` | PASS |
| 2 | `:Surface/method` RESOLVES (no UnresolvedReference) | the probe's startup no longer throws `Resolve(UnresolvedReferences[":t::Shape/area"])` | startup Ok |
| 3 | dispatch ROUTES by runtime type | the probe asserts `(:t::circle-area)` ≈ 12.566 (π·2²) and `(:t::square-area)` = 9.0 (3²) — distinct impls | both correct |
| 4 | a NON-satisfier receiver is rejected (negative) | a record that does NOT satisfy `:t::Shape` passed to `:t::describe` (or to `:t::Shape/area` directly) | rejected at check time — executor SHOULD add this arm; orchestrator adds at weigh if absent |
| 5 | 293.4a still green (no regression to parse/satisfy) | `cargo nextest run --release -E 'test(method_member_surface_parses_and_is_satisfied_by_a_defn)'` | PASS |
| 6 | acceptance demo stays RED (untouched) | `grep -n '#\[ignore' tests/types/probe_arc293_acceptance_demo.rs` | still `#[ignore]`'d |
| 7 | whole workspace green | `cargo nextest run --release` | `4088 passed / 0 failed / N skipped` (floor 0; the new probe adds 1 pass) |

## Independent prediction
- **Runtime:** 30–55 min. A 3-layer mirror (resolve + check + runtime) of an existing, well-grounded path
  (the arc-232 protocol dispatch). The work is parallel-arm insertion, not new design — the contract is pinned.
- **Load-bearing rows:** #1 + #3 (routing is correct, not just resolving) + #7 (no regression). #4 proves the
  dispatcher REQUIRES satisfaction, not just any receiver.

## Trap-door risks (named)
- **The protocol path is the template, but surfaces route to a `defn` not an `extend-def`.** The ONE semantic change.
  If the executor copies the protocol dispatch verbatim and looks up `extend:<S>:<T>` (which won't exist for a surface),
  #1/#3 fail. Weigh #3 against the disk — confirm the dispatched value is the satisfier's OWN impl.
- **Resolve-layer omission.** The RED is a *resolve* error, so the FIRST wall is the resolver. If the executor fixes
  check + runtime but not resolve, the head stays UnresolvedReference and #1 never reaches dispatch. All three layers.
- **Field-accessor / surface-method collision (STOP-3).** `:<T>/<field>` and `:<S>/<m>` share the `<x>/<y>` head shape;
  disambiguation is by registry kind (record-fn vs surface-with-method-member). Verify the disambiguation is real.

## What "done" means
#1, #3, #4, #7 green by the orchestrator's own re-run; #5 confirms 293.4a un-regressed; #6 confirms the demo untouched;
the dispatched values are the satisfiers' own impls (read the probe result). Then commit on green.
