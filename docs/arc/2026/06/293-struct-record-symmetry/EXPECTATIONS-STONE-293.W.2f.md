# EXPECTATIONS — 293.W.2f: process may not dial shared-memory address

Independent scorecard, fixed BEFORE the strike. Scored by the orchestrator's own re-run.

| # | what | command | expected |
|---|---|---|---|
| 1 | illegal circuit is a check error | `cargo nextest run --release -E 'test(process_map_of_thread_handle_is_a_check_error)'` | PASS — startup Err names Shared/Wire/shared-memory |
| 2 | error is not RustOpaque | the probe's assertion | no `RustOpaque` in the check error |
| 3 | 2e mouth un-regressed | `cargo nextest run --release -E 'test(address_wire)'` | PASS — `[false true]` + TypeMismatch on i64 |
| 4 | legal thread map + thread handle | `cargo nextest run --release -E 'test(thread_map_kwargs_reaches_thread_service)'` | PASS |
| 5 | thread listen/connect un-regressed | `-E 'test(probe_arc209_c0b1_thread_connection) or test(probe_arc272_autobind_listener)'` | PASS |
| 6 | no runtime Address entity rewrite | `grep -n 'struct Address' src/kernel/address.rs` | still one `Address { inner }` |
| 7 | clippy | `cargo clippy --release -p wat --tests --bins --examples` | exit 0 |

## Independent prediction

- **Runtime:** 90–180 min. Cascade on Handle/start/Bound/listener types.
- **Load-bearing:** #1 (the MCP circuit is illegal at check) + #3 (2e lives) + #4 (legal mix lives).

## Trap-door risks

- **kwargs+defclause (STOP-1).** If `/start` cannot split, the stone is blocked on a
  substrate extension. A false-green that leaves `/start -> Handle` (T unbound) will
  make `ann-form Wire` succeed on a thread handle. #1 catches that.
- **connect over-tightened.** Thread-dials-thread / thread-dials-process must remain
  legal. #4 and #5.
- **Coords record containment.** If `Address<_,_,Shared>` is impure and Coords stays
  a record of 2-arg Address, do not force Coords fields to Shared (that makes thread
  map undeclarable). 2-arg Address stays pure. #4.

## What "done" means

#1–#5 green on the orchestrator's re-run; #6 holds; the illegal MCP program would
now fail at `eval` as a CheckFailed/Fault from check, not a child EDN panic.
The orchestrator scores and commits.
