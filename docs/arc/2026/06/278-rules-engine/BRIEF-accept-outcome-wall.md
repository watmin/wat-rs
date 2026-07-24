# BRIEF — the `accept'` OUTCOME WALL (peer-lifecycle Strike 3)

> **The work in one paragraph.** `:wat::kernel::accept'` returns a bare `Peer'<R,S>` and RAISES on its
> *handleable* failures (rendezvous dropped/shutdown, decode error, `select` error, `peer_cred` read fail).
> Per the peer-lifecycle LAW (2026-07-23) — *"we deliver an enum for code to handle exceptions with; raise
> is uncatchable on purpose, a thing that must never happen"* — every handleable failure becomes a matchable
> `:wat::kernel::AcceptOutcome<R,S>` variant. This is the recv'/send' wall pattern applied to accept',
> mirroring `RecvOutcome<O>` (Impure — `Accepted` holds a live `Peer'`). **Small sweep** (grep shows 2 live
> wat sites; the rider re-scouts via the CHECKER, not a grep — the recv' lesson).

## The shape — RULED (four-questions + grounding), do not re-fork
```clojure
;; mirror RecvOutcome<O> (types.rs:1169) — PARAMETRIC + Impure (Accepted holds a live Peer').
(:wat::core::defenum :wat::kernel::AcceptOutcome<R,S> :wat::enum::Impure
  :Accepted [peer <- :wat::kernel::Peer'<R,S>]  ;; an AUTHORIZED peer connected (success)
  :Closed   []                                   ;; the listener's rendezvous shut down / address dropped (clean; no peer)
  :Failed   [cause <- :wat::kernel::Failure])    ;; decode / select / peer_cred io error
```
**`Rejected` is CUT** — grounded: the security gate BOUNCES a stranger INTERNALLY (process tier: "drop +
re-poll", `listener.rs:370`; thread tier: no gate, "the crossbeam handle IS the grant"). No tier returns a
security-reject to the caller → a `Rejected` variant would never be constructed → fails Honest. **DO NOT change
the bounce** (four-Q ruled A: surfacing it is a behavior change + a DoS amplifier, out of scope for the wall).

## Read in order (the rooms)
1. `src/types.rs:1149-1188` — the `RecvOutcome<O>` `register_builtin` — **the parametric+Impure exemplar** to mirror.
2. `src/kernel/listener.rs:478` — `accept_as_value` (the conversion seam — wraps `accept()`, returns Peer-as-Value today).
3. `src/kernel/listener.rs:94-138` — the THREAD-tier `accept()` (its raises: `Disconnected`/`Shutdown`, `DecodeError`, Tuple-unpack).
4. `src/kernel/listener.rs:324-417` — the PROCESS-tier `accept()` (its raises: `select` err, `peer_cred` fail, "interrupted by shutdown"; and the security bounce at ~:350-370 — **leave it**).
5. `src/runtime.rs:21133` — `eval_accept_prime` (arity + listener-type-mismatch raises STAY; delegates to `accept_as_value`).
6. `src/check.rs:11181` — `infer_accept_prime` (returns `Peer'<R,S>` today → make it `AcceptOutcome<R,S>`).
7. `src/check.rs` — `MUST_USE_PARAMETRIC_HEADS` (the `["wat::kernel::RecvOutcome", "wat::spawn::ServiceEvent"]`
   array) — **add `"wat::kernel::AcceptOutcome"`** (parametric head, bare-FQDN no leading colon).
8. the recv' wall commit (`ee522630`) + `DESIGN-peer-lifecycle-outcome-walls.md` — the campaign playbook.

## The eval/outcome disposition (both `accept()` impls + `accept_as_value`)
The raises live in the `accept()` trait impls; `accept_as_value` wraps them. Convert so `accept_as_value`
returns an `AcceptOutcome` VALUE (construct the enum value; the `Accepted` payload is the wrapped `Peer'`).
Mechanism is the rider's judgment (e.g. change `accept()` to return a distinguishable `Result<Peer, AcceptFail>`
where `AcceptFail::{Closed, Failed(cause)}`, then `accept_as_value` builds the outcome), but the OUTCOME MAPPING is fixed:

| current raise (listener.rs) | tier | → AcceptOutcome |
|---|---|---|
| `Disconnected`/`Shutdown` → "address dropped or shutdown" (~:105) | thread | `Closed[]` |
| "interrupted by shutdown" (~:411) | process | `Closed[]` |
| `DecodeError` → "rendezvous recv decode error" (~:115) | thread | `Failed[cause]` |
| "accept' select: {}" (~:342) | process | `Failed[cause]` |
| "peer_cred on accepted socket: {}" (~:364) | process | `Failed[cause]` |
| Ok(peer) (an authorized peer) | both | `Accepted[peer]` |
| the security bounce (stranger) | process | **UNCHANGED** — stays internal drop+re-poll (NOT an outcome) |
| Tuple-unpack "malformed connect-request" (~:134) | thread | see STOP-3 |
| arity / listener-type-mismatch (`eval_accept_prime`) | — | **STAY raises** (must-never-happen; checker-prevented) |

Use the canonical `message-only-failure` Failure constructor that send'/recv' `Lost` use — NOT a hand-rolled
`struct-new` Failure (R57's Struct-Failure mask).

## `infer_accept_prime` + must-use
- `infer_accept_prime` (check.rs:11181) currently returns `TypeExpr::Parametric { head: "wat::kernel::Peer'", args: [r,s] }`.
  Change to `head: "wat::kernel::AcceptOutcome"`, same `args: [r,s]` (R then S — match accept's current `Peer'<R,S>` order).
- Add `"wat::kernel::AcceptOutcome"` to `MUST_USE_PARAMETRIC_HEADS`. `push_must_use_error` may need an accept' branch
  for the verb name (mirror how it names recv'/poll'); if the generic message suffices, leave it.

## The sweep (checker-scouted — NOT a grep)
Once `infer_accept_prime` returns `AcceptOutcome`, run `target/release/wat --check` across the corpus (or the
floor) to find EVERY site that now faces an unfaced outcome. Grep shows 2 (`tests/comms/probe_arc272_6a_capability_handoff.wat:20`,
`tests/comms/probe_arc209_c0b1_thread_connection.wat:13`) — but the grep UNDERCOUNTS (the recv' lesson: it missed
6 files + an embedded-wat site). Face every checker-named site: `(match (accept' l) (AcceptOutcome::Accepted p) …
(AcceptOutcome::Closed …) (AcceptOutcome::Failed c …))`. Per-site: `Closed`/`Failed` where a gone listener is
fatal → `assertion-failed!`; in an accept-loop → break/continue. **Atomic** — no green state where accept' returns
the outcome but a site drops it.

## The probe (RED-first)
`tests/comms/probe_arc278_accept_outcome_wall.{rs,wat}` (or extend an existing accept' fixture):
- accept' a happy authorized peer → `Accepted[peer]` (structural `Value::Enum` assert; then use the peer to prove it's live).
- accept' on a listener whose address/rendezvous is dropped/shut down → `Closed[]`.
- (if cheaply reachable) a decode/io error → `Failed[cause]`; else assert via the eval mapping + say so (no faking).
RED before the eval change (accept' raises / returns bare Peer'), GREEN after.

## STOP triggers (rejection criteria — halt + surface, do NOT improvise)
- **STOP-1:** if the checker-scout finds accept' sites in NON-test PRODUCTION wat (a stdlib `:wat::` caller), the
  sweep is bigger than a probe-only wall — surface the full site list before sweeping.
- **STOP-2:** if the `accept()` error paths CANNOT be cleanly distinguished into Closed-vs-Failed (a single opaque
  error type lumps shutdown with decode-error), STOP — the split needs the error kinds distinguishable; report the
  actual error surface so the shape can be re-weighed (maybe 2 variants Accepted/Failed, not 3).
- **STOP-3:** the thread-tier Tuple-unpack "malformed connect-request" (~:134) — is it must-never-happen (an
  in-process substrate bug: the crossbeam connect' built a bad request) or handleable (a wire-corrupted request)?
  Ground it: thread tier is in-process (crossbeam) → a malformed request is a SUBSTRATE BUG → lean **stays a raise**
  (must-never-happen). If you find it can carry adversarial/wire data, surface it — do not silently pick.

## Weigh (the orchestrator re-runs; do NOT trust the report)
- the RED probe: RED before, GREEN after.
- **the floor: `cargo nextest run --release`, read the Summary line** (never a piped exit). Expected 4213/0 + the
  new probe green. Any OTHER new RED = a swallow site the scout missed → STOP-1.
- content-integrity: the diff is types.rs + listener.rs + runtime.rs (eval) + check.rs (infer + must-use) + the
  2 faced fixtures + the new probe. Nothing else moved. Do NOT touch the security bounce, the `.bad` fixtures, or
  the recv'/send'/poll'/close' walls.

## Copy for shape
The recv'-must-use commit (`ee522630`) is the closest twin (parametric `RecvOutcome<O>`, must-use, small sweep).
`BRIEF-close-outcome-wall.md` (Strike 2) for the eval-disposition-table + earned-rune-probe pattern.
