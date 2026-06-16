# DESIGN + BRIEF — 258.5b-ii: encode in eval, ship bytes (kill the send-type thread-local)

## Why

258.5b (the `recv'`/`select'` arrow kill) is functionally GREEN, but it left a `sequi` smell: the **send'
encode** of the unified `Peer` (`PEER_TYPE_PATH`) goes through `EdnRepresentable::to_wire(&self) -> String`
(`comms/mod.rs:788`, no types param) *inside the transport*, so it couldn't see `sym.types()` and emitted
`field-0`/`field-1`. The 258.5b strike bridged it with a **thread-local `SEND_TYPE_ENV`** — hidden ambient
state the encoder secretly reads. `sequi`: *"state must follow through every transformation, visibly,
through the types. Hidden state breaks composition."* Four-questions failed it on Obvious/Simple/Honest.

## The decomplection (the real asymmetry)

**Decode already happens in the eval layer** — `recv'` receives a raw wire `String` (`edn_str`) from the
transport and decodes it with `decode_trusted_wire(edn_str, sym.types())` *in `eval_peer_recv_prime`*.
**Encode is the asymmetric wart**: `send'` hands the transport a `Value` and the transport encodes
internally via `to_wire()` (no types). The fix is to make send symmetric with recv: **the eval layer
encodes with `sym.types()` and ships the wire `String`; the transport just moves bytes.** Precedent
already exists — the bundle/process arm does exactly this: `bundle.peer.send(edn_str)` (`runtime.rs:23588`)
sends an eval-encoded `String`.

## Contract (pinned)

1. **`eval_peer_send_prime` PEER_TYPE_PATH arm** (`src/runtime.rs` ~23545, and the sibling ~23632):
   instead of `peer.send(payload_val)` (Value, encodes internally), encode in eval —
   `let wire = crate::edn_shim::value_to_edn_string_with(&payload_val, sym.types().map(|a| a.as_ref()))`
   (a `String`) — and send that `String` over the peer's **string/raw send path** (the counterpart to the
   recv side that already yields `edn_str`). Remove the `with_send_types(...)` wrapper.
2. **`src/edn_shim.rs`**: DELETE the thread-local `SEND_TYPE_ENV` + `with_send_types` + `current_send_types`.
   `value_to_edn_string` either (a) gains a `_with(types)` sibling the eval calls, or (b) the eval calls
   the existing `value_to_edn_with(v, types)` + `wat_edn::write` directly. Remove the thread-local read in
   the Value `to_wire` impl — `to_wire` returns to its registry-free form (it is now only used for the
   non-typed transport paths / primitives, OR is no longer on the typed-record send path at all).
3. **Symmetry check:** confirm the Peer's transport exposes a raw-`String` send that mirrors how recv
   delivers `edn_str` (the transport is fundamentally a bytes pipe; `to_wire`/`from_wire` were the
   asymmetric wart). If the unified `Peer` has NO raw-string send path, **STOP** and report — that means
   the clean shape needs a small Peer method (`send_wire(String)`), which is still in-scope but worth
   surfacing before adding.

## STOP triggers
1. If the unified `Peer` (`from_socket`, `CommSender<Value>`) has no raw-`String` send path and adding one
   is non-trivial, STOP and report the transport's actual send surface — do NOT re-introduce a thread-local
   or change the `EdnRepresentable`/`CommSender` trait contract without surfacing it first.
2. If removing the thread-local breaks a NON-process-peer encode that depended on it, STOP + report (the
   thread-local should only have fed the unified-Peer send' record case).
3. Do NOT touch the decode side (it already threads `sym.types()` correctly) or `recv'`/`select'` (258.5b done).

## Probe (the guard already exists)
`tests/probe_arc272_6c2_record_ipc_derisk.rs` must STAY GREEN (a base record round-trips over process IPC,
no arrow) — but now via the honest eval-encode path, with NO thread-local in the tree. Add an assertion is
unnecessary; the existing probe proves the record's *fields* survive (the projection), which only holds if
the encode emitted NAMED keys — so green-via-no-thread-local is the proof.

## Blast radius
`src/runtime.rs` (eval_peer_send_prime PEER arm), `src/edn_shim.rs` (delete the thread-local + helpers;
value_to_edn_string sibling), possibly `src/kernel/peer.rs` or `src/comms/process.rs` (a raw-string send
path if not already present). NO trait-contract change to `EdnRepresentable`/`CommSender`. NO decode-side
or recv'/select' change.

## Verify (run + report each)
Baseline: lib 928 passed / 36 failed.
1. `cargo build --release -p wat 2>&1 | tail -5`
2. `cargo test --release -p wat --test probe_arc272_6c2_record_ipc_derisk 2>&1 | grep "test result"` (stays passed — now WITHOUT a thread-local)
3. `cargo test --release -p wat --test probe_arc272_6a_capability_handoff 2>&1 | grep "test result"` (capability over IPC — stays passed)
4. `cargo test --release -p wat --lib -- --test-threads=1 2>&1 | grep "test result"` (≥928 passed; failed == 36)
5. `cargo test --release -p wat 2>&1 | grep -E "test result: FAILED" | head` (no NEW failing binaries)
6. `grep -rn "SEND_TYPE_ENV\|with_send_types\|current_send_types" src/` — ZERO (the thread-local is gone)

Commit nothing — the orchestrator weighs the diff and commits on green.
