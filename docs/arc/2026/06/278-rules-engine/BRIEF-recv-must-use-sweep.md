# BRIEF — recv'-must-use sweep: face the 16 discarded-recv' sites (the symmetric completion)

> **Tier:** sonnet shadowdancer. **Arc:** 278, the recv'-must-use strike — the symmetric twin of the send'-wall
> (`DESIGN-send-outcome-wall.md`). R53 made `recv'` return a matchable `RecvOutcome<O>` (never a raise past the
> reader) — but a `_`-bound or do-dropped recv' outcome was still a silent SWALLOW (the R55 harness sin, at the
> class level). **The gate is ALREADY DONE** (I added it): `is_must_use_type` gained a parametric-head arm and
> `:wat::kernel::RecvOutcome` is now must-use, so a dropped recv' outcome — literal `recv'` OR a generated
> `:nature :Peer` client-method call (both `RecvOutcome<Response>`) — is a located compile error in both discard
> doors. This strike FACES the 16 pre-existing dropped sites so the floor is green with the gate on.

## DO NOT TOUCH (already done by the orchestrator)
- `src/check.rs` — the gate (`MUST_USE_PARAMETRIC_HEADS`, `is_must_use_type`, verb-aware `push_must_use_error`). Done.
- `tests/services/probe_arc278_recv_outcome_must_use_wall.{rs,wat.bad}` + `..._do.wat.bad` — the RED-gate probe. Done.
- The send'-wall probes/gate (`probe_arc278_send_outcome_must_use_wall*`). Done + must stay green.

## The 16 sites — ALL discarded generated client-method calls (RecvOutcome<Response> dropped)
The checker enumerated them (`./target/release/wat --check <file>` → "unhandled …RecvOutcome…must be faced").
13 are `_`-bound, 3 are do-position (no `_`):
```
wat-tests/service-telemetry-bridge.wat:118          _ (:wat-tests::Worker/work wc …)
wat-tests/service-locus-parity.wat:74               _ (:wat-tests::Counter/increment c …)
wat-tests/service-stop-resp.wat:66                  _ (:wat-tests::RespCounter/increment c …)
wat-tests/service-admin-facet.wat:53                _ (:wat-tests::AdminCounter/increment c …)
wat-tests/service-hibernate-resume.wat:55           _ (:wat-tests::HibCounter/increment c …)
wat-scripts/probes/arc-293/s4c-thread.wat:36        _ (:my::Counter/increment c …)
wat-scripts/probes/arc-293/s4c-messages-acceptance.wat:36   _ (:my::Counter/increment c …)
tests/process/probe_arc272_rs2_thread_stop_returns_final_state.wat:37    _ (:my::Counter/increment c …)
tests/process/probe_arc272_rs2_process_stop_returns_final_state.wat:40   _ (:my::Counter/increment c …)
tests/services/probe_arc272_6b_defservice_on_process.wat:39          _ (:my::Counter/increment c …)
tests/services/probe_arc209_c3_defservice_client_face.wat:40         _ (:my::Counter/increment c …)
tests/services/probe_arc272_rs1_state_must_be_record.wat:27          _ (:my::counter/increment c …)
tests/services/probe_arc209_locus_agnostic_start.wat:38              _ (:my::Counter/increment c …)
tests/services/probe_arc278_log_captures_call_line.wat:34    (:wat::telemetry::log span …)          [do-position]
tests/services/probe_arc278_span_nested.wat:22              (:wat::telemetry::Span/incr outer …)   [do-position]
tests/services/probe_arc278_span_macros.wat:21             (:wat::telemetry::Span/incr span …)     [do-position]
```

## The facing — UNIFORM (grounded: every site fires the request and genuinely does not use the response — it checks state separately, or it's a fire-and-forget metric/log). Wrap the client-call node in:
```clojure
(:wat::core::match <THE CLIENT CALL, verbatim>
  ((:wat::kernel::RecvOutcome::Message _resp) nil)                                              ;; response genuinely unused → discard
  ((:wat::kernel::RecvOutcome::Lost _c)
     (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message _c) :wat::core::None :wat::core::None))  ;; surface the transport death (visible chosen death, R53)
  (:wat::kernel::RecvOutcome::Closed
     (:wat::kernel::assertion-failed! "recv': peer closed" :wat::core::None :wat::core::None)))
```
- **`_`-bound sites:** `_ (CALL)` → `_ (:wat::core::match (CALL) …arms…)`.
- **do-position sites:** `(CALL)` (a do non-final) → `(:wat::core::match (CALL) …arms…)`.
- Message → `nil` (behavior-preserving: the site already dropped the response; the happy path proceeds exactly as
  before, so passing tests stay green). Lost/Closed → surface (a transport failure that was silently vanishing now
  crashes loudly with its cause — the whole point). This is the recv'-wall's own default
  (`wrap-client-method-match-in-recvoutcome.wat`), adapted for a bare dropped call.

## Method — the CHECKER IS THE WORKLIST (R52), iterate to zero
For each of the 16 files: face the flagged call, then re-run `./target/release/wat --check <file>` and confirm it
reports **0** "RecvOutcome…must be faced" errors. If facing one site reveals another in the same file, face that
too — iterate until each file is clean. (The scout counted 1 site/file, but trust the checker, not the count.)

## STOP triggers
- **STOP-0:** after all 16, the whole floor is not 0-failed → report which + why. Do NOT mass-edit.
- **STOP-1:** a site's context shows the response IS used (bound to a real name and consumed downstream, or the
  call is in FINAL position and its value flows somewhere) → do NOT blindly `nil` the Message arm; STOP + report
  that site — the facing must bind + forward the response, which is per-site judgment.
- **STOP-2:** a wrap turns a *passing* test red (not a transport failure — a paren/nesting error, or a Message-arm
  type mismatch) → STOP, report; the wrap must be behavior-neutral on the happy path.

## Verify (report; the orchestrator WEIGHS by its own `--release` re-run)
1. `cargo build --release` clean (you should NOT need to rebuild — check.rs is untouched; but confirm).
2. Each of the 16 files `--check`s with 0 RecvOutcome-must-use errors.
3. **Whole floor `cargo nextest run --release`** — report the Summary line (target 0 failed).
4. All four gate probes green:
   `probe_arc278_recv_outcome_must_use_wall::{discarded_recv_outcome_in_let_underscore_is_compile_error,
   discarded_recv_outcome_in_do_non_final_is_compile_error}` +
   `probe_arc278_send_outcome_must_use_wall::{…do…,…let…}`.
5. `git diff --stat`.

## Deliverable (do NOT commit — the orchestrator banks after its own weigh)
The 16 faced `.wat` files. Report: (1) each site's before→after wrap (a one-line each is fine); (2) any STOP hit;
(3) the floor Summary read by you; (4) the four probes green; (5) `git diff --stat`.

## Blast radius
Exactly the 16 `.wat` files above. NO `src/`, NO probe files, NO codemod (these are type-flagged per-site sites,
not a structural form a codemod can key on — hand-face them). Scratch logs → `/tmp/claude-scout/`.
