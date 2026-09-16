# NOTE (arc 109 substrate) — the reactor is used as a DISPOSABLE POLLER; one io_uring per endpoint, and one per DEADLINE

**Filed 2026-09-16, builder-directed** (*"the reactor concern.... make an arc 109 NOTE about it..."*),
from a three-day CI red whose mechanism is still unknown. **NOT STARTED, and deliberately so** — the
builder's ruling, verbatim:

> *"we are not undoing anything.. we will make the reactor superior.... but... that doesn't feel like a
> now thing... but... a mid-term thing... i do not wish to operate on the reactor loop extensively here"*

⛔ **So this note is evidence, not a plan.** It exists so whoever opens that arc starts from measurements
instead of re-deriving them, and so the three dead hypotheses below are never re-walked.

## ⭐⭐ SOLVED 2026-09-16 — the bill is RLIMIT_MEMLOCK, in BYTES, accounted PER-UID

Asked the runner directly (`src/bin/ring-ceiling.rs`, dispatched by
`.github/workflows/ring-ceiling.yml`) instead of inferring from a floor:

```
one process alone            CEILING: refused at ring #1025  → 1024 held
four concurrent processes    332 + 254 + 254 + 184           = 1024   ← one budget, split to the last ring
memlock 8 MB ÷ ~8 KB/ring    = 1024                                     ← exact
at refusal: CommitLimit 11.3 GB · Committed_AS 2.08 GB · MemAvailable 15 GB · VmSize 12.5 MB
```

⭑ **It is a BYTE budget, not a ring quota and not a descriptor limit** (`nofile` was 65536 and
irrelevant). A ring costs ~8 KB of locked memory; 1024 is arithmetic. A bigger ring costs more —
which is why **one shared ring with many entries is CHEAPER per waiter, not dearer**, and why this
finding argues *for* the direction below rather than against io_uring.

⛔ **And it is PER-UID: shared across every process that user runs.** The four-way split is the
proof. So the floor's four concurrent nextest processes, each spawning `wat` services that mint a
ring per `after`, drain one budget between them — which is why the failing SET moved between runs
and why one process reported `created-so-far=14` while another reported 244.

### ⚠ THE DEV BOX AND THE RUNNER DISAGREE, AND THAT COST FOUR DAYS

Debian **6.12.63** does **not** charge rings to memlock — 3000 held with `ulimit -l 0`.
ubuntu-24.04 / **6.17-azure** does. **Same 8 MB limit on both boxes** — the difference is the
kernel's accounting, not the limit.

★ So hypothesis #1 was **RIGHT on the runner the whole time**, and the local refutation was sound
*for the local kernel* and was over-generalised to a kernel nobody had measured. *"Measured
locally"* was true and did not travel. ⛔ **The number must come from the box that refuses.** That
is the entire justification for `ring-ceiling` existing, and for it being runnable anywhere in
seconds rather than costing a ~15-minute floor per question.

⚠ **This is NOT a CI artifact.** 8 MB is the stock `RLIMIT_MEMLOCK` systemd sets on ordinary Linux
hosts, and the dev box carries the identical limit. A production host on a kernel that charges
rings hits the same wall at ~1024 concurrent rings per UID. The dev box is the outlier that ages
out, not the runner. Run `ring-ceiling` on a target host before trusting it.

## ⭐ WHAT "CORRECT" LOOKS LIKE — one ring per I/O-EXECUTING THREAD, share-nothing

The builder's question was whether one ring per *process* is the target. It is not, and the reason
is structural: the SQ and CQ are **single-producer/single-consumer** ring buffers shared with the
kernel. That is where io_uring's speed comes from. Submitting from several threads needs a mutex —
reintroducing the contention the design exists to remove — and because completions land in the ring
that submitted them, a shared ring needs a broker to route each completion to whoever was waiting,
i.e. a cross-thread wakeup per operation.

**One ring per process is correct only when the process has ONE I/O thread.** To have it with many
threads you need a dedicated poller thread owning the ring, with waiters parked and woken by
`user_data` — a real event loop, and a much larger change than per-thread rings.

The pattern the systems that lean hardest on io_uring converge on (thread-per-core, share-nothing):
**ScyllaDB/Seastar** (ring per shard), **glommio** (ring per executor, futures deliberately
`!Send`), **tokio-uring** (ring per worker), **Netty's io_uring transport** (ring per event loop),
**libuv** (per loop). ⚠ Attribution: this is the orchestrator's knowledge, not a citation check —
verify against current docs before building on the finer points.

Their refinements are what "correct" means at scale, and each one is absent here:

| refinement | what it buys | our state |
|---|---|---|
| `IORING_SETUP_ATTACH_WQ` | many rings, ONE shared kernel worker pool — the idiomatic answer to "too many rings" | unused |
| registered files / buffers | resolve fds and pin buffers once, not per op | unused |
| SQPOLL (+ `SQ_AFF`) | kernel-side submission polling, approaching zero syscalls | unused |
| batching | many SQEs per `submit`, completions harvested in bunches | **`submit_and_wait(1)` everywhere** |

⭑⭑ **So the deeper mismatch is not the ring COUNT — it is that we would collect none of io_uring's
benefit even if rings were free.** The count is the visible bill; the missing amortization is the
actual inversion. The module header's principle — *"FDs are the persistent state; io_urings are
[ephemeral]"* — is backwards from the mechanism's design, where the **ring** is the persistent
object and the **operations** are ephemeral. Every symptom in this note follows from that one
inverted lifetime.

## Kin, and why this belongs in 109

109 is where the substrate's *shapes* are argued — `NOTE-io-boundary-outcome-enum.md` (every failing IO
boundary returns a matchable outcome) and `NOTE-an-outcome-variant-no-primitive-can-construct.md` (a
variant no primitive can construct is a **painted brick**). This note is the same genus one layer down:
**not the shape of the outcome, but the cost and failure surface of the machinery that produces it.**

It also has a direct tie to that painted-brick note. `:wat::kernel::after` can fail — the kernel can
refuse a ring — and it reports that by **raising `RuntimeErrorKind::MalformedForm`**
(`src/runtime.rs:27821`), which **no arm can face**. That is the *third* unfaceable failure found in
excursus 001 (`a-send-cannot-say-it-is-blocked`: a blocked `send` has no `SendOutcome` variant;
`a-wait-that-should-be-bounded` §6: `recv-all`'s timeout has no form in `Result :- [(Vector O),
LociDiedError]`). ⭑ All three are **missing forms**, not painted bricks — the doctrine covers variants
that exist and cannot be constructed, and is silent on failures that have no variant at all.

## The measurement — what the rings are actually used for

```
opcodes submitted, whole tree:   9 × PollAdd   ·   2 × Read   ·   1 × Write   ·   1 × AsyncCancel
every ring:                      IoUring::new(4)          (4 entries)
every wait:                      submit_and_wait(1)       (submit one, wait for one)
advanced features in use:        none — no SQPOLL, no register_files, no register_buffers, no fixed I/O
```

⭐ **That is `poll(2)`.** `PollAdd` + `submit_and_wait(1)` over a handful of fds, with no batching and no
kernel-side polling, is the semantics of a blocking `poll`, obtained by creating a kernel object with
mmaps and a setup/teardown syscall pair. The module header even describes the use case in poll's own
terms — *"fan-in over N receivers (generalizes Stone B's 2-arm POLL_ADD to N)"* — which is one `poll(2)`
call over an array of `pollfd`.

### Where the cost comes from

| fact | site |
|---|---|
| **every `Receiver` owns a ring** (`ring: RefCell<IoUring>`) | `src/comms/process.rs:315` |
| `timer()` **mints a `Receiver` per call** → **one ring per `after`** | `:1470`, `:1509` |
| `after` is called by `call-by-deadline` on **every round-trip of every generated client method** | `wat/service.wat` |
| ring-creation failure has two shapes: a hard `.expect` panic, and an `io::Error` → a **raise** | `:1105` · `:1509` → `runtime.rs:27821` |
| the **thread tier has no ring at all** (crossbeam, futex-based) | `runtime.rs:27796` |

⛔ **The assumption that hides it is stated in the module header itself**: *"FDs are the persistent state;
io_urings are [the ephemeral part]."* Treating a ring as disposable scaffolding is exactly what makes a
per-endpoint — and per-deadline — allocation look free. A ring is not a stack variable; it is a kernel
object with mappings and a setup cost, and it can be **refused**.

### A doctrinal collision worth naming

`docs/arc/2026/06/253-lock-step-audit/STUB.md`: *"every wait in wat must be lock-step: it arrives via the
wire (a blocking `poll(2)`/fd-event)"*, and the runtime **already uses `poll(2)` and `signalfd` directly**
for the shutdown multiplex (arc 170's `DESIGN-FD-MULTIPLEX-SHUTDOWN.md`). So the tree carries **two
polling mechanisms**, and the doctrine names the other one. ⚠ Recording the collision is not proposing a
winner — see the ruling above.

## ⭑ THREE HYPOTHESES FOR THE CI RED, ALL MEASURED DEAD — do not re-walk them

CI red since **2026-09-13 04:36** (`bb993ffd5`, the commit that ADDED the chaos-gate tests), 173+
consecutive failing runs, arm: `IoUring::new(4) failed at timer(): Cannot allocate memory (os error 12)`.

| # | hypothesis | how it died |
|---|---|---|
| 1 | `RLIMIT_MEMLOCK` | `ulimit -l 64` **and** `ulimit -l 0` both PASS locally. io_uring stopped charging the ring to memlock in **Linux 5.12**; this box is 6.12.63, the runner ubuntu-24.04. Both past it. |
| 2 | memory exhaustion | repo is PUBLIC on `ubuntu-latest` → **4 vCPU / 16 GB**. A capped local run (300m/120m) dies by **SIGKILL from the cgroup OOM killer** — a different arm, so not a repro. The cap was ~50× smaller than CI's allocation and proved nothing about it. |
| 3 | `vm.max_map_count` | plausible (`mmap` returns **ENOMEM** past the limit; this box runs `1048576` vs a stock `65530`). Measured during a chaos-gate run: **peak `/proc/<pid>/maps` = 176 lines**, `wat` children **96–100 each**. Not 65 thousand. |

⭐ **And #3 carries a positive finding: rings are NOT accumulating in this workload.** ~100 mappings per
`wat` process is near baseline. The per-endpoint ring cost is **real but not yet pathological at this
scale** — it would become so only under a workload holding many concurrent deadlines. That is the
condition to watch for, and it is the honest reason this is mid-term rather than urgent.

⛔ **Whatever refuses the ring on that runner is not a resource this box can run out of.** That is why
`a2154de5f` shipped instrumentation rather than a fourth guess: all nine `IoUring::new` sites now route
through one counting helper, the failure text carries `rings created-so-far=N` and names the refuted
governor, and CI prints its own kernel/ulimit/cgroup facts before the tests. **The next red diagnoses
itself.**

## ⛔⛔ THE RULING — EXACTLY ONE RING PER THREAD, LAZILY CREATED. THE WAT SURFACE DOES NOT MOVE.

**Builder, 2026-09-16, verbatim:**

> *"it is forcefully modifying wat to have precisely one ring per thread, lazily created.. whatever
> this means for the substrate, i do not care - the wat surface must remain unchanged.. the substrate
> in rust must satisfy the current contracts under the hood"*

This is no longer a menu. The shape is **decided**:

1. **Exactly one `IoUring` per thread.** Not per Receiver, not per Sender, not per timer, not per
   call. Ring count tracks **threads** (bounded, small) and never **waiters** (unbounded).
2. **Lazily created.** A thread that never performs process-tier I/O never allocates one. The thread
   tier uses crossbeam and must continue to allocate **no ring at all**.
3. **Every operation goes through that thread's ring**, demultiplexed by `user_data`.

### ⛔ THE INVARIANT THAT GOVERNS THE WHOLE STONE: THE WAT SURFACE IS FROZEN

**No wat program may be able to tell.** Not by behaviour, not by types, not by timing class, not by
error text. The substrate satisfies the existing contracts underneath or the stone is not done:

- `:wat::kernel::after` keeps returning a **`Peer`** — a timer remains a peer, selectable beside
  other peers, with the same tier rules (`peer-wire?` chooses process vs thread today).
- `:wat::kernel::select` keeps its fan-in over N peers, its `ServiceEvent` arms, its index semantics
  (idx 0 is the peer, idx 1 the timer, as `call-by-deadline` depends on), and its refusal of
  mixed-tier sets.
- `recv` / `recv-by-deadline` / `send` / `try-send` keep their outcome enums **unchanged**:
  `RecvOutcome`, `SendOutcome`, `TrySendOutcome`, `ConnectOutcome`, `CloseOutcome`. No variant added,
  removed, or re-meaninged by this work.
- **No new wat verb is required, and none may be introduced** to make the substrate's job easier.
- Generated `defservice` code is untouched: `child-main`, the serve loop, the client methods,
  `call-by-deadline`, `race-reply`. If a `wat/` file changes, justify it or you have leaked the
  substrate into the surface.

⭑ **The one wat-visible thing that MAY change is a failure that currently cannot be faced** —
`after` raising `MalformedForm` on ring-creation refusal (`runtime.rs:27821`). Making that faceable
is the third missing-form sibling and is **a separate ruling**; it must not be smuggled in here.

### ⭐ THE ACCEPTANCE TEST, and it is unambiguous

**Delete the `ulimit -l` stopgap from `.github/workflows/ci.yml` and CI must stay green.** That
stopgap exists only because ring count tracks waiters; when it tracks threads, the 1024-slot per-UID
budget stops being reachable and the line has no reason to exist. Anything less than removing it is
headroom, not a fix.

Supporting proof, all already built:

- `src/bin/ring-ceiling.rs` on the runner — and a new probe showing **ring count tracking thread
  count, not waiter count**, under the same four-concurrent-process shape that produced
  332+254+254+184.
- `./scripts/floor.sh` green, and the manifest runner's lifecycle rows
  (`the-probes-run-in-the-floor`) green — those exist precisely so a substrate change cannot quietly
  break a lifecycle invariant that was only ever proven once by hand.
- The circuit happy path byte-identical: `distinct=8000;dup=0`, `timeout=yes;…;retry-on=fresh`.

### The obstacles, named up front so they are designed around and not discovered

1. ⛔ **The select/Receiver borrow collision.** `process.rs:~1714` keeps the select ring in a
   *different* `RefCell` from the Receiver's **deliberately**, so select can call Receiver methods
   while holding its own. One shared ring makes that a live re-entrancy bug on day one. This is the
   first thing to design, not the first thing to hit.
2. **Blocking `submit_and_wait(1)` is the current model.** A per-thread ring is compatible with it
   (one wait in flight per thread); a per-PROCESS ring is **not**, which is why this ruling says
   thread. Do not drift toward process-wide without the poller thread that requires.
3. **Receivers crossing threads.** Establish whether a `Receiver` created on one thread is ever
   waited on by another; the ring must be the waiting thread's, not the creating thread's.
4. **Cancellation.** `AsyncCancel` exists because a submitted `PollAdd` must be withdrawn when
   another select arm fires. With a shared ring, cancel must target the right `user_data` and must
   not disturb another waiter's operations on the same ring.
5. **Fork.** `spawn.rs` forks; a child inherits the parent's thread-local ring state and must not
   reuse a ring created before the fork. The runtime already rebuilds `signalfd` in a fork child
   (`runtime.rs` `SHUTDOWN_SIGNAL_FD`) — copy that discipline.

### Shapes to adopt where they fit, once the per-thread invariant holds

None of these are the ruling; they are how the pattern is finished at scale:

1. **Registered files** (`IORING_REGISTER_FILES`) so hot fds stop being re-resolved per submission.
2. **`IORING_SETUP_ATTACH_WQ`** — many rings, ONE shared kernel worker pool. The idiomatic answer if
   thread counts ever make per-thread rings expensive on the kernel side.
3. **Real batching** — today every wait submits one SQE and waits for one CQE, which is the shape that
   makes io_uring indistinguishable from `poll` except in cost.
4. **Timers stop needing a dedicated reactor.** A timerfd is just another fd in a shared ring's poll set;
   `after` minting a whole `Receiver` is what turns a deadline into a kernel object.

⛔ **NOT on the table**: replacing io_uring with `ppoll`/`epoll`. It was raised, costed, and **ruled out
by the builder** — *"we are not undoing anything"*. Recorded so it is not re-proposed as a discovery.

## Provenance

- Excursus `docs/excursus/2026/08/001-sns-sqs/a-deadline-does-not-cost-a-ring/DESIGN.md` — the stone this
  came from, carrying all three refutations with their commands and numbers.
- `a2154de5f` — the ring census and the CI runner-facts step (diagnosis; CI is still red).
- The unfaceable-failure siblings: `a-send-cannot-say-it-is-blocked/DESIGN.md`,
  `a-wait-that-should-be-bounded/FINDING-the-classification.md` §6.
