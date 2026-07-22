# BRIEF — S1: the `recv'` outcome wall (substrate reshape + drive the cascade to green)

> Full design: `DESIGN-recv-outcome-wall.md` (read it first — the wall, the ratified enum, the measured 4×2
> evidence). This is the ROOT closure of the no-hidden-failures class (builder: *"make us never blind to
> errors again"*). Substrate-as-teacher strike: make the `recv'` contract honest, then drive the cascade the
> exhaustive match creates to a green floor.

## The work (one paragraph)
`recv'` today RETURNS the message and RAISES on close/crash (`eval_recv_prime`, `runtime.rs:26310`) — an
unwinding raise that masks (it blows past the reader; a crash reads as a clean close on the RuntimeError path).
Reshape it to return a matchable enum so mute is unrepresentable: add `:wat::kernel::RecvOutcome<O>` (Impure)
with `:Message [msg <- :O]` · `:Closed []` (clean EOF ONLY) · `:Lost [cause <- :wat::kernel::Failure]` (abnormal
loss — NEVER reason-free); make `recv'` return it (both tiers); extend `serve-dispatch-op'` to broadcast the
reason-free crash sentinel on the RuntimeError arm too (so a client is never blind to a crash-vs-clean); then
drive every `recv'` site (the exhaustive-match cascade — the METER, not a crisis) to a green floor.

## ⛔ STOP-0 — prove the Failure carries FIRST (the one risky composition; HARD STOP)
The cause is a **structured `:wat::kernel::Failure`**, NOT a String (builder-ruled: wat is EDN everywhere; a
String is a prompt-inject hack). But `crash_tx` today sends a **String** (`spawn.rs:678` panic →
`assertion_failure_envelope(&a)`; `:685` RuntimeError → `re.to_string()`). Before the cascade, PROVE a
`RecvOutcome::Lost(Failure)` can carry the structured crash cause end-to-end on the REAL path — a ~15-line probe:
a service whose op-handler crashes, the owner reads its Handle, `(match (recv' h) ((RecvOutcome::Lost cause)
<the cause is a Failure whose /message contains the crash sentinel>) …)`. Precedent for building a `Failure`:
`test.wat:705 failure-from-thread-died`; the crash forms are already structured (`AssertionFailure`,
`ProcessPanics`, `DivisionByZero`). **If the structured `Failure` cannot be carried cleanly** (e.g. `crash_tx`'s
`String` type is load-bearing and reshaping it to carry a structured payload is a separate large change), **STOP
and surface the exact blocker** — do NOT fall back to a String stuffed into a `Failure` (that is the prompt-inject
hack the builder ruled out). The crash reason IS structured at the source (`a: AssertionPayload`, `re:
RuntimeError`) — the question is only whether it survives to `recv'` as structure.

## Read in order (the rooms — grounded 2026-07-22)
1. `DESIGN-recv-outcome-wall.md` — the wall + the ratified enum + the measured evidence.
2. `src/runtime.rs:26310` (`eval_recv_prime`, THREAD arm) + `:26359` (PROCESS arm) — where `PeerRecvError::Crashed`
   → a `RuntimeError{MalformedForm{reason}}` raise today; both must instead RETURN `RecvOutcome::Lost(Failure)` /
   `::Closed` / `::Message`. `Ok(v)` → `Message(v)`; `Disconnected` → `Closed`; `Crashed(reason)` → `Lost(<Failure>)`.
3. `src/check.rs:11903` (`infer_recv_prime`) — return `RecvOutcome<O>` (not `O`).
4. `src/runtime.rs:27519` (`eval_kernel_serve_dispatch_op_tail`) — the `Err(payload)` panic arm broadcasts the
   reason-free `PEER_CRASHED_SENTINEL` to `clients`; the `Ok(result) => result` arm passes a wat `RuntimeError`
   (a normal `Err`) through with NO broadcast. Extend: when `result` is an `Err` (a crash bubbling out), broadcast
   the sentinel to `clients` too, THEN propagate — so a client on a RuntimeError crash gets `Lost`, never a mute EOF.
5. `src/kernel/spawn.rs:678`/`:685` (`crash_tx.send`) + `:330` (`Handle::recv` → `PeerRecvError::Crashed`) — the
   owner-delivery path (already works; STOP-0 is about carrying it as a `Failure`, not a String).
6. `wat/spawn.wat:172` (`ServiceEvent` — the exact `:Lost [idx cause <- :wat::kernel::Failure]` sibling to mirror;
   define `RecvOutcome` beside the `:wat::kernel::` types — ground where they live). `wat/service.wat`,
   `wat/bracket.wat` — the STDLIB `recv'` sites (the generated `defservice` op-call + `/stop`/`/hibernate`/grant/
   revoke `recv' (Handle/handle h)` sites — fix the macro's emission once; the client op-call must, on `Lost`,
   surface a reason-free 500 (discard the cause — the client never gets the reason), and the owner methods
   `eprintln` the cause on `Lost` (loud, terminal)).

## The cascade (drive to green — substrate-as-teacher)
After the reshape, `cargo build --release` + the test corpus go red: every `(recv' p)` now yields `RecvOutcome<O>`,
not `O`. Run cargo, read the errors, wrap each site in `(:wat::core::match (recv' p) -> T ((RecvOutcome::Message m)
…) ((RecvOutcome::Closed) …) ((RecvOutcome::Lost cause) …))`, iterate the fail-count to zero (it IS the progress
meter). ~160 `recv'` sites; the `defservice` macro consolidates many (fix its emission once). The idiom by caller
role: owner/main that won't recover → `((Lost cause) (eprintln (Failure/message cause)))`; client op-call →
`((Lost _) <reason-free 500>)`; a genuine terminate → `((Closed) <done>)`; `((Message m) …)` the happy path.

## RED gate (acceptance)
Create `tests/services/probe_arc278_recv_outcome_wall.{rs,wat}` (from `probe_arc278_crash_split_measure.{rs,wat}`
— the disconfirming probe already proves the current surface). Assert, all four paths (panic/rterr × thread/
process): the **ADMIN** (`Handle/handle`) `match`es `RecvOutcome::Lost cause` (a VALUE, not a raise) and
`(Failure/message cause)` contains the crash sentinel; the **CLIENT** `match`es `RecvOutcome::Lost` (never
`Closed`) and its cause message does NOT contain the sentinel (a reason-free 500). GREEN when the reshape lands.

## Deliverable + weigh
`cargo nextest run --release` → 0 NEW failures (Summary line; the known `wat-cli sigterm…polling_contract` flake
passes isolated); the RED gate green all four paths; `every_wat_scripts_file_loads` green (the wat stdlib loads).
The orchestrator re-runs `--release` by its own hand and reads the Summary — never a piped exit.

## STOP triggers (halt + surface; never improvise)
- **STOP-0** (above) — the structured `Failure` cannot carry from the crash site to `recv'`. The crux; do not
  fall back to a String-in-a-Failure.
- **STOP-1** — the cascade is unbounded / a `recv'` site's correct handling is genuinely undecidable (a site where
  the caller-role — owner vs client vs terminate — is ambiguous). Surface the site; do not guess the disposition.
- **STOP-2** — reshaping `recv'` forces a change to `select'`/`poll'`'s `ServiceEvent` contract (it should NOT —
  they already return matchable enums; only the point-to-point `recv'` changes). If it does, STOP — the blast
  radius was mis-scoped.
- Do NOT touch the uncommitted self-scheduling macro WIP (`wat/service.wat` +332 is Stone 2-A — leave it; but the
  `recv'` sites in the COMMITTED service.wat you edit are in-scope). Do NOT commit — the orchestrator weighs + commits.

## Blast radius
`src/runtime.rs` (`eval_recv_prime`, `serve-dispatch-op'`), `src/check.rs` (`infer_recv_prime`), `src/kernel/spawn.rs`
(only if STOP-0 needs `crash_tx` to carry a `Failure`), the `:wat::kernel::RecvOutcome` def, the wat STDLIB + the
test-corpus `recv'` sites (the cascade), and the new RED gate. NO change to `select'`/`poll'`/`ServiceEvent`.
Any `.wat` corpus form-change that is a mechanical rename → a wat-fix codemod, not hand edits.
