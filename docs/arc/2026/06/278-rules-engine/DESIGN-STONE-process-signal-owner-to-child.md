# DESIGN-STONE — the owner signals the child it spawned

> **Status: RULED 2026-08-03 by the builder — Option A, on `Process'`.** Supersedes task **#65**'s
> framing (*"a wat program cannot signal itself"*), which was the wrong subject; see **§ #65 was
> misfiled**. Downstream beneficiary: the five in-process signal tests and the three
> `libc::raise(SIGTERM)` harness tests, rebuilt as wat deftests over the spawn tooling.
>
> Lineage: the races are arc 170's (`063ab25f` deleted the per-test fork quarantine); the capability
> gap is 278's to close because 278 is where it surfaced and where #65 was filed.

## The thing being built

**One verb: an owner signals the child process it spawned.** Plus the outcome enum it returns and
the must-use gate that forbids dropping that outcome — the established peer-lifecycle wall shape.

The builder's framing, and it is the spine:

> *"if we are going to measure a process level setting — we need a dedicated process to observe
> this … we must not modify our runtime to measure a thing … the flags are purposefully process
> global states."*
>
> *"we can spawn a program via the spawn tooling — it runs an entire wat world and the rust side can
> interface with it directly … it knows the pid … we can issue our kills and measure the results …
> all of wat's world operates on stdio so we can probe via stdin writes and read out via stdout
> reads — stderr may only be written to on crash … all things are edn in, edn out … we called this
> 'typed unix' … and it is."*

## What already exists — grounded, this session, `file:line`

Everything in that description is on the disk **except the signal**. This is assembly, not invention
(R2), and the stone must not re-derive any of it:

| capability | where | state |
|---|---|---|
| spawn a whole wat world in a process | `wat/spawn.wat:311-322` → `spawn_process_peer` (`kernel/spawn.rs:802`) | ✅ built |
| the child runs real signal handlers | `bin/wat.rs:23` → `distribution::run` → `install_substrate_signal_handlers()` (`distribution/mod.rs:347`) → `SIGUSR1/2/HUP` at `process/child.rs:84-92` | ✅ built |
| the owner holds the child | `spawn-program'` returns `:wat::kernel::Process<I,O>`; `Process'` **derives** `Peer'` (`wat/spawn.wat:238`) | ✅ built |
| **the pidfd is in that peer** | `kernel/peer.rs:539` — `pub(crate) pidfd: crate::process::Pidfd` | ✅ built |
| reuse-safe signal delivery | `Pidfd::send_signal(sig: i32)` (`process/clone.rs:195`) — **already generic over the signal** | ✅ built, one caller |
| drive the child (stdin) | `peer.rs:313` `send` / `:328` `send_wire` | ✅ built |
| read the child (stdout) | `ProcessPeerBundle::recv` (`kernel/spawn.rs:344`) | ✅ built |
| crash channel (stderr) | the 3rd `comms::process` pair; `classify_peer_error` → `PeerDeath` | ✅ built |
| exit code discriminates | `ChildHandle::wait_or_cached_exit` (`process/handle.rs:98`) | ✅ built |
| **signal the child** | — | ⛔ **THE GAP** |

**`Pidfd::send_signal` is already generic over the signal and has exactly one caller** —
`handle.rs:138`, hardcoding `SIGKILL` on the Drop path. That is a caller limitation, not a
capability one. The kernel-side mechanism is finished; nothing new is being invented below the
surface.

## ★ THE FINDING — the pid is handed over and is inert

`wat/spawn.wat:41`:

```clojure
(:wat::core::defrecord :wat::spawn::ProcessLaunch [pid <- :wat::core::i64])
```

Delivered owner-side by `post-spawn-fn`. **Twelve call sites of `ProcessLaunch/pid`** across
`wat-scripts/probes/arc-278/`, `tests/services/`, and the sift-arena probe. **Every one carries it as
an identity** — `(Vector i64 pid)` for the `ps`-visible label (closure #6), or forwards it down a
channel to prove the accessor type-checks. **Zero act on it.** There is no kill or signal verb
anywhere in `wat/spawn.wat`.

**And this is the correct end state, not a gap to fill.** `ProcessLaunch/pid`'s job is *identity*,
not capability. It stays consumer-less by design. Recorded here so a future UNADOPTED census (#48)
does not read an identity value as orphaned capability and go hunting for a verb to hang on it.

## ★★ #65 WAS MISFILED

Task #65 reads *"a wat program cannot signal itself."* True, and **not the blocker.** The rider that
filed it reached for self-signal because today's tests raise **into the harness's own process**, so
self-raise looked like the shape. It is not the shape the builder asked for.

> **The real gap: a wat parent cannot signal the child it owns.**

Materially different, and far safer. The parent spawned the child, holds its lifeline, reaps it, and
is already handed its pid owner-side as a typed record. This verb is **the missing member of an
ownership relationship that already exists** — not a program raising arbitrary signals into itself
with no owner at all.

**#65 stays filed, unbuilt, retitled** to the self-signal capability question, with the note that it
was never what blocked the tests.

## The fork, and how it was ruled — the four questions, run

**A — the verb hangs off the process peer.** **B — the verb takes `ProcessLaunch{pid}`, runtime
resolves pid → owned pidfd.**

Two disk facts decided it before the questions:

1. **`ProcessLaunch{pid}` is a during-spawn callback value.** `wat/spawn.wat:307` — post-spawn-fn
   *"runs owner-side after the peer is spawned, **before `spawn-program'` returns**, for effects."*
   The caller does not hold the handle yet. Which is exactly why all twelve sites **smuggle the
   integer out** over a channel. B would make that smuggle load-bearing.
2. **The pidfd is in the peer** (`peer.rs:539`) — the owner already holds reuse-safe delivery.

| | **A — on the peer** | **B — on `ProcessLaunch{pid}`** |
|---|---|---|
| **Obvious?** | **YES** — signal the process this peer talks to. The peer *is* the live relationship. | **NO, and this kills it.** The record only exists inside a callback that runs before you hold the handle; every real use must smuggle the integer out and act on it later. The form reads *"signal this pid"*; the behaviour is *"consult a hidden table of children we own."* Unstatable from the form. |
| **Simple?** | **YES** — one noun, one verb, zero new state; the fd is already in the bundle. | **NO** — needs an owned-children registry keyed by pid, with lifetime and eviction rules. New mutable global state, in a ZERO-MUTEX substrate, to serve a verb with a stateless alternative. |
| **Honest?** | **YES** — cannot address a recycled pid, because it never uses a pid. Cannot outlive the relationship: the peer's Drop closes the fd. | **NO** — our own code carries the refutation: *"Do NOT use this PID for `kill(pid, sig)` … This PID may be reused"* (`clone.rs:215-216`). A verb whose parameter is a documented-unstable identity is a type that lies. |
| **Good UX?** | **YES** — available for the child's whole life, to exactly whoever owns the child. | not reached |

**RULED: A.** B fails three of three before UX is weighed.

## ★★ THE REFINEMENT — `Process'`, NOT `Peer'`. This is the load-bearing part.

`Peer'` is the parent of both `Thread'` and `Process'` (`wat/spawn.wat:238` — *"Thread'/Process' ARE
Peer's"*). **A thread peer has no process to signal**, and the user-signal flags are *process*-global,
so signalling one would be incoherent even where pthread-level delivery exists.

- On `Peer'` the verb is **PARTIAL** — a domain hole named *"the peer is a thread."*
- On `Process'` the verb is **TOTAL**.

This is this arc's own ruling applied one layer out — the same reason `i64::>` beat generic `>` on
2026-08-03: **monomorphising does not merely narrow the type, it deletes the domain hole.** Per-type,
period. A `Peer'`-typed signal verb would need an outcome variant for *"this peer cannot be
signalled,"* which is a fence hole dressed as an enum.

## The shape

### The verb

Provisional, **naming is intueri-owed** (see § Owed):

```clojure
(:wat::spawn::process/signal proc :user1)   ;; proc <- :wat::kernel::Process<I,O>
```

**No `{:restricted-to …}`.** Holding the `Process'` **is** the capability — you cannot signal what
you did not spawn. That is the ocap argument (Miller, R15's constellation), and it is stronger than a
namespace list: the wall is the value, not a caller check. `spawn-program'` is restricted because it
*mints* the capability; consuming a capability you already hold needs no second gate.

### The signal argument is a CLOSED SET, therefore an enum

The closed-set rule, from the telemetry design (R27): *a closed set is an enum; the name holds the
value.* A bare `i64` signal number would be the string-key mistake with a different hat.

**And the variants should name what the SUBSTRATE'S OWN HANDLERS DO, not the POSIX numbers.** A wat
program's model is `(:wat::kernel::sigusr1?)` and `(:wat::kernel::stopped?)` — it has never once
thought in signal integers. Provisional, intueri-owed:

| variant | POSIX | what the child's installed handler does |
|---|---|---|
| `User1` | SIGUSR1 | sets `KERNEL_SIGUSR1` → `(sigusr1?)` reads true |
| `User2` | SIGUSR2 | sets `KERNEL_SIGUSR2` → `(sigusr2?)` reads true |
| `Hangup` | SIGHUP | sets `KERNEL_SIGHUP` → `(sighup?)` reads true |
| `Stop` | SIGTERM | `substrate_on_stop_signal` → `(stopped?)` reads true |

**⛔ SIGKILL is a builder question, deliberately NOT decided here.** `ChildHandle`'s Drop already
sends it (`handle.rs:138`), so the mechanism is present and the capability is *de facto* the
substrate's. Exposing it to wat means an owner can hard-kill a child that cannot refuse — which may
be exactly right (it owns the child) or may belong only to teardown. **Not needed by any consumer in
this stone.** Left out of the initial mint; raised, not smuggled.

**SIGINT is also omitted** — it routes to the same `substrate_on_stop_signal` as SIGTERM, so `Stop`
already covers the observable behaviour. A second spelling of one outcome is the `()`/`nil` shape arc
179 just finished killing.

### The outcome is a matchable VALUE, never a raise

The builder's LAW, recorded at the peer-lifecycle walls: *"for any options — four-questions — we
deliver an enum for code to handle exceptions with; raise is uncatchable on purpose, a thing that
must never happen."*

Provisional shape, and **the middle arm is conditional on a reachability proof** (see STOP-2):

```clojure
:wat::kernel::SignalOutcome        ;; non-parametric — returns no live resource
  Delivered                         ;; the signal reached the child
  Gone[cause <- Failure]            ;; the child already exited (ESRCH) — a REAL race the owner must face
  Failed[cause <- Failure]          ;; io failure
```

**Must-never-happen stays a raise:** `EINVAL` (bad signal — unrepresentable, the enum forbids it),
`EBADF` (closed pidfd — a substrate bug). Those are arity/type-class faults, not handleable
conditions.

**Non-parametric ⇒ `MUST_USE_TYPES`, not `MUST_USE_PARAMETRIC_HEADS`** (`check.rs:7020-7024`) —
the same slot as `CloseOutcome`. A *faced* signal (matched over the variants) has an arm-joined type,
never `SignalOutcome`, so the gate fires only on a raw dropped call, closing both discard doors
(`do`-non-final and `let`-`_`).

## The consumers — the tests, rebuilt

The whole point. **The runtime is not touched; the flags stay exactly as process-global as they are
meant to be; the observation happens in a process dedicated to being observed.**

| test | asserts | disposition |
|---|---|---|
| `sigusr1_query_reflects_flag_state` | flag state | → the dedicated process |
| `sigusr2_and_sighup_independent` | flag state | → the dedicated process |
| `reset_sigusr1_flips_flag_false` | flag state | → the dedicated process |
| `reset_sighup_returns_unit` | `Value::Unit` | **race CAUSE, not victim** — drop the gratuitous global mutation; stays in-process |
| `user_signal_predicates_refuse_arguments` | `ArityMismatch` | **race CAUSE, not victim** — drop the gratuitous `reset_user_signals()`; stays in-process |

Only three of the five ever asserted flag state. The other two mutate the shared statics and clobber
their siblings while being structurally unable to suffer the race themselves. They do not need a
process; they need to stop touching globals.

**The same mechanism covers the three `libc::raise(libc::SIGTERM)` harness tests** —
`tests/channel/probe_arc170_writer_joins_lockstep.rs:135`,
`tests/process/shutdown_cascade_memory.rs:122`, `tests/process/shutdown_cascade_pipefd.rs:129` —
each of which raises into the harness's own process and whose doc comments say the safety rests on
*"nextest already gives every test its own process."* Signal a real child; read the cascade off the
peer.

### The protocol — a wire handshake, never a sleep

`mora`: *sleep is a guess; guesses race.* The handshake is EDN over the peer, every step an event:

```
owner                                   child (a real wat world)
  spawn-program' ──────────────────────▶ starts, handlers installed
  recv' ◀──────────────── #probe/Ready {}          (child announces; owner now knows it is alive)
  process/signal proc :user1 ──────────▶ handler sets KERNEL_SIGUSR1
  send' :observe ──────────────────────▶ (stdin)
  recv' ◀── #probe/Observed {:user1 true :user2 false :hangup false}
  wait  ◀──────────────── exit 0
```

The `Ready` line is what makes the signal safe to send: the child is **provably alive and past
handler install** at signal time. The `:observe` request is what makes it a measurement rather than a
poll — the child answers when asked, so there is no spin loop and no timeout to tune.

Assert the returned EDN **structure exactly** — `wat` stdio is EDN, and a `contains?` on a rendered
string is the launder this project has a rule against.

## The RED probe — before the brief, per `examinare`

A ten-line probe that fails on **exactly** the gap: spawn a child, attempt to signal it, and confirm
there is no verb. Expected today: an `UnknownCallee` / no-matching-clause naming the missing head,
with everything around it clean — the spawn, the peer, the recv all working. Commit it; the brief
cites it as the worked reference.

If the probe cannot be made to isolate the gap, the foundation is not ready and the brief does not
get written.

## ★ THE DELIBERATE BREAK — the gate must be proven able to go RED

**R59 `NISI FRANGAS, NIHIL PROBAS`, and it applies to this stone with force**, because the exact
failure being repaired is *a signal test that passed while no signal was ever delivered.* Shipping a
green test here without breaking it first would re-commit the sin under a new mechanism.

**The break: comment out `install_substrate_signal_handlers()` and confirm the gate goes RED naming
the signal.** If the test still passes with the handler gone, it is measuring the harness, not the
substrate, and the strike has failed regardless of what the Summary line says.

Name the red condition out loud before crediting the green: *this goes red if the child's SIGUSR1
handler does not run.* That is the mechanism; nothing else is being claimed.

## STOPs — rejection criteria, ship nothing and surface

- **⛔ STOP-1 — the verb goes on `Process'`.** If the implementation path pushes toward `Peer'`
  (shared codegen, a derive, a convenience), STOP and surface. A `Peer'` verb is partial and this
  stone exists partly to refuse it.
- **⛔ STOP-2 — prove `Gone` is reachable before minting it.** Write a probe that signals a child
  which has exited but not been reaped, and observe `ESRCH`. If it is not reachable through the
  pidfd, **do not mint the arm** — an unreachable arm accumulates lies because nothing ever
  contradicts them. Two arms and a raise is a fine answer.
- **⛔ STOP-3 — never `kill(pid, sig)`.** Delivery routes through `Pidfd::send_signal` or the strike
  is wrong. Our own `clone.rs:215-216` documents why. A `libc::kill` anywhere in this diff is a
  rejected strike.
- **⛔ STOP-4 — do not modify the runtime to make the measurement work.** The builder's ruling. If
  the tests appear to need a runtime change, the test design is wrong; STOP and surface it. Adding a
  public `send_signal` to `ChildHandle` for the harness's benefit is the specific move already
  withdrawn once this session.
- **⛔ STOP-5 — the flags stay process-global.** They are that way on purpose. A per-thread or
  per-runtime refactor to make testing easier is a rejected strike.
- **⛔ STOP-6 — no `_` wildcard on the outcome enum.** Doctrine whose checker rule is unbuilt, so
  nothing will stop a rider taking it. Taking it is a rejected strike.
- **⛔ Do not fold in SIGKILL.** Raised above; the builder's call; no consumer needs it here.

## Strike order

| stone | what | state |
|---|---|---|
| **P0** | Cast **intueri** on the vocabulary: the verb, `SignalOutcome` + its variants, the signal enum + its variants. Materialize the whole set as a `.wat` artifact and spawn the ward. | ▶ first, blocks the mint |
| **P1** | The RED probe — spawn + attempt signal, fails on exactly the missing head. Committed. | ▶ startable now, independent of P0 |
| **P2** | **Mint**: the signal enum, `SignalOutcome` in `types.rs`, the `Process'` verb over `Pidfd::send_signal`, the `MUST_USE_TYPES` entry. STOP-2 gates the `Gone` arm. | blocked by P0, P1 |
| **P3** | The three flag tests → wat deftests over the spawn tooling; the two cause-tests stop touching globals. **The deliberate break runs here.** | blocked by P2 |
| **P4** | The three `libc::raise(SIGTERM)` harness tests → the same mechanism. | blocked by P3 |

P0 and P1 are mutually independent and start together.

## Owed

- **intueri on the whole vocabulary** (P0) — no name in this document is ratified. The verb spelling,
  `SignalOutcome`, the variant names, and the signal-enum variants are all provisional. Precedent
  check found **no existing verb on a `Process'`/`Peer'` value in `wat/*.wat`**, so there is no house
  style to inherit; the cast is doing real work, not rubber-stamping.
- **The builder's SIGKILL call** — in or out.
- **#65 retitled** to the self-signal capability question, unbuilt, with the misfiling recorded.
- **A note against #48** — `ProcessLaunch/pid` is identity-only *by design*; it must not be read as
  unadopted capability by a future census.

## Open — the builder's

1. **SIGKILL:** does an owner get to hard-kill a child from wat, or does that stay teardown-only?
2. **Does the child need a `Stop` consumer test at all**, or do the three cascade tests (P4) already
   cover `Stop` end-to-end? If they do, the initial signal enum could ship `User1`/`User2`/`Hangup`
   only, and `Stop` arrives with P4 — smaller first mint, one fewer unexercised variant.
