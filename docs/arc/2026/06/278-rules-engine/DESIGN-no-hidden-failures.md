# DESIGN — wat never hides a failure (the IPC death/error path)

> **THE LAW (builder, 2026-07-17):** *"i want wat to never hide failures ever again … this masking of
> failure is actively hostile against wat's intent."* Every place on the peer/service death path that
> discards an error, collapses distinct failures into one mute value, writes a reason to a closed pipe,
> or kills a whole service over one bad message is the SAME class — **failure-masking** — and this arc
> pulls the class out by the root. We own wat; the arc-294 "crash reasons are administrative" ruling does
> NOT shelter a masking behavior — we change our minds when the mask keeps blinding us.

## STATUS — 2026-07-17 (Mechanism A LANDED + verified + pushed; the transport-tier twin is NEXT)

**Half the law is real** (`66d6aed7`, pushed). The **alive-service-rejects-you** path is done —
Mechanism A: `poll'` returns `ServiceEvent::Malformed{idx, cause}` instead of raising; the serve loop
replies `Reply::Failed{cause}` and **keeps serving** (no more DoS); `recv'` surfaces `Reply::Failed` as
a catchable raise carrying the reason. A reserved `Reply::Failed[cause <- Failure]` variant on every
synthesized `<S>::Reply` (`types.rs synthesize_surface_protocol`) is the **protocol-tier completion**
of the 293 outcome-enum model (op-tier failure = `<Op>Response::{Transient,Fatal}`; protocol-tier
failure — "couldn't resolve your message to any op" — was the missing floor). Verified by own re-run:
`probe_arc278_dead_child_speaks` GREEN (caller carries `unknown tag #probe/Note … no matching struct
or enum`); floor 4168 passed / 1 failed = the standing `no_inlined_wat` lint at **351**, zero new;
`no_loose_string_assert` PASS.

**Sites addressed (of the table below):** #9 (`poll'` no longer service-fatal), #10 (the `_cause`
discard), and the protocol-tier gap (there was no `Reply::Failed` at all).

**TWO JUDGMENT CALLS awaiting the builder's ratification** (both grounded, non-blocking — see the
`66d6aed7` message): **(1)** surfacing lives in `recv'`, not a client-method match arm — a wat
`assertion-failed!` in a client method is `panic_any`, uncatchable by `eval_in_frozen`/`apply_function`;
`recv'` is the only catchable, uniform point (this doc's step 3 / "room 8"). **(2)** `:Lost`'s cause is
surfaced via `eprintln` (no reply target — the peer is dead; sending up the lineage `self` peer would
desync the admin request/reply protocol).

**NEXT — the transport-tier twin (the dead-service-speaks half):** a **genuinely crashed** service (a
real handler panic, not a decode rejection) still EPIPEs its reason and `RecvError` still has no slot.
Sites R, 1–8 remain: give `RecvError` a `Failed(String)` variant (`Disconnected` = clean EOF only); bind
`|e|` at every `map_err(|_| Disconnected)` in `comms/process.rs`; thread the reason through `recv'`
(`runtime.rs:26196/26219`, including `PeerRecvError::Crashed`); keep the crash channel's read end alive
so the dying child's envelope lands instead of EPIPE-ing. Its RED gate: a probe where a service's
HANDLER genuinely panics and the caller must carry *that* reason.

## SUB-STRIKE — `eprintln` is terminal (2026-07-18; closes `feedback_eprintln_is_terminal`)

Surfaced while ratifying the `:Lost` disposition: `eprintln` was **designed** as a dying declaration —
builder direction 2026-05-15 (`docs/arc/2026/04/109-kill-std/INVENTORY.md:1284`): *"eprintln is a 'we are
crashing, here's what I know' and exits"*; the kernel's three **terminating** forms are `eprintln` (value),
`panic!` (message), `assertion-failed!` (assertion shape). `COMPACTION-AMNESIA-RECOVERY.md:1847`: eprintln →
*exit code non-zero*. But the **implementation** (`services/verbs.rs:147 eval_kernel_eprintln`) writes to the
stderr service and returns `Value::Unit` — benign, non-terminal — and `USER-GUIDE.md:3577` documents it that
way. The doctrine was deferred ("pending — separate slice") and never closed. So the crash-with-message
primitive **silently doesn't crash** — the masking law's own shape, baked into the primitive. Builder:
*"eprintln was meant to be 'this is the last thing I'll say' … it is quite frustrating to see this."*

**The fix (ratified — build now, take the fallout):** `eprintln` (and `epprintln`) emit the value's EDN to
stderr, then **terminate non-zero** — uncatchable, uniform across loci (a panic → `emit_structured_exit` in a
forked child / kills the serve loop on a thread / non-zero exit in main), the same convention as
`assertion-failed!`; return type `∀T. T -> :()` (the terminating-form type, not `-> :wat::core::nil`). Then
the `:Lost` serve-loop arm (`service.wat:864`, already calling `eprintln`) is correct as written — an
abnormal transport break is an unexpected failure that lets-it-crash (OTP), and the reason lands on stderr
before exit.

**Fallout (the ~benign usages):** most are tests *of* eprintln (`ambient-stdio.wat`, `test.wat:184/206`,
`probe_arc255_epprintln`, `wat_arc170_slice_1f`) — they become tests of the **terminal** behavior (program
eprintln's → exits non-zero, value on stderr). One is an incidental mid-`let` diagnostic
(`tests/channel/…drain_and_join…:7 (eprintln "diag")`) that must migrate (→ `println`, or drop). **Open (do
NOT mint speculatively):** whether the substrate wants a *benign* stderr write at all — the immediate fallout
doesn't require one; if a use surfaces, its name is a separate intueri cast.

**RED gate:** a program running `(do (eprintln "dying words") (println "AFTER"))` must NOT emit `AFTER`
(eprintln terminated), must carry `dying words` on stderr, and must exit non-zero. At HEAD `AFTER` prints and
it exits 0 (benign). GREEN when eprintln terminates.

## The incident that surfaced it (grounded)

`tests/services/probe_arc278_journal_logs_on_process` — a `journal'` service forked to a **process**;
the client `write-logs` a `Log` whose `message` is a user record (`:probe::Note`). The caller gets a mute:

```
recv failed: peer closed / channel disconnected   (runtime.rs recv', line 28 of the fixture)
```

`strace -f -s 4000` on the forked child revealed the TRUTH the caller never saw:

```
[pid …] write(2, "#wat.kernel/ProcessPanics [ … poll' (process tier): client message decode failed:
        src/edn_shim.rs:2424:45: unknown tag #probe/Note (body shape: map);
        no matching struct or enum in the type registry … ]", 687) = -1 EPIPE (Broken pipe)
[pid …] exit_group(1)
```

The child **had** a rich, located reason (687 bytes), formatted it correctly, and **wrote it to a pipe
whose read end was already closed** — it vanished on `EPIPE`. The whole service died (exit 1) over one
undecodable client message, and the caller got a hardcoded "peer closed."

## The masking sites (the full class, grounded)

| # | site | what it hides |
|---|---|---|
| R | `comms/mod.rs:899` | `RecvError {Disconnected, Shutdown, FrameTooLarge}` — **no slot for a reason** (the root) |
| 1 | `comms/process.rs:944` | `from_wire` **decode failure** → `Disconnected` (`|_|`) — hid `unknown tag #probe/Note` |
| 2 | `comms/process.rs:940` | UTF-8 failure → `Disconnected` (`|_|`) |
| 3 | `comms/process.rs:579,752,884,887` | I/O read errors (errno) → `Disconnected` (`|_|`) |
| 4 | `comms/process.rs:992` | `FrameScan::Malformed` → `Disconnected` |
| 5 | `channel/transfer.rs:172` | `FrameTooLarge` → `RecvOutcome::Disconnected` (collapse downstream) |
| 6 | `runtime.rs:26196` | socket `recv_wire()` error → `|_|` → hardcoded "peer closed" |
| 7 | `runtime.rs:26219` | `peer.recv()` → `|_|` → hardcoded "peer closed" — **discards `PeerRecvError::Crashed(reason)`** |
| 8 | `spawn.rs` err channel | child's crash-reason write **`EPIPE`s** — read end torn down before the child speaks |
| 9 | `service.wat:791` (`poll'`) | a client decode failure is **service-fatal** — one bad message kills every client |
| 10 | `services/peer.rs:114,120` | malformed `Req` field → `continue` **without replying** → the caller **hangs** |

Sites 6/7 preserve a reason *one branch down* already (`runtime.rs:26209` binds `|e|` for the decode
door) — the codebase knows how; the recv-side just doesn't do it.

## The strike — climb to "a mute failure has no form"

**Ratified (four-questions, 2026-07-17): Option A** — a per-message decode failure is a **client-scoped
error replied to that caller**, not a service-fatal crash. (A: Obvious/Simple/Honest/Good-UX all YES.
B "service-fatal + reason to admin only" fails Obvious/Simple/Honest — one client must not be able to
kill a shared service, and the requester must not be left blind.) A does not contradict arc-294: it
*reclassifies* — bad input goes back to the sender; a **genuine** crash still goes to the creator (and
must no longer `EPIPE`).

Ladder, top rung = unrepresentable:

1. **Give `RecvError` a reason.** Add a failure variant that carries the message, e.g.
   `RecvError::Failed(String)` (decode / utf8 / io / malformed all become `Failed(reason)`).
   `Disconnected` then means **only** a genuine clean EOF. *Wall:* "a real failure indistinguishable
   from a clean close" now has no representation. Exhaustive `match` sites (`channel/transfer.rs:165-172`,
   the `Display` impl) go red until they handle it — the compiler drives the sweep.
2. **Bind the error at every collapse.** Every `map_err(|_| RecvError::Disconnected)` on sites 1-4 →
   `map_err(|e| RecvError::Failed(e.to_string()))`. Site 4/5 `Malformed`/`TooLarge` keep their distinct
   variants but the *reason* rides along.
3. **Surface it in `recv'`.** `runtime.rs:26196/26219` stop the `|_|` + hardcoded string; thread the
   `RecvError`/`PeerRecvError` message (esp. `Crashed(reason)`) into the raised `MalformedForm`.
4. **Keep the crash channel alive until the reason is delivered.** The `EPIPE` (site 8) means the err
   read end drops before the dying child writes. Hold it open across the child's death so
   `emit_structured_exit`'s envelope lands, and route it to the creator (the `Handle`).
5. **`poll'` replies, doesn't crash (Option A).** A client decode failure returns a client-scoped error
   event the serve loop replies with (the rich reason), and the service keeps serving. Kill site 10's
   `continue`-without-reply (every `Req` gets a reply — the ZERO-MUTEX discipline already in
   `services/peer.rs:148`, applied to the process tier).

Steps 1-3 are one coherent change (the type + its fill sites + the surfacing). Steps 4 and 5 are the
crash-lifetime and the poll'-reply changes. Each lands with its own RED gate; all share the one law.

## The RED gate (acceptance — a NEW probe, diagnostic-scoped)

A forked-process service receives a client message it cannot decode. Assert BOTH:
- **the caller's error carries the real reason** — contains `unknown tag` / `decode failed` /
  `no matching struct or enum`, NOT the bare `peer closed / channel disconnected`; and
- **the service is still alive** — a subsequent request to the same service succeeds.

At HEAD both fail (mute "peer closed" + dead service). GREEN when steps 1-5 land. This is independent of
the `LogMessage`-opaque question (deliberately deferred) — it tests *diagnosis*, not the log-message shape.

**Content-integrity / no-regression:** whole floor back to exactly the standing `no_inlined_wat` lint,
zero new failures; a genuine handler panic in a process service still delivers its reason to the creator
(a second RED probe: a service whose handler raises → the `Handle` surfaces the reason, no `EPIPE`).

## The lesson this plants (for the next self, across the gap)

Failure-masking is one class, not N bugs. A recv/decode/io error that cannot carry its reason, a
`map_err(|_|)` that binds the error to nothing, a crash reason written to a closed pipe, a service that
dies over one client's bad message — all the same disease. The wall: **the error type carries the
reason, so `Disconnected` can mean only a clean goodbye, and a mute failure cannot be constructed.**
`RVINA ERVDIT` — the ruin must educate; a failure that cannot speak teaches nothing and induces exactly
the "guaranteed confusion and flailing" this arc forbids.
