# EXPECTATIONS — 293.W.2e: `address-wire?`

Independent scorecard, fixed BEFORE the strike. Scored by the orchestrator's own re-run.

| # | what | command | expected |
|---|---|---|---|
| 1 | the FM 2-bis probe flips GREEN | `cargo nextest run --release -E 'test(address_wire_is_false_on_thread_true_on_process)'` | PASS — `[false true]` |
| 2 | thread address is not a wire | the probe's first element | `false` |
| 3 | process address is a wire | the probe's second element | `true` |
| 4 | non-Address is TypeMismatch | `cargo nextest run --release -E 'test(address_wire)'` (the `.wat.bad` sibling) | PASS — check error names `Address<S,R>` |
| 5 | `peer-wire?` un-regressed | `cargo nextest run --release -E 'test(peer_wire) or test(peer-wire)'` | green (or 0 tests if none named that — then the service-client path still type-checks) |
| 6 | connect / listener un-regressed | `cargo nextest run --release -E 'test(probe_arc209_c0b1_thread_connection) or test(probe_arc272_autobind_listener)'` | PASS |
| 7 | no Address type-param invented | `grep -n 'Address<.*,.*,' src/check.rs src/runtime.rs src/kernel/address.rs` | no third type argument |
| 8 | workspace green | `cargo nextest run --release` (or the modules touched + lint) | 0 failed on the touched binaries; clippy clean on `src/runtime.rs` `src/check.rs` |

## Independent prediction

- **Runtime:** 15–35 min. One verb, copied from `peer-wire?` + the connect downcast.
- **Load-bearing rows:** #1 (the gap closes) + #2/#3 (the axis is right) + #7 (no 2f leak).

## Trap-door risks

- **Wrong predicate.** If `portable_form` is empty for a live process listener, #3 goes red
  and that is a finding, not a prompt to special-case `listener`.
- **Overload `peer-wire?`.** A TypeMismatch that used to name `peer` now names Address —
  #5 / existing `peer-wire?` sites catch it.
- **Scope creep into 2f.** Third type param, `/start` defclause, Setup compile error —
  #7 and the BRIEF STOP-1. Surface and stop.

## What "done" means

#1–#4 green on the orchestrator's re-run; #6/#7 hold; no type-parameter work landed;
`portable_form().is_some()` is the body. Then the orchestrator scores and commits.
