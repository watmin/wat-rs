# BRIEF — Stone rs-2: the `:Stop` terminal op (a service's value is its final state)

> Single-hop sonnet Shadowdancer. Do NOT spawn sub-agents. Work only in `~/work/holon/wat-rs`. Commit
> nothing; the orchestrator weighs + re-runs the gate. Grounded against HEAD `dfd77a4f`.

## The work (one paragraph)

A service's final state comes back as a `:Stop` REPLY over the client connection (gen_server `{stop,
State}` — the banked C.4 terminal op). defservice gains: (1) an `Outcome::Stop` variant; (2) a `serve`
dispatch arm that, on `Outcome::Stop`, sends the reply to the client and EXITS the loop (no recur);
(3) an AUTO-generated `stop` op (`Op`/`Reply` variant + handler returning `Outcome::Stop(state, state)`)
and a client method `(<svc>/stop c) -> <state-ty>` that returns the final state. CONSTANT SHAPE across
thread/process/remote — it rides the client connection (`connect'`/`send'`/`recv'`), no new substrate, no
lineage reshape. `serve` STAYS `-> :nil` (the state travels as the reply, not serve's return).

## Build (in `wat/service.wat`)

**1. `Outcome` enum** (service.wat:48, `defenum :wat::service::Outcome<S,R>` — currently `:Reply [new-state resp]`):
add `:Stop [final-state <- :S  resp <- :R]` — gen_server `{stop, NewState, Reply}`.

**2. `serve` per-op outcome-match** (service.wat ~325-340 — currently only matches `Outcome::Reply` →
`(do (send' (nth clients idx) (reply-variant resp)) (serve self l clients new-state))`): add a `:Stop`
arm for EVERY op so a handler may terminate:
```
((:wat::service::Outcome::Stop final-state resp)
  (:wat::core::do
    (:wat::kernel::send' (:wat::core::nth clients idx) (~reply-variant-kw resp))
    nil))            ; EXIT — return nil, do NOT recur; the loop ends, the wrapper/peer exits cleanly
```
(`:Reply` still recurs; `:Stop` exits. Deadlock-free: the same clean owner-drop→exit path, just initiated
by a client request — the peer ends, the owner's Handle drop reaps as today.)

**3. The AUTO `stop` op** (defservice always emits it, NOT from `:ops`):
- `Op` enum gains a `:Stop [req <- :<fqdn>::StopRequest]`; `Reply` gains `:Stop [resp <- :<fqdn>::StopResponse]`.
- Records: `<fqdn>::StopRequest []` (empty) + `<fqdn>::StopResponse [state <- ~state-ty]` (carries the final state).
- serve dispatch arm for `Op::Stop`: the auto-handler returns `(:wat::service::Outcome::Stop s (:<fqdn>::StopResponse s))`
  → matched by the `:Stop` outcome-arm above → sends `Reply::Stop(StopResponse s)` + exits.
- client constructor `<fqdn>/stop-request` (nullary) + method `(:wat::core::defn <fqdn>/stop [c <- ~client-peer-ty] -> ~state-ty …)`:
  `send'` `Op::Stop(StopRequest)` over `c`, `recv'` the `Reply`, match `Reply::Stop` → `(StopResponse/state resp)`.
  (Mirror the existing per-op method codegen, service.wat ~430-470; stop is nullary — no request fields.)

## Already proven / do NOT touch
- The crash path is EXISTING + locked by `probe_arc272_rs2_crash_surfaces_to_client` (GREEN): a crashing
  handler → the client's call raises. Your change must keep it green (don't catch/swallow handler raises
  in serve).
- The `:Reply` path (c3/headline/deftests) must stay green — additive only.
- Do NOT reshape the lineage/`Address'` handoff (that was the rejected approach).

## Rooms (read in order)
1. `wat/service.wat:40-55` (Outcome), `:60-115` (binders: enum-name/reply-name/peer-ty/client-peer-ty/state-ty/fqdn-str),
   `:300-365` (serve dispatch + the per-op outcome-match + serve-body), `:420-470` (per-op constructors + methods),
   `:480-590` (start + the final `do` assembly — where the auto stop op's records/variants/method get spliced).
2. `tests/probe_arc272_rs2_thread_stop_returns_final_state.rs` (the GATE) + `…_crash_surfaces_to_client.rs` (keep green).
3. `tests/probe_arc209_c3_defservice_client_face.rs` (the :Reply client-face shape to mirror).

## ADD a process gate
Create `tests/probe_arc272_rs2_process_stop_returns_final_state.rs` — the thread probe with
`(:wat::spawn::process)` instead of `(:wat::spawn::thread)` (constant shape; the reply crosses the socket).
It must go GREEN (proves `stop` is locus-agnostic).

## STOP triggers (halt + report)
1. STOP if exiting serve on `:Stop` cannot be done cleanly (the loop must end + the peer reap without a
   hang) — deadlocks are intolerable; report rather than ship a hang.
2. STOP if the auto stop op collides with a user op named `Stop`/`stop` — report (we'll reserve the name).
3. STOP if `recv'` of the `Reply` for the stop method needs a `-> :T` ascription (must infer — 258.5b).
4. STOP if `StopResponse [state <- ~state-ty]` mis-types because `~state-ty` is a bare scalar — report
   (the state-must-be-a-record rule is deferred to arc 273; i64 state must still work for now).

## Gate (orchestrator re-runs)
- `cargo test --release -p wat --test probe_arc272_rs2_thread_stop_returns_final_state -- --include-ignored --test-threads=1` → GREEN (5); `#[ignore]` removed.
- `cargo test --release -p wat --test probe_arc272_rs2_process_stop_returns_final_state -- --include-ignored --test-threads=1` → GREEN (5).
- `cargo test --release -p wat --test probe_arc272_rs2_crash_surfaces_to_client -- --test-threads=1` → GREEN (still).
- `cargo test --release -p wat --test probe_arc209_c3_defservice_client_face -- --test-threads=1` → GREEN (5).
- `cargo test --release -p wat --test test -- counter 2>&1 | grep "test result"` → the locus-parity deftests GREEN.
- `cargo test --release -p wat --lib -- --test-threads=1 | grep "test result"` → 929/36 (zero new).
- `cargo test --release -p wat --test nursery -- --test-threads=1 | grep "test result"` → 893/4 baseline.
- `cargo build --release -p wat` → clean.

Report: exact files+lines changed, how the auto stop op + serve `:Stop` exit were emitted, the gate
results from your OWN runs (pasted), and any STOP hit.
