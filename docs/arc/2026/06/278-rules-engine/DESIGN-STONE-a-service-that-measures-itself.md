# DESIGN-STONE — a service that measures itself (the first signal consumer)

> **Status: RULED 2026-08-03 — UNBLOCKED, ready to build.** The fork below is closed: option (ii),
> `Handle/process → Option<Process>`. And the admin-ask was **cut** — the app queries as it goes.
> Supersedes the P4 framing in `DESIGN-STONE-process-signal-owner-to-child.md` (the three
> `libc::raise` tests). Builder's origin: *"should we just build a real mini app? … the service's
> purpose is to be able to do measurements as it does its ops."*

## The governing rulings (builder, this session — do not re-litigate)

1. **wat must never deadlock.**
2. **A service delivers its state to the admin handle WHEN THE ADMIN ASKS.** Request/reply, never
   push. A service that pushes unsolicited state blocks on a pipe nobody drains — and blocking during
   teardown is the deadlock forbidden above.
3. **If the process is killed before the admin asks, there is nothing to ask.** Not lost data — an
   empty set. Silent drop is correct.
4. **Anything that measures how a process behaves is measured HERMETICALLY. Never modify the runtime
   to take a measurement.** *"the signal handlers were clear violators to this and caused
   unpredictable tests."*
5. Non-stop signals are a **bitflip eventually observed by an interested app**; **stop must be dealt
   with.**

**What ruling 2+3 close, recorded so it is not reopened:** `ServiceEvent::Shutdown → nil`
(`service.wat:1227`) is **correct, not accidentally correct** — it is the only non-blocking thing a
killed service can do. There is **no race** between main's orderly ask and the child's self-terminate;
both orderings are correct outcomes. **No new `ServiceEvent` variant** is warranted: the child's
correct behaviour is identical whether it was signalled or orphaned. Verified this session — both
`Admin::Stop` and `Shutdown` face every `SendOutcome` and return `nil`; **neither can block.**

## ★ THE FINDING THAT JUSTIFIES THE STONE

**No wat service, anywhere in the substrate, has ever observed a signal.** Census, this session:

| predicate | `wat/` | `wat-tests/` | `wat-scripts/` |
|---|---|---|---|
| `stopped?` | **1 — and it is a COMMENT** (`kernel/services/stdio.wat:95`) | 0 | 2 |
| `sigusr1?` / `sigusr2?` / `sighup?` | **0** | 3 (P3's new deftests) | 3 |

The handlers set the flags. The predicates read them. **The actor layer has never asked.** This app is
the first consumer — `ALIVS ARGVIT`.

## What the app is — the builder's sequence, verbatim

```clojure
(signal proc :sighup)     ;; → SignalOutcome::Delivered
(query  client :sighup)   ;; → true
(signal proc :user1)
(query  client :user1)    ;; → true
(signal proc :user2)
(query  client :user2)    ;; → true
(signal proc :terminate)  ;; → child dies, no notification
;; the admin handle wakes on the lifeline
```

One `defservice` that **measures as it serves**: a durable record with a field per observation,
updated when the serve loop finds a flag set, **queried through the ordinary client ops as the test
progresses.**

```clojure
;; durable — a field per observation. A record, because records ROUND-TRIP TAGGED (proven below).
(:wat::core::defrecord …::Obs
  [requests <- :i64  sighup <- :bool  user1 <- :bool  user2 <- :bool])
```

**★ THE ADMIN ASK IS CUT.** An earlier draft had the final record delivered via `<svc>/stop h`
(ruling 2's shape). The builder: *"why does admin stop even need to get invoked? we can just have the
service measure if the bit is flipped as we progress."* Correct, and it proves **more**:

- querying as you go demonstrates observation **during real operation**, which a final tally cannot
- it is how a real app reads its own flags — nobody stops a service to find out whether it got a HUP
- `stop` then needs no state path at all, which is exactly the silent-drop ruling already made

**One run proves all four handlers**, plus discrimination between them, plus clean death.

## Grounded mechanisms — all of it exists

| piece | where | state |
|---|---|---|
| the serve loop TCOs into itself | `service.wat:1095` `(~serve-name self l selectables new-state)` | ✅ |
| **on stop it does NOT recur** | `service.wat:1227` `(ServiceEvent::Shutdown nil)` | ✅ |
| a handler can end the loop AND reply | `Outcome::Stop final-state resp` | ✅ |
| the admin ask returns final state | `Admin::Stop` → `send self (Status::Stopped (stop-project state))` | ✅ |
| **records round-trip the wire TAGGED** | proven by own probe: `#probe/Obs {:user1 7 :user2 9}` | ✅ |
| start/connect/op/stop exemplar | `wat-tests/service-admin-facet.wat` — thread AND process, one token apart | ✅ |

**⚠ `Tuple` does NOT round-trip** (P3's finding — the untyped decode rebuilds it as `Vec`). Use a
record or a typed `Vector`.

## The delivery asymmetry the app demonstrates

Grounded in the two handlers (`process/child.rs`, `runtime.rs`):

```
substrate_on_stop_signal → request_kernel_stop() → sets KERNEL_STOPPED
                                                 → libc::write(WAKE_FD)   ← WAKES a blocked poll
substrate_on_sigusr1     → set_kernel_sigusr1()  → sets the flag.          ← that is ALL
```

So, exactly per ruling 5: **user signals are a bitflip observed on the next op; stop wakes the blocked
`poll` and must be dealt with.** No test in the substrate demonstrates both models. This one does, in
one program.

## ✅ RESOLVED 2026-08-03 — the blocker, and the one-line fix

**RULED: option (ii).** Mint `Handle/process → (Option Process)` — `Some` on a process locus, `None`
on a thread. Then the sequence above works with no safety compromise, `signal` stays `Process`-typed,
and the locus difference becomes *statable* instead of erased.

**This is not new capability — it is un-erasing what `start` already has.** `spawn-program` returns
`Process<I,O>`; `start` wraps it into `Launched.handle` typed as `Peer`. The concrete locus is thrown
away at that boundary. The `Option` says the true thing the type already knows.

**The pid route was considered and declined.** `start`'s locus param takes `ProcessOpts`, so
`(:wat::spawn::process/post-spawn f)` does hand the owner a `ProcessLaunch{pid}` — the sequence could
be driven by pid today. But `signal` deliberately does not take one: `clone.rs:215-216` documents a pid
as reuse-unsafe for `kill()`, and the four-questions killed the bare-pid verb on **Honest**. In this
test the pid happens to be safe (child alive, unreaped) — but that is *safe-because-the-caller-is-
careful*, which is precisely what the `Process` typing exists to remove. The accessor costs one
function and keeps the guarantee structural.

### The original blocker, kept for the record — a started service COULD NOT be signalled

`wat/spawn.wat:265` — the Handle carries a **`Peer`**, not a `Process`:

```clojure
(:wat::core::defstruct :wat::spawn::Launched<S,R,Sh,Lu>
  [handle  <- :wat::kernel::Peer<Sh,Lu>
   address <- :wat::kernel::Address<S,R>])
```

That erasure is **deliberate and correct** — it is what makes `stop` locus-agnostic (`service.wat:260`:
*"locus-agnostic launch, locus-agnostic start"*), so the same test runs on thread and process.

But `:wat::kernel::signal` takes **`Process<I,O>`**, ruled per-type in
`DESIGN-STONE-process-signal-owner-to-child.md` because a thread peer has no process to signal and a
`Peer`-typed verb would be PARTIAL. `Peer` → `Process` is a **downward** narrowing; the checker
refuses it, correctly — on a thread locus that field really does hold a `Thread`.

**So there is no path today from `start` to a signalable `Process`.** This is not a defect in either
ruling; it is two correct rulings meeting. The app cannot be built until it is resolved.

### The fork — the builder's

- **(i) The app does not use `start`.** Spawn the service program via `spawn-peer` (which returns
  `Process`), signal that, and drive it over the peer. **Cost:** it is then not a *started defservice*
  with a Handle — it loses the admin ask, which is ruling 2's whole subject.
- **(ii) A locus-specific accessor.** Something like `Handle/process → Option<Process>` — `Some` on a
  process locus, `None` on a thread. **Cost:** re-introduces locus-awareness to a surface built to be
  locus-blind; but it is honest (an `Option` states the truth: a thread has no process) and mirrors
  `ProcessLaunch`'s existing owner-side pid.
- **(iii) The app is process-locus only** and obtains the `Process` some other way at spawn time.
  **Needs grounding** — I have not found one, and I will not assert it exists.

**I lean (ii)** — the `Option` makes the locus difference *statable* rather than erased, which is what
the type already knows and the surface currently hides. But it widens a deliberately narrow surface,
so it is a ruling, not a default.

## The measurement is hermetic BY CONSTRUCTION — ruling 4

The app **is** the hermetic instrument. Everything observed happens in a spawned child that exists to
be observed and is thrown away. **Nothing modifies the runtime**: no handler is patched, no flag is
reset from the harness, no global is touched. That is the whole difference from the five tests P3
retired and the three `libc::raise` tests below.

## Second phase — the `libc::raise` tests, ruled

> *"anything that measures how a process behaves is done hermetically — never modify the runtime …
> if these libc::raise things are not providing meaningful coverage they are annihilated … if they are
> proving something, should that proof be in a hermetic test?"*

Three sites, each raising SIGTERM **into the harness's own process** —
`tests/process/shutdown_cascade_memory.rs:122`, `…pipefd.rs:129`,
`tests/channel/probe_arc170_writer_joins_lockstep.rs:135`. **All three violate ruling 4 by
construction**, and two of them leave `KERNEL_STOPPED` flipped with no reset — under `cargo test`
every later test in that process runs inside a shutdown that never happened.

**The disposition is already determined by the ruling; only the sorting is open.** For each:

1. **Census first** — what, other than this file, proves its specific claim? (The two cascade tests
   turn on a blocked `comms::thread` `typed_recv` woken via the crossbeam `select!` arm.
   `wat_cli__sigterm_blocked_on_stdin` proves the blocked-wake contract **out-of-process, with no
   timing assertion at all** — but through a *different* blocking primitive.)
2. **Covered ⇒ ANNIHILATE.** No relocation, no ceremony.
3. **Not covered ⇒ rebuild hermetically** over this stone's mechanism.

**⚠ And the `< 100ms` goes either way.** `shutdown_cascade_memory.rs:139` / `pipefd.rs:146` assert
`elapsed.as_millis() < 100`. Nothing derives 100; the doc just declares it. It asserts a *performance*
property while the test's subject is *correctness*, it is load-sensitive (R60 measured this box at load
8.47), and the "did it hang" question it is really asking **is already answered by nextest's 30s kill.**
The honest signal is the event — the join returned. **Delete the bound; do not re-derive it.**

`probe_arc170_writer_joins_lockstep` is a **separate** case: SIGTERM there is a *means* to test writer-join
lockstep, not the subject. Sort it on its own.

## STOPs — rejection criteria

- **⛔ Never modify the runtime to measure.** Ruling 4. No patched handler, no harness-side flag reset,
  no global touched. If a measurement seems to need it, the design is wrong — STOP.
- **⛔ Never push state.** The service replies when asked. An unsolicited send is the deadlock.
- **⛔ No sleep.** The wire is the synchronisation; the child answers when asked. `mora`.
- **⛔ No `Tuple` across the wire** — it degrades to `Vec`. Record or typed `Vector`.
- **⛔ No `_`-prefixed discard bindings** — the must-use gate is an exact `_` match
  (`check.rs:10926`), so `_x` slips it silently. This already cost one inert probe today.
- **⛔ Do not "fix" the `Shutdown → nil` arm.** Ruled correct above.
- **⛔ Do not delete a `libc::raise` test before its census.** Covered-elsewhere must be shown, not
  assumed.

## Strike order

| | what | state |
|---|---|---|
| ~~**A0**~~ | ~~the (i)/(ii)/(iii) fork~~ — **RULED: (ii), `Handle/process → (Option Process)`.** Admin-ask cut. | ✅ done |
| **A1** | Mint `Handle/process → (Option Process)` in the defservice Handle. One accessor, un-erasing the concrete locus `start` already holds. | ▶ **startable now** |
| **A2** | Build the app + its deftest — the builder's sequence verbatim. **The deliberate break:** remove `install_substrate_signal_handlers` at **`spawned_runtime.rs:51`** — the CHILD's install. **NOT `distribution/mod.rs:347`**, which is inside `run_with_args`, a path nextest never executes; that error cost a P3 rider a cycle. | after A1 |
| **B1** | Census the three `libc::raise` tests against § Second phase. | ▶ startable now, independent |
| **B2** | Annihilate or rebuild hermetically per B1; delete the `<100ms` bound either way. | after B1 |

## Open — the builder's

**The fork is ruled; nothing blocks.** One optional question remains:

- Does the app also run on a **thread** locus, asserting identical behaviour for everything except the
  signal? The exemplar (`service-admin-facet.wat`) runs both loci one token apart, so it is cheap —
  and it would make `Handle/process → None` visible in a test rather than only in the type. Worth it,
  but not required for the app to prove what it exists to prove.
