# DESIGN-STONE — the owner signals the child it spawned

> **Status: FULLY RULED 2026-08-03 — READY TO BUILD, resuming at P1.** Option A on `Process`; the
> vocabulary intueri-cast and weighed; SIGINT/SIGTERM independent; `Kill` IN as a variant. **No open
> questions remain.** Builder: *"it has been reasoned - get our docs in order and build it."*
> Supersedes task **#65**'s
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
| the owner holds the child | `spawn-program` returns `:wat::kernel::Process<I,O>`; `Process` **derives** `Peer` (`wat/spawn.wat:238`) | ✅ built |
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
   *"runs owner-side after the peer is spawned, **before `spawn-program` returns**, for effects."*
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

## ★★ THE REFINEMENT — `Process`, NOT `Peer`. This is the load-bearing part.

`Peer` is the parent of both `Thread` and `Process` — grounded on the **code**, `wat/spawn.wat:245-246`:

```clojure
(:wat::core::derive :wat::kernel::Thread  :wat::kernel::Peer)
(:wat::core::derive :wat::kernel::Process :wat::kernel::Peer)
```

**A thread peer has no process to signal**, and the user-signal flags are *process*-global, so
signalling one would be incoherent even where pthread-level delivery exists.

> **⛔ A CORRECTION, KEPT VISIBLE.** The first draft of this stone cited `wat/spawn.wat:238` —
> *"arc 291 3a-ii-β: Thread'/Process' ARE Peer's"* — as the anchor for this ruling. **That line is a
> COMMENT, and it is STALE**: arc 278 `"0z"` (`70fe856d`, 2026-08-02) stripped the primes from every
> IPC/kernel name, types included (`wat-scripts/fixes/reclaim-ipc-prime-names.wat:64-67` maps
> `Peer'`→`Peer`, `Thread'`→`Thread`, `Process'`→`Process`). The whole first draft was written in the
> retired spelling. The **ruling** survived the correction; every **spelling** in it did not. Caught
> by the intueri cast, verified by own read (`":wat::kernel::send"` at `runtime.rs:5717`; zero primed
> hits in `runtime.rs`/`check.rs`). Cite the derives, not the prose about them.

- On `Peer` the verb is **PARTIAL** — a domain hole named *"the peer is a thread."*
- On `Process` the verb is **TOTAL**.

This is this arc's own ruling applied one layer out — the same reason `i64::>` beat generic `>` on
2026-08-03: **monomorphising does not merely narrow the type, it deletes the domain hole.** Per-type,
period. A `Peer`-typed signal verb would need an outcome variant for *"this peer cannot be
signalled,"* which is a fence hole dressed as an enum.

## The shape — RATIFIED 2026-08-03 (intueri cast + the builder's rulings)

### The verb — `:wat::kernel::signal`, NO PRIME

```clojure
(:wat::kernel::signal proc :wat::kernel::Signal::User1)   ;; proc <- :wat::kernel::Process<I,O>
```

**No prime.** Arc 278 `"0z"` (`70fe856d`) stripped the primes from the whole IPC/kernel surface the
day before this stone was drawn. A verb minted in the same arc that just finished removing the mark
does not reintroduce it. *(The first draft asserted the opposite from stale comments — see the
correction box in § THE REFINEMENT.)*

**`:wat::kernel::`, not `:wat::spawn::`.** Every value-acting peer verb lives in `kernel`
(`send`/`recv`/`close`/`poll`). `:wat::spawn::`'s `process/*` names are `ProcessOpts` **builders** —
a different grammatical class (methods on a config record, not verbs on a live handle).

**No `{:restricted-to …}`.** Holding the `Process` **is** the capability — you cannot signal what
you did not spawn. That is the ocap argument, and it is stronger than a namespace list: the wall is
the value, not a caller check. `spawn-program` is restricted because it *mints* the capability;
consuming one you already hold needs no second gate.

### `:wat::kernel::Signal` — a CLOSED SET, therefore an enum, and it has THREE TIERS

The closed-set rule (R27): *a closed set is an enum; the name holds the value.* A bare `i64` signal
number would be the string-key mistake with a different hat. Name verified free — zero hits for
`wat::kernel::Signal` across `src/` and `wat/`.

**Six variants, and they are not uniform in what they cause. The tiers are the point:**

| tier | variant | POSIX | who observes, and how |
|---|---|---|---|
| **flag** | `User1` | SIGUSR1 | the **child**, and it keeps running — `(sigusr1?)` reads true |
| **flag** | `User2` | SIGUSR2 | the **child**, and it keeps running — `(sigusr2?)` reads true |
| **flag** | `Hangup` | SIGHUP | the **child**, and it keeps running — `(sighup?)` reads true |
| **stop** | `Interrupt` | SIGINT | the **child**, and it chooses when to stop — `(stopped?)` reads true |
| **stop** | `Terminate` | SIGTERM | the **child**, and it chooses when to stop — `(stopped?)` reads true |
| **kill** | `Kill` | SIGKILL | the **OWNER** — the child observes nothing and stops mid-instruction |

**⚠ THIS TABLE IS THE ENUM'S DOC COMMENT. It is not commentary on the design; it is the only place
two load-bearing facts can honestly live**, because both are facts about the **set**, not about any
single name:

1. **`Interrupt` and `Terminate` share one landing.** Both reach `substrate_on_stop_signal`; the
   child cannot tell them apart. The intueri cast ruled explicitly that **the names cannot carry
   this** — no word for "interrupt" also means "and by the way this is indistinguishable from
   terminate" — and that smuggling it in (`InterruptStop`) would be a mumble restating a fact about
   the enum in every variant. So it lives here.
2. **`Kill` has no child-side observable at all.** SIGKILL is uncatchable (`handle.rs:136`: *"SIGKILL
   is unignorable"*) — a POSIX guarantee, not a substrate choice. The round trip still closes, on the
   **owner**: `CloseOutcome::Signaled[signal <- i64]` (`types.rs:1747`) is documented as *"process
   TERMINATED by a signal."*

**★ RULED 2026-08-03 — SIGINT and SIGTERM are named INDEPENDENTLY.** The builder: *"SIGINT and
SIGTERM can be named.. independently... but they flow into `stopped?`."* An earlier draft collapsed
them to one `Stop` variant, reading *"we treat them as equal in wat"* as an identity claim. **That
conflated the observation with the identity** — the shared landing is a HANDLER decision, not a fact
about the signals. And the collapse was independently dishonest: one `Stop` variant must pick a
signal to put on the wire, and the form then stops saying which one went out, while `strace`, `ps`,
and any non-wat observer still see the difference.

**★ RULED 2026-08-03 — `Kill` IS IN, as a variant of this enum.** Four-questioned twice:

*Is `Kill` in the surface at all?* **IN.** Omitting it fails **Honest**, because
**omission is not absence**: `handle.rs:17` — *"Drop now owns the only unconditional SIGKILL+reap
path"* — so the authority **already exists**, and leaving the variant out removes only its *name*. A
surface that silently holds an authority it refuses to name is dishonest by construction. The UX gap
is concrete: `Drop` is SIGKILL **then `wait_status()` with the result discarded** (`:138-139`), so
killing a wedged child today forfeits its exit status, and `wait_or_cached_exit` first does not help
— it blocks on the very child that is wedged. **You cannot kill-then-inspect.** `Kill` is not new
authority; it is existing authority made legible at the call site.

*A variant, or its own `kill` verb?* **A VARIANT.** A separate verb fails **Obvious** and **Simple**:
its boundary would be *"does the OS permit the process to handle it"* — a distinction wat's surface
draws nowhere else and no reader can derive without POSIX knowledge. **That is structurally the same
failure this arc already rejected** when the `where` stone threw out "syntax stays core-spelled"
because its boundary was *"wherever `classify_expr` happens to have a structural arm"* — an
implementation detail leaking into surface shape. Same shape, different layer. And the receive side
already groups every signal uniformly: `Signaled[i64]` reports SIGKILL exactly like the rest.

**Drop's invariant is untouched** — its claim is about the *unconditional* path, and `Kill` is
conditional by construction.

### `:wat::kernel::SignalOutcome` — a matchable VALUE, never a raise

The builder's LAW at the peer-lifecycle walls: *"for any options — four-questions — we deliver an
enum for code to handle exceptions with; raise is uncatchable on purpose, a thing that must never
happen."*

```clojure
:wat::kernel::SignalOutcome        ;; non-parametric — returns no live resource
  Delivered                         ;; the kernel accepted it for that process
  Gone                              ;; the child had already exited (ESRCH)   ⚠ SEE STOP-2
  Failed[cause <- Failure]          ;; io failure
```

**`Delivered`, not `Sent`.** `Sent` names the *owner's* action and is silent on arrival — which is
the entire reason this type exists. Reusing `SendOutcome::Sent` for a different guarantee is a
meaning transplant.

**`Gone`, not `Closed` or `Exited`.** `Closed` already means *"the channel is cleanly shut"* in three
sibling enums; reusing it for *"the whole process is gone"* collides two closure concepts under one
word. `Exited` implies clean termination, which ESRCH does not.

**⚠ `Gone` CARRIES NO FIELD, and STOP-2 now covers two things.** The cast caught that a
`cause <- Failure` on an arm whose only trigger is ESRCH is a field that never varies — one the
reader carries for nothing. So STOP-2 is sharpened: prove the arm is **reachable** *and* that a field
would **earn its place**, before minting either.

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
  spawn-program ──────────────────────▶ starts, handlers installed
  recv ◀──────────────── #probe/Ready {}          (child announces; owner now knows it is alive)
  process/signal proc :user1 ──────────▶ handler sets KERNEL_SIGUSR1
  send :observe ──────────────────────▶ (stdin)
  recv ◀── #probe/Observed {:user1 true :user2 false :hangup false}
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

- **⛔ STOP-1 — the verb goes on `Process`.** If the implementation path pushes toward `Peer`
  (shared codegen, a derive, a convenience), STOP and surface. A `Peer` verb is partial and this
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
- **⛔ STOP-7 — `Kill` does not get a second SIGKILL+reap path.** `Kill` sends and returns; it does
  NOT reap. `ChildHandle::Drop` remains the only unconditional SIGKILL+reap (`handle.rs:17`). A `Kill`
  that also reaps destroys the very thing the variant exists for — killing and then still inspecting.

## ⛔ A CONSTRAINT ON P3/P4, DISCOVERED AT P2 — there is NO wat door into `close`

Found while building row 5's fixture, and grounded by own read at `src/resolve/registration.rs`:

```text
Absent + namespaced + reserved + Privilege::User -> Reserved
```

A test fixture is `Privilege::User`, so it **cannot define anything under `:wat::`** — and `close`
is `#[restricted_to(":wat::kernel::")]`. The `:wat::kernel::`-namespaced helper that would bridge to
it is refused at *registration*, before `restricted_to` is ever consulted. **So no wat-level fixture
can reach `close` at all** — not awkwardly, not at all.

**Consequence, and P3 must plan around it rather than rediscover it:** the `Kill` case cannot be
asserted at the wat surface today. P2's row-5 fixture therefore asserts
`ExitStatus::Signaled(SIGKILL)` through `peer.wait()` in Rust — the exact mechanism `close` itself
calls, one layer beneath the unreachable verb. **The kill IS proven delivered and the child proven
terminated BY SIGKILL; the wat-level `CloseOutcome::Signaled` value is NOT proven constructed.**
That is an honest degradation, documented in the fixture's own header, and it is the one row of this
stone that did not ship as specified.

Closing it needs a sanctioned wat path to `close` — its own question, not this stone's, and not
something to route around with a privilege escape.

## Strike order

| stone | what | state |
|---|---|---|
| ~~**P0**~~ | ~~Cast **intueri** on the vocabulary~~ — **CAST + WEIGHED 2026-08-03.** Target at `wat-scripts/intueri/process-signal-vocabulary.wat.intueri`; verdict folded into § The shape. Its first finding was against the target itself (the stale prime premise), verified by own read and corrected. | ✅ done |
| **P1** | The RED probe — spawn + attempt signal, fails on exactly the missing head. Committed. | ▶ **startable now** |
| ~~**P2**~~ | ~~Mint~~ — **LANDED `ae662ba0`**, weighed by own `--release` re-run. Floor `4342/4342/0/262`, clippy clean, both discard doors refuse, `Gone` NOT minted (STOP-2 held — a zombie's pidfd returns `Ok(())`; ESRCH only after reaping, which only `close` does, and it consumes the peer). Row 4 **broken on purpose** (send `User2`, child checks `sigusr1?`) → RED, then restored byte-exact → green. Row 5 honestly degraded, see the constraint above. | ✅ done |
| **P3** | The three flag tests → wat deftests over the spawn tooling; the two cause-tests stop touching globals. **The deliberate break runs here.** | blocked by P2 |
| **P4** | The three `libc::raise(SIGTERM)` harness tests → the same mechanism. | blocked by P3 |

## Owed

- **A WHY comment bridging the send/receive asymmetry** — the cast ruled the enum-on-send /
  bare-`i64`-on-receive split **defensible** (closed domain vs open domain: we choose what is
  sendable; we do not control what kills you) **but currently undocumented in shipped code.** A
  reader touching both sides sees one concept spelled two ways with nothing bridging them. One line
  on `CloseOutcome::Signaled` or on `Signal` closes it. Land it with P2.
- **#65 retitled** to the self-signal capability question, unbuilt, with the misfiling recorded. ✅
- **A note against #48** — `ProcessLaunch/pid` is identity-only *by design*; it must not be read as
  unadopted capability by a future census.
- **FOLLOW-ON, out of this stone's scope, tracked not smuggled:** `check.rs`'s `MUST_USE` doc blocks
  are stale on the de-prime axis — they name `send'`/`recv'` for verbs that shed the prime in `"0z"`.
  A stale comment lies actively, and this stone was itself misled by exactly that class today.

## Open — the builder's

**Both prior questions are RULED; none remain.**

1. ~~SIGKILL in or out~~ → **IN**, as `Kill`, a variant. Four-questioned twice; see § The shape.
2. ~~Does the enum ship `Stop` before P4 exercises it~~ → moot. The question presumed one collapsed
   `Stop`; `Interrupt` and `Terminate` are now independent variants, and P4 exercises `Terminate`
   end-to-end via the cascade tests. `Interrupt` and `Kill` ship exercised by P3's own cases —
   `Kill`'s is the sharpest gate in the stone, because **no handler can fake it: there is no
   handler.** Kill the child; assert the owner reads `Signaled`.
