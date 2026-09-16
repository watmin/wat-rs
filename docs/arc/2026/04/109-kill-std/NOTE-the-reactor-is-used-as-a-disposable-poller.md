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

## The direction — MAKE THE REACTOR SUPERIOR, not remove it

Builder's ruling. Candidate shapes for whoever takes it, none of them evaluated here:

1. ⭐ **One ring per I/O-executing THREAD** (lazily created, so only threads that do I/O pay), borrowed
   by Receivers, CQEs demultiplexed by `user_data`. Stone E-1 already established the "borrow the
   caller's ring" pattern (`process.rs:1201`, `:1410`) — this generalises it from *per-call* to
   *per-thread*, and it is the shape every thread-per-core io_uring system converges on. It also makes
   the measured ceiling irrelevant: ring count would track THREADS (bounded, small) instead of WAITERS
   (unbounded). ⚠ Known obstacle, already documented in the code: the select path keeps its ring in a
   *different* `RefCell` from the Receiver's precisely so the borrows do not collide (`process.rs:~1714`).
   Collapse them onto one shared ring and that becomes a live re-entrancy bug on day one — it is the
   first thing to design around, not to discover.
2. **Registered files** (`IORING_REGISTER_FILES`) so hot fds stop being re-resolved per submission.
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
