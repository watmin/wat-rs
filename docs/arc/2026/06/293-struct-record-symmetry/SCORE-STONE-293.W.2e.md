# SCORE — 293.W.2e: `address-wire?`

**Verdict: GREEN, weighed by the orchestrator's own re-run.** The FM 2-bis probe that was
`UnknownFunction` at `34e9bd32` now returns `[false true]`. Body is
`addr.portable_form().is_some()`. No 2f leaked in.

## Scorecard (each row re-run by the orchestrator)

| # | what | result |
|---|---|---|
| 1 | FM 2-bis probe GREEN | **PASS** — `[false true]` |
| 2 | thread address is not a wire | **PASS** — first element `false` |
| 3 | process address is a wire | **PASS** — second element `true` |
| 4 | non-Address is TypeMismatch | **PASS** — `address_wire_non_address_is_type_mismatch` names `Address<S,R>` |
| 5 | `peer-wire?` un-regressed | **N/A as a named test** (0 tests); verb untouched |
| 6 | connect / listener un-regressed | **PASS** — `probe_arc209_c0b1_thread_connection` + `probe_arc272_autobind_listener` |
| 7 | no Address type-param invented | **PASS** — grep of `Address<.*,.*,` in the three rooms is only `"Address<S,R>"` strings |
| 8 | clippy on the touched crate | **PASS** — `cargo clippy --release -p wat --tests --bins --examples` exit 0 |

Independent command: `cargo nextest run --release -E 'test(address_wire) or test(probe_arc209_c0b1_thread_connection) or test(probe_arc272_autobind_listener)'` → **4 passed**.

## What shipped

- `src/runtime.rs` — dispatch next to `peer-wire?`; `eval_address_wire` reuses the
  `eval_connect_prime` `ADDRESS_TYPE_PATH` downcast; returns
  `Value::bool(addr.portable_form().is_some())`.
- `src/check.rs` — `infer_address_wire`: 1 arg, unify `Address<S,R>`, return bool.
  Does not call `project_peer_io`.
- Negative control kept: `probe_arc293_W2e_address_wire.wat.bad` + driver.

## Honest deltas

None on the axis. Full floor was not re-run for this weigh (stone is one verb; #6
covers the sibling listener/connect rooms). 2f is still sheathed: the type of
`Address` still lies; a process Setup of a thread address is still a runtime EDN
death, not a check error.
