# DESIGN — the recv' outcome wall: a peer-read never surfaces a failure without a matchable reason

> **THE LAW (builder, 2026-07-22):** *"i do not care about how wide the blast radius is — the cost of never
> seeing a fucking masked error is worth it. make us never blind to errors again."* This is the ROOT closure
> of the failure-masking class — the top rung of the extirpare ladder. Every prior kill (Mechanism A,
> eprintln-terminal, the transport twin `RecvError::Failed`, the RST `PeerCrashed`, startup-crash honesty)
> bound a *known* mute site; none made mute **unrepresentable**, so the class regrew (breadcrumb: *"we've
> killed them like 5 times AND THEY REFUSE TO DIE"*). This stone makes a reason-free failure **unconstructible**.

## The class, unified (all one disease: a failure surfaced without a matchable reason)

Measured this session (`tests/services/probe_arc278_crash_split_measure.{rs,wat}`, the RED-gate seed):

| crash → | **CLIENT** (connected peer) | **ADMIN** (owner reads `Handle/handle`) |
|---|---|---|
| panic · thread  | reason-free, honest (*"abnormal far-side crash … administrative … owner's crash channel"*) | ✓ full `AssertionFailure` — **as a raise** |
| panic · process | reason-free, honest | ✓ full `ProcessPanics[Panic[…]]` — **as a raise** |
| **rterr · thread**  | **bare `"peer closed / channel disconnected"`** — indistinguishable from a clean EOF | ✓ full `DivisionByZero` — **as a raise** |
| **rterr · process** | **bare `"peer closed / channel disconnected"`** | ✓ `ProcessPanics[RuntimeError[…]]` — **as a raise** |

Two grounded facts settle the scope:
1. **The admin ALWAYS gets the exact reason — all four paths.** No tear-down, no EPIPE. So this is a
   **reshape of the surface, not a build of missing delivery.**
2. **The admin gets it as an unwinding `recv'` RAISE, never a matchable enum** (R41's mechanism — a raise
   unwinds past the reader, which is itself a masking); and **the client's 500 is a mute on the RuntimeError
   path** (`serve-dispatch-op'` only broadcasts on a *panic*, so a `RuntimeError` crash reads to the client as
   a clean close — the exact `recv': peer closed` the self-scheduling macro hit).

## The wall (top rung — make mute UNREPRESENTABLE)

A peer-read yields a **matchable outcome enum** with exactly three shapes (mirroring the reason-bearing
`ServiceEvent` that `select'`/`poll'` already return):

```clojure
;; NAMES intueri-RATIFIED (2026-07-22) — ServiceEvent::Lost parity minus the multiplex `idx`; builder-ruled
;; the structured cause (wat is EDN everywhere; a String is a prompt-inject hack — use the structured carrier).
(:wat::core::defenum :wat::kernel::RecvOutcome<O> :wat::enum::Impure   ;; Impure like ServiceEvent (an I/O outcome)
  :Message [msg   <- :O]                                               ;; a real message
  :Closed  []                                                          ;; a GENUINE clean EOF — the ONLY reason-free terminal
  :Lost    [cause <- :wat::kernel::Failure])                           ;; abnormal loss — UNCONSTRUCTIBLE without a structured cause
```

Names (intueri-cast + weighed against the settled `ServiceEvent`/`Reply::Failed` anchors): the death variant is
**`Lost`, not `Crashed`** — `Crashed` *lies* (it fires on any abnormal break, incl. a transport loss / ECONNRESET
that is not a crash); `Lost` is the honest superset and the exact `ServiceEvent::Lost{cause}` word. The message
variant is **`Message`** (a recv `Value` and `ServiceEvent::Message` are one concept), and the cause field is
**`cause <- :wat::kernel::Failure`** — the first-class structured carrier `ServiceEvent::Lost`/`Reply::Failed`
already use (never a flat String: crash reasons are structured EDN — `AssertionFailure`/`ProcessPanics`/
`DivisionByZero` — so they get the structured `Failure`; precedent for building one: `test.wat:705
failure-from-thread-died`).

The three structural guarantees that make blindness have no form:
- **`Lost` cannot exist without a cause** — the variant's mandatory structured `Failure` field. A reason-free
  abnormal loss has no constructor.
- **`Closed` (reason-free) is producible ONLY from a genuine clean EOF** — never from an error path. This is the
  discipline the lint backstops; an abnormal loss can never masquerade as a clean close.
- **The cause is a VALUE the caller must handle**, not a silent unwind. There is no path where "the peer is
  gone" arrives without also telling you *clean vs lost* — and a loss always carries whatever cause is on
  *that caller's channel* (the full `Failure` for the owner; the honest reason-free "administrative" `Failure`
  for a client — it learns a loss happened, never the reason).

### How the ruling falls out of one enum + two channels (no per-caller enum)
Same enum; the *channel* determines the `Failure` it carries, and the *caller* decides how to handle it:
- **CLIENT** (connected peer) → `Lost(<reason-free administrative Failure>)` — a **500**; the client knows a
  loss happened, gets **no reason**. Handling: surface a reason-free error. *(Never `Closed` on a loss — that
  is the mute we are killing.)*
- **ADMIN / owner** (`Handle/handle`) → `Lost(<full Failure>)`. Handling: the "real final caller" who does not
  recover **`match`es it and `eprintln`s the cause → loud, terminal** (R51 `eprintln` IS the dying declaration;
  R52 explicit-exception-paths, verbosity is the shield). Crashing is a bug, so the reason is **always known**.

## The three moves (all reshape/extend a working delivery — none is a build)

1. **`recv'` : raise → matchable `RecvOutcome<O>`.** `eval_recv_prime` (`runtime.rs:26310`) stops building a
   `RuntimeError{MalformedForm}` from `PeerRecvError`; it returns `RecvOutcome::Crashed(reason)` /
   `RecvOutcome::Closed` / `RecvOutcome::Value(v)`. `infer_recv_prime` (`check.rs:11903`) returns
   `RecvOutcome<O>`. Both tiers (thread `runtime.rs:26310`, process `:26359`).
2. **Client honesty on the RuntimeError path.** `serve-dispatch-op'` (`runtime.rs:27519`) currently broadcasts
   the reason-free `PEER_CRASHED_SENTINEL` to clients **only on a panic** (`Err(payload)` arm); a `RuntimeError`
   slips through `Ok(result) => result` with no broadcast → the client's read is a bare EOF (`Closed`-looking).
   Extend the broadcast to the `RuntimeError` case too, so a client on **any** crash kind gets `Crashed(administrative)`,
   never a mute `Closed`. *(This annihilates the exact failure the macro surfaced.)*
3. **The lint backstop (stem #7 becomes a build error).** A sibling of `no_inlined_wat`: RED-flag any
   `=> …Disconnected` / `map_err(|_| …Disconnected/Closed)` in the recv paths (`comms/`, `channel/`,
   `kernel/spawn.rs`, `runtime.rs` recv arms) — a reason-free terminal may be constructed only from a genuine
   clean EOF. This is the check the previous five kills never planted; it makes the class unable to regrow.

## The corpus sweep (160 `recv'` sites — the blast radius, accepted)
Every `(recv' p)` becomes a `match` over `RecvOutcome`. The exhaustive-match cascade **drives** the sweep
(substrate-as-teacher: each red site names the next). Edit-only riders, orchestrator weighs CENTRALLY once
(FM 18 — never per-rider cargo). Idiomatic handling by caller role:
- an owner/`:user::main` that does not recover → `((RecvOutcome::Lost cause) (eprintln cause))` (loud, terminal) ;
- a client op-call → `((RecvOutcome::Lost _) <a reason-free 500>)` ;
- a genuine terminate → `((RecvOutcome::Closed) <done>)` ;
- `((RecvOutcome::Message m) …)` the happy path.
The generated `defservice` client-face op-call + `/stop`/`/hibernate`/`grant`/`revoke` (`service.wat` — they
`recv'` the Handle peer) are the highest-value sites; the macro emits the `match` once.

## The ONE contract decision (pinned)
`recv'` returns `RecvOutcome<O>` (flat 3-variant enum), NOT a raise, NOT a `Result<O,E>` nesting, NOT a
narrower crash-only reshape that leaves a raise path alive (ruled out — a live raise path is not the root).
Rationale: the flat enum is the exact shape `select'`/`poll'` already return (`ServiceEvent`), it forces every
reader to be explicit (R52), and it is the only shape under which mute is unrepresentable (the builder's ruling:
blast radius is accepted; the root, not a stem-cut).

## RED gate (acceptance — the measurement probe, reshaped to a gate)
`probe_arc278_crash_split_measure.{rs,wat}` → `probe_arc278_recv_outcome_wall.{rs,wat}`. Assert, all four
paths (panic/rterr × thread/process):
- **CLIENT** never gets `Closed` on a loss — it gets `RecvOutcome::Lost(<reason-free Failure>)` (a 500), and the
  cause's message does NOT contain the crash sentinel; and
- **ADMIN** (`Handle/handle`) gets `RecvOutcome::Lost(<Failure>)` as a **matchable value** (not a raise):
  `(match (recv' …) ((RecvOutcome::Lost cause) …assert (Failure/message cause) contains the sentinel…) …)`.
At HEAD: recv' raises (no `RecvOutcome` to match) → RED. GREEN when the three moves land.

## Out of scope (affirmative cuts — not deferrals)
- **The self-scheduling / item (c) macro** resumes AFTER this closes (R50 — the ruin forges the way; this is
  the ruin blocking the macro). The uncommitted Stone 2-A macro WIP stays on disk, untouched.
- **STOP-2** (crash-broadcast of the *full reason* to `connect'`-ed clients) stays out — clients get a
  reason-free 500 by ruling; the full reason is the owner's. `#[ignore]`'d probe unchanged.
- **The over-budget `FrameTooLarge` mute** (breadcrumb "STRUCTURAL CLOSURE" Stone 1) is the SAME wall — folds
  in here (its `=> Disconnected` at `transfer.rs:176` is exactly what the lint forbids and the `RecvOutcome`
  reshape reasons). One wall, one sweep.
- **Parametric purity for user `defenum`s** (NAMED, not deferred — no owning arc yet). `RecvOutcome<O>` is
  *precisely* pure-iff-`O`-pure, but user enum purity is DECLARED/fixed (`check.rs:14072`); only built-in
  containers (`Vector`/`Option`) get "pure iff args pure" (`check.rs:14103`). `Impure` is the honest fixed
  approximation (never lies — a `Pure` marking would lie the moment `O` is a live resource, and forbid
  recv'-ing one). If a future need surfaces a wire-mobile pure-`O` `RecvOutcome`, that is a substrate stone
  (parametric enum purity); it does NOT block this wall (the diagnostic `Closed`/`Lost[Failure]` are pure regardless).

## Sequencing (the stones)
1. **S1 — the enum + the recv' reshape** (`RecvOutcome<O>`; `eval_recv_prime`/`infer_recv_prime` both tiers).
   The cascade goes red across the 160 sites — that is the meter, not a crisis.
2. **S2 — the client-honesty broadcast extension** (`serve-dispatch-op'` broadcasts on the RuntimeError arm).
3. **S3 — the corpus sweep to green** (edit-only riders, central weigh; the `defservice` macro emits the match).
4. **S4 — the lint wall** (the reason-free-terminal-from-error build error).
5. Weigh `cargo nextest run --release` → 0 new; the RED gate green all four paths; **then** item (c) resumes.

> Cross-ref: `DESIGN-no-hidden-failures.md` (the LAW + the "STRUCTURAL CLOSURE" section this completes);
> R41 `EGO SVM LEX` (the recv'-raise mechanism this corrects — a new realization owed at close: R41's
> mechanism was wrong, the enum is the fix); R50 `RVINA VIAM FABRICAT` (the ruin forges the way — this
> unblocks the macro); R51 `TYPO TANGO` + R52 `QVOD LEX ACCENDIT` (typed effect channels + explicit-exception
> paths, the verbosity the shield).
