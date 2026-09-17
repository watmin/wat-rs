# SCORE — one ring per thread, lazily created

**Struck 2026-09-16** (local; the floor stamps below read `2026-09-17` UTC), on `sns-sqs` at
`fedc02257` (the DRAWN commit), implementing
`DESIGN-one-ring-per-thread.md` and the ruling in
`NOTE-the-reactor-is-used-as-a-disposable-poller.md`.

Blast radius: **`src/comms/process.rs`** (the stone), **`src/kernel/spawn.rs`** (two stale doc
comments that quoted the removed ring's byte cost), **`tests/comms/probe_one_ring_per_thread.rs`**
(new), and this SCORE. **No `wat/` file.** No `src/runtime.rs`. Nothing committed — the tree is
handed over dirty for the orchestrator to grade.

⚠ `git diff --stat` says **1374 insertions / 619 deletions**, and that number OVERSTATES the
change: ~450 of those lines are the existing census + thread-ring code **re-indented** into the new
private `mod ring_door` (the privacy gate, row 3). Reading `git diff -w` is the honest way to see
the logic.

## ⛔⛔ READ THIS FIRST — THE STONE BROKE THE CIRCUIT 3× AND THE FIRST GREEN WAS A LIE

Every unit test passed, clippy was clean, the ring census was flat, and the circuit's
**`distinct=8000;dup=0` was correct** — and the run took **70.3 s instead of 23.8 s**, with
**system time 0.215 s → 28.2 s**. The correctness gate could not see it because the defect was a
syscall storm, not a wrong answer.

```
                       wall      user      sys      distinct/dup   store-ms   setup
pre-stone (control)   23.98 s   4.43 s   0.215 s   8000 / 0        5059       13549
stone, first version  70.33 s  10.35 s  28.229 s   8000 / 0       36247       37568
stone, after the fix  23.53 s   4.19 s   0.211 s   8000 / 0        4650       13508
```

**The mechanism, bisected and then fixed:** the first version withdrew a finished operation's
still-armed SQEs by riding `AsyncCancel`s out on the *next* operation's submission — "free",
because they cost no syscall of their own. They were not free. The cancels' own CQEs, plus the
`-ECANCELED` of what they cancelled, made the following `submit_and_wait(1)` **return
immediately with nothing of the caller's generation in it**. `select` read that as "no arm
fired", took its re-poll path, and **armed every fd again** — recording those arms for withdrawal
in turn. `select` became an arm/cancel storm that ended only when data happened to arrive.

Two changes fix it, and both are now the load-bearing comments in the source:

1. **The wait loops never treat a straggler as "nothing fired."** `select`, `select_raw`,
   `wait_for_data_or_cascade` and both bare `Read`s now loop on `submit_and_wait(1)` until a CQE
   **of their own generation** arrives. The arms already submitted stay armed across the extra
   wait, so nothing is re-submitted and nothing spins.
2. **Withdrawal is batched at a watermark** (`WITHDRAW_WATERMARK = 512`) in its own submission,
   so an ordinary `recv`/`select` submits no cancel SQE at all and pays no extra
   `io_uring_enter`.

⭑ **The finding that matters beyond this stone:** a shared ring makes `submit_and_wait(1)`'s
contract weaker in a way that is invisible to a correctness test. "A completion is ready" stops
meaning "your completion is ready", and any code that assumed otherwise degrades into a spin
rather than into a wrong answer. ⚠ And I half-knew it: three of the five wait sites in this file
(`wait_for_data_or_cascade` and both bare `Read`s) got generation-checked loops as I wrote them, for
exactly this reason. `select`'s two I left with the pre-stone single-drain shape because the
surrounding code already had a re-poll path that *looked* like it handled the case. It was the
handler that became the storm.

⚠ **And the instrument that caught it was the circuit, not the floor and not a unit test.** The
regression is pure latency/syscall cost; every assertion in the tree still passed.

## The floor, verbatim — and there are TWO, because the first weighed a tree that changed

**THE FLOOR OF RECORD is the second run.**

```
     Summary [ 559.500s] 5267 tests run: 5267 passed (9 slow), 22 skipped
```

`./scripts/floor.sh`, **`.floor/2026-09-17T00-27-11Z/`**, `[floor] exit=0`, **no `ARM.txt`** (the
directory holds only `clean.log` and `raw.log`). **0** `FAIL` / `SIGSEGV` / `TIMEOUT` / `TRY` lines
in the whole captured log. **559.5 s** sits inside the **540.4–561.5 s** same-code band this box has
recorded, near its top.

⭑ **And the tree it weighed is the tree being handed over**, checked rather than assumed:
`md5sum src/comms/process.rs` was taken at launch (`77da1eac396cfa94bb4c44050f616acb`) and is
identical now; `src/kernel/spawn.rs` likewise (`80e58b15fb2ff82c32041b4fe1fd2080`). The only thing
that has changed since this run started is **this document**, which no gate reads. All 9 of the
stone's tests and all 7 `wat::probes` manifest rows appear in `clean.log`.

⭑ **Why there are two, said out loud.** The first run was green —

```
     Summary [ 556.945s] 5267 tests run: 5267 passed (9 slow), 22 skipped
```

`.floor/2026-09-17T00-06-55Z/`, `[floor] exit=0`, **no `ARM.txt`**, 556.9 s (inside the band) — but
while it was running I found and fixed a real defect in `withdraw_leaked_arms`: on the
cannot-place-a-cancel path it had `mem::take`n the backlog and then **dropped the remainder**, with
a comment claiming the next pass would find them. It would not. `nextest` compiles before it runs,
so that green describes the pre-fix source. Re-weighed rather than reused. Both are reported so
neither is hidden; the only difference between the two trees is that fix, a corrected
`ring_capacity_for` doc comment, and this document.

All **5267** tests ran in both (**8 of them new in this stone**: six in
`comms::process::one_ring_per_thread_tests`, two in `tests/comms/probe_one_ring_per_thread.rs`; the
census test was MOVED into `ring_door::census_tests`, not added).

### Row 6 — the manifest runner's rows, named

All seven `wat::probes` rows PASS in both runs:

```
scratch_pad_manifest::two_faults_before_stop_are_faced_at_all_four_surfaces
scratch_pad_manifest::a_silent_child_cannot_hang_launch_on_the_thread_tier
scratch_pad_manifest::a_silent_child_cannot_hang_launch_on_the_process_tier
scratch_pad_manifest::a_dead_runner_names_the_item_it_orphaned
scratch_pad_manifest::a_handler_raise_does_not_kill_the_service_for_everyone
scratch_pad_manifest::every_manifest_row_has_its_own_test
scratch_pad_manifest::every_manifest_probe_says_it_is_in_the_floor
```

These are the lifecycle invariants that exist *precisely* so a substrate change cannot quietly
break something proven once by hand — five of them drive real spawned children over the process
tier, which is the tier this stone rewrote. They are the rows I would expect a wrong borrow
discipline or a mis-demuxed CQE to redden, and they are green.

## The thirteen rows

| # | what | result |
|---|---|---|
| 1 | ⭑⭑ Ring count tracks THREADS, not waiters — DRIVEN | ✅ **before=1 after=1** for 200 pairs + 200 timers + 1 select on one thread. Plus the system-level control: **max io_uring fds in any one `wat` process 125 → 1** |
| 2 | ⭑⭑ Lazy | ✅ a thread that builds a pair, a timer and a `Select` but never blocks holds **0** rings; `thread_ring_raw_fd()` is `None` → `None` → `Some(fd)` across the first round-trip |
| 3 | ⭑⭑ The thread tier still allocates NO ring | ✅ and now **rustc-enforced**: a ring can only be constructed inside `mod ring_door` |
| 4 | ⭑⭑ The wat surface did not move | ✅ **no `wat/` file changed.** No new verb; no outcome-enum variant added/removed/re-meaninged |
| 5 | ⭑⭑ Floor | ✅ green, **and weighed TWICE** — the first run's tree changed under it; see the floor section for both |
| 6 | ⭑⭑ The lifecycle rows still pass | ✅ all **seven** `wat::probes` rows PASS, named in the floor section (five lifecycle + two meta) |
| 7 | ⭑ Circuit byte-identical | ✅ `distinct=8000;dup=0` **and** `timeout=yes;discarded=yes;redial=Connected;retry-on=fresh` on **every** run — **8 captured runs**: 2 on the fixed stone, 3 on the regressed first version, 3 pre-stone controls |
| 8 | ⛔ No `RefCell` double-borrow anywhere | ✅ **RELEASE-BEFORE-CALL**, driven positively (8-arm select → `Receiver::read_into_acc`, twice over) and with a negative control that shows the panic |
| 9 | ⛔ Fork rebuilds, never reuses | ✅ DRIVEN — the child's inherited per-thread census reads **parent + 1** |
| 10 | ⭑ `ring-ceiling` under concurrency | ⚠ **PARTLY ABSENT, and the reason is the NOTE's own** — this box does not charge rings to memlock, so no ceiling exists here to be reachable. A substitute measurement is reported instead, and it is the sharper one |
| 11 | clippy + tests compile | ✅ `--release --workspace --all-targets`: **0**. `nextest --release --no-run`: clean |
| 12 | ⭑ Cost as a BAND | floor band below; circuit **23.5–23.8 s** against a **23.8–24.2 s** same-box pre-stone control |
| 13 | ⛔ What is NOT done is stated | ✅ below, including the 3× regression's residue and one thing the DESIGN did not ask about that I found |

---

## ⭐ The borrow discipline I chose: RELEASE-BEFORE-CALL, made structural

DESIGN trap-door 1 was the thing to decide before writing code. The choice:

**`with_thread_ring(arms, site, |ring| …)` — the `&mut IoUring` exists only inside a closure.**
The alternative (pass the slot down explicitly, threading a `&mut IoUring` through
`Receiver::read_into_acc`, `take_buffered_frame` and every caller) was rejected: it changes
signatures across the whole tier and makes every future caller re-derive the rule.

⭑ **And the reason release-before-call was already safe is worth recording, because the source
said something subtly wrong about it.** `Select::select` already scoped its ring borrow to a
block and called `Receiver` methods only *after* the block closed. The comment read:

> *"Select-ring borrow released; safe to call Receiver methods below (Receiver borrows its own
> ring; different RefCell)"*

The **code shape** was correct on its own — the borrow really was released. The **parenthetical**
credited the safety to the two borrows living in different `RefCell`s, which is the half that
stops being true the moment there is one slot. So the stone did not have to restructure
`select`; it had to delete a reason and make the shape mandatory rather than conventional. A
nested borrow now panics with a message that names the discipline:

```
process-tier ring re-entrancy: this thread's io_uring is already borrowed by an enclosing
operation. The discipline is RELEASE-BEFORE-CALL — close the ring block before calling a
Receiver/Sender method that needs the ring (arc 109, one ring per thread)
```

⛔ **It PANICS rather than returning an error, deliberately.** Mapping it into `RecvError` /
`SendError` would dress a substrate re-entrancy bug as a peer failure — the exact collapse this
tree keeps removing (`[[feedback_a_fallback_that_collapses_failures_reports_nothing]]`). A
re-entrancy is not a condition a wat program can be in; it is a defect in this file.

## ⭐⭐ Row 1 — the flat count, and a second instrument that does not trust the first

**In-process, the per-thread census** (`comms::process::rings_created()`; printed by the test so
this number is re-readable rather than quoted from a passing assertion):

```
[arc109 row 1] waiters=200 pairs + 200 timers + 1 select —
               rings on this thread: before=1 after=1 (process-wide=1)
```

200 `pair()`s (each driving a real send + recv), 200 `timer()`s, and one `Select` that fires a
timer through its whole path — **1 ring before, 1 ring after.** Pre-stone the same shape costs
**601**: `pair()` built two rings (`Receiver construction` + `Sender construction`), `timer()` built
one, and `Select` built one. ⚠ **That 601 is READ OFF the pre-stone constructors, not measured** —
this test did not exist before the stone, and the per-thread counter it asserts on did not either.
The measured pre/post pair is the `/proc` count below, which is why it is here.

⛔ **Asserted on the PER-THREAD count, not the process-wide one**, and that matters: the
process-wide counter moves when any other thread creates a ring, and under `cargo test` (threads,
not nextest's process-per-test) when any sibling test does. The per-thread counter is new in this
stone and exists so the claim has an instrument that can hold it.

### ⭑⭑ The second instrument: `/proc/<pid>/fd`, counted from outside the program

A census the stone's own code maintains is a weak witness for the stone. So the same claim was
taken **externally**, by counting `anon_inode:[io_uring]` entries in `/proc/<pid>/fd` of every
live `release/wat` process at 0.25 s intervals through a full fanout-circuit run, **on the
pre-stone and post-stone binaries in turn** (same input record, same box, `distinct=8000;dup=0`
both ways):

```
                      wat processes   MAX io_uring fds     MAX SIMULTANEOUS io_uring fds
                      sampled         in any ONE process   across all of them (per-UID)
pre-stone                       31                  125                            500
stone                           31                    1                             29
```

⭐ **500 → 29 is the number the acceptance test is really about.** The per-UID `RLIMIT_MEMLOCK`
budget on a charging kernel is ~1024 rings (the NOTE's measurement: 8 MB ÷ ~8 KB, proven by
332+254+254+184 on the runner). **One** circuit run demanded 500 of them pre-stone; the floor
runs **four** nextest processes concurrently, so its demand was ~2000 against a 1024 budget —
which is why the CI red moved between runs and why one process reported `created-so-far=14` while
another reported 244. Post-stone one run demands 29, so four demand ~116.

⚠ **This is a demand measurement, not the acceptance test.** It says the budget stops being
reachable; only a CI run with the `ulimit -l` line deleted can say CI stays green (row 13).

⭑ One more thing that pair of numbers says: each `wat` process runs **5 threads** and holds
**1** ring, so the count tracks *the threads that actually perform process-tier IO* — one per
process here — not the thread count and certainly not the waiter count.

## Row 2 — lazy, driven twice

`a_thread_doing_no_process_tier_io_creates_no_ring` spawns a thread that builds a `pair()`, a
`timer()` and a `Select`, registers the receiver, and blocks on nothing. Its per-thread census is
**0**. Construction is not IO.

`the_thread_ring_fd_appears_only_after_the_first_io` takes the same claim from the fd side:
`thread_ring_raw_fd()` is `None` on a fresh thread, still `None` after `pair()`, and `Some(fd)`
after one round-trip.

## Row 3 — the thread tier, and the gate that makes it a compiler error

Three statements, weakest to strongest:

1. `src/comms/thread.rs` contains **0** occurrences of `IoUring`, `io_uring` or `comms::process`
   (grep, whole file).
2. `after_timer_peer`'s `is_thread_tier` branch (`runtime.rs:27793`–`27807`) reaches only
   `crate::comms::thread::timer` and `crate::comms::thread::pair` — crossbeam, futex-based. Read,
   not recalled.
3. ⭐ **`new_ring` is now module-private to `mod ring_door` inside `src/comms/process.rs`, so a
   ring cannot be constructed anywhere else in the tier — rustc refuses it.** A future
   `Receiver` that tried to regain a ring of its own would not compile, and no census would have
   to notice it climbing afterwards. This is the invariant gated rather than observed
   (`[[feedback_gate_on_the_property_not_the_path]]`).

⚠ **What the gate cannot see:** `src/bin/ring-ceiling.rs` creates its own rings, deliberately —
it exists to ask a box for its ceiling. The gate covers `comms::process`, which is the tier the
wat surface runs on.

## Row 4 — the wat surface did not move

```
 M src/comms/process.rs
 M src/kernel/spawn.rs
?? tests/comms/probe_one_ring_per_thread.rs
```

No `wat/` file. Therefore: no `defservice` generated code touched, no new verb, no change to
`after`'s return type, `select`'s fan-in, its `ServiceEvent` arms, its idx-0-peer / idx-1-timer
semantics or its mixed-tier refusal, and no `RecvOutcome` / `SendOutcome` / `TrySendOutcome` /
`ConnectOutcome` / `CloseOutcome` variant added, removed or re-meaninged.

`src/kernel/spawn.rs` is **doc comments only** — two blocks justifying the `Box`es on
`ProcessSelectable`'s variants by quoting *"each embeds a persistent `RefCell<IoUring>` by
value"*, 696 and 336 bytes. Those rings are gone, so the numbers were false. Re-measured
(2026-09-16): `Receiver<String>` **56** bytes, `Sender<String>` **4**, `Select<String>` **32**,
`ProcessPeerBundle` **136**, `ProcessSelectable` **16**. The boxes are kept — unboxing `Spawned`
moves fields whose declaration order is a drop-order invariant, which is a separate change with
its own risk and no benefit any measurement asks for. Stated rather than silently left stale.

### ⚠ The one place a REFUSAL's report site moves, named rather than glossed

Rings are now created at a thread's first blocking operation rather than at endpoint
construction. In the world where the kernel **refuses** a ring, the refusal therefore surfaces
at `send`/`recv`/`select` instead of out of `pair()` / `timer()`:

- `Sender::send` → `SendError::Failed(value, <the census text>)` — an **existing** variant,
  carrying the same census string. No enum moved.
- `Receiver::recv` → `RecvError::Failed(<census>)` — likewise existing.
- `:wat::kernel::after` → **untouched.** `timer()` no longer creates a ring, so its
  `MalformedForm` raise (`runtime.rs:27821`) can now fire only for a real `timerfd_create` /
  `timerfd_settime` failure, with its text unchanged. Making that raise **faceable** is the third
  missing-form sibling and a separate ruling; nothing here smuggles it in.

This is a change no wat program can observe except in the world this stone exists to make
unreachable. It is reported because the invariant says *"not by error text"*, and this is the
honest edge of that.

## ⛔ Row 8 — trap-door 1, driven from both sides

**Positively**, `select_over_many_arms_then_a_receiver_read_shares_one_ring`: 8 receivers, all
pre-filled, registered on one `Select` on one thread; eight `select()` calls, each of which runs
the ring closure and *then* calls `Receiver::read_into_acc` on the same thread-local slot — the
exact collision the trap-door warned about. Every arm delivers its own message (index semantics
are part of the frozen surface, so they are asserted, not assumed). Then the whole fan-in is run
a second time:

```
[arc109 row 8] 8-arm select x2 + 8 sends x2 — rings on this thread: before=1 after_pass1=2 after_pass2=2
```

⭑ **That `1 → 2 → 2` is the capacity discipline, visible.** An 8-arm select needs a wider ring
than the 2-arm sends did, so the thread's ring is rebuilt **once**; the repeat of the same fan-in
creates **nothing**. Growth is bounded by log2(widest fan-in) per thread, not by waiters.

**Negatively**, `a_nested_ring_borrow_names_the_reentrancy` is a `#[should_panic(expected =
"process-tier ring re-entrancy")]` that nests two `with_thread_ring` calls. Without it the
positive test proves only that *this* arrangement happens not to collide.

⭑ And the borrow-discipline change generalises one more site: a **`Receiver` created on one
thread and waited on by another** (DESIGN trap-door 4) now submits on the *waiting* thread's
ring, because the ring is reached by thread-local **lookup** and never held in an owner field.
There is nothing to migrate and nothing to get wrong. ⚠ Whether the tree actually does this was
**not** established — the DESIGN asked to establish it, and I made the question moot instead of
answering it. Saying so rather than claiming a survey I did not run.

## ⛔ Row 9 — fork rebuilds, and the witness is a NUMBER, not the absence of a hang

`a_fork_child_rebuilds_the_thread_ring_and_never_reuses_it`. io_uring marks a ring's mappings
`MADV_DONTFORK`, so a `clone3` child inherits the thread-local's *bytes* and not the ring; a
child that reused it would submit into nothing. The guard is the `pid` stored beside the ring and
compared to `getpid()` at every entry — `runtime.rs`'s `SHUTDOWN_SIGNAL_FD` rebirth discipline,
copied.

Two things make the probe a proof rather than a smoke test:

- **The non-vacuity step.** The parent does a full round-trip *before* forking, so the child
  inherits a **populated** slot. Without that the child builds a ring for the ordinary lazy
  reason and the test would pass in a world with no pid guard at all.
- **The witness.** `RINGS_CREATED_THIS_THREAD` is a thread-local, so the child inherits the
  parent's count by COW. The child echoes its own count back, and the parent asserts it is
  **`parent + 1`**. A *reuse* prints `parent`; *more than one rebuild* prints higher. Those are
  three different worlds printing three different numbers, which is what
  `[[feedback_a_fallback_that_collapses_failures_reports_nothing]]` asks for.

⭑ A related fact found while discharging this, which changes how the DESIGN's trap-door reads:
**`spawn.rs`'s child `execve`s** (arc 170), so the production fork path gets a fresh process image
and could not have reused a ring anyway. The fork-without-exec paths that *can* — and this probe
is one — live in the test tree. So the pid guard is load-bearing for the tests and for anyone who
forks without exec later; it is not what saves `spawn-process'`.

## ⛔ Row 2 of the DESIGN's trap-doors — the fd-survival lists, and why element 1 is GONE

`Sender::raw_fds` and `Receiver::raw_fds` returned `[data_fd, ring_fd]`, and the module header
said each endpoint *"owns its data fd AND its io_uring ring fd; both must survive"*. The ring fd
is **removed** from both lists. The DESIGN said those lists *"must now name the thread's ring"*;
they cannot, honestly, and here is what was measured before deciding that:

1. The sweep the second element existed for — `close_inherited_fds_above_stdio` /
   `child_post_fork_init_preserving` — **no longer exists in `src/` at all.** `grep -rn` over the
   whole tree finds it only in three comments inside `process.rs` itself. Arc 170's child
   `execve`s: it inherits 0/1/2 and the lifeline and nothing else.
2. **Every caller in the tree indexes `[0]`** — `spawn.rs:1015`, `:1016`, `:1042`–`:1044`, and
   four probes (`probe_arc278_over_budget_recovers`, `probe_arc278_sender_grows_a_ring`,
   `probe_arc278_partial_frame_residue`). **Nothing read element 1.**
3. An endpoint cannot name the thread's ring truthfully: it may not exist yet when a caller asks,
   and a raw fd captured on one thread is the wrong ring on another.

So the thread's ring fd is asked for at the moment it is needed, by
**`comms::process::thread_ring_raw_fd() -> Option<RawFd>`**, which is `None` exactly when the
thread owns no ring. The `raw_fds` doc comments now carry this reasoning at both sites.

## The demultiplexer, and the silent channel death it removed

`user_data` is now `generation << 16 | tag`, where the generation comes from a thread-local
counter (one per submission batch — one per `write_once` attempt, not per `send`, or a queued
withdrawal could cancel the next attempt's live `Write`). Generation **0** is never handed out and
is what the withdrawal `AsyncCancel`s carry, so every drain discards them with no special case.

⭐ **`wait_for_data_or_cascade` had this line, and on a shared ring it is a bug:**

```rust
            // Unreachable: we only push two SQEs with these two tokens.
            _ => return Err(RecvError::Disconnected),
```

True of a private ring. False of a shared one — a straggler from any earlier operation would have
arrived here and been reported as **a clean channel death on a healthy channel**. The same shape
was in both `select` drains, where an unknown token was turned into a data arm *arithmetically*
(`token - 1`): on a shared ring that fires the wrong arm, or panics on underflow when the token is
0. All three are now generation-checked.

⭑ **Discarding a straggler is lossless, and the reason is a property of the arms:** every one is a
level-triggered `POLLIN | POLLHUP` poll, so a readiness that was true is still true and this
generation's own arm on the same fd reports it. That property is what lets a shared ring work
without a broker thread — it is the load-bearing assumption of the whole demux, and it is written
down in `RingGen`'s doc.

## Capacity: grow-only, and why the shrink had to go

Stone E-2's *"reflexive rebuild discipline"* rebuilt on any capacity **mismatch**, grow or shrink,
because the ring served one `Select` site. A per-thread ring serves every operation on the thread,
so it must be as wide as the **widest live submission** — and a shrink is a ring **creation**,
i.e. exactly the cost this stone removes. It would also have thrashed: a thread alternating an
8-arm select with a 2-arm recv would rebuild twice per round-trip and the census would track work.
So: `cap < needed` rebuilds, `cap > needed` does not. `ring_capacity_for(arms) =
next_power_of_two(arms * 2).max(4)` — doubled for headroom (a withdrawal pass batches through the
same queue, the CQ is sized off this, and there is exactly ONE ring per thread so entries are the
cheap axis). ⚠ It is **not** doubled so cancels can ride on the caller's submission; that was the
first design and it is what cost 3×.

## Row 10 — `ring-ceiling`, and the part that is honestly ABSENT

```
kernel=6.12.63+deb13-amd64 nproc=12
memlock=Max locked memory 8388608 8388608 bytes  max_map_count=1048576  io_uring_disabled=0
held=5000: CommitLimit=42349156kB Committed_AS=1053908kB MemAvailable=30191976kB VmSize=45288kB
NO CEILING FOUND under cap=5000 — 5000 rings held simultaneously
```

⛔ **This box cannot answer the row as asked, and the NOTE already says why:** Debian 6.12.63 does
not charge rings to `RLIMIT_MEMLOCK`, so there is no ceiling here for a 4-way parallel run to
reach — before the stone or after. Running four concurrent `ring-ceiling`s would print "no
ceiling" four times and prove nothing about the runner. *"The number must come from the box that
refuses"*, and that box is CI.

**ABSENT: whether the per-UID ceiling is still reachable by a 4-way parallel run.** What replaces
it is the demand measurement in row 1 — **500 → 29 simultaneous rings per circuit run** — which
is the quantity the ceiling would be compared against, taken on the box I have.

## Row 12 — cost, as bands

- **Circuit**: **23.53 / 23.78 s** (stone, 2 runs) against **23.77 / 23.98 / 24.19 s** (pre-stone,
  3 runs, same box, same input record, `git stash` + rebuild both ways with md5-verified restore).
  A **23.5–23.8 s** band against a **23.8–24.2 s** band — overlapping-to-marginally-faster, and both
  inside the BREADCRUMB's ~20–25 s. The 70.3 s first version is quoted above as the regression, not
  as the stone. ⚠ Two of the pre-stone runs carried the `/proc` sampler or `time`; the sampler was
  measured not to matter (70.75 s sampled vs 70.33 s plain on the regressed build).
- **Floor**: **559.5 s** (floor of record) and **556.9 s** (the first run, near-identical tree) —
  both inside the **540.4–561.5 s** band, in its upper half. ⚠ **Reported as a band membership, not
  a delta**: with a ±16 s noise floor, 559.5 − 556.9 = 2.6 s is not a measurement of anything, and
  neither is either number's distance from the band's midpoint. The floor grew from **5251** tests
  (the band's original runs) to **5267**, so a same-code comparison across that gap does not exist
  either.

⚠ Per the BREADCRUMB's own correction, this box's floor noise floor is **at least ±16 s**, so the
floor cost is a band against 540.4–561.5 s and never a delta.

## Row 13 — what is NOT done

- ⛔ **The acceptance test is the ORCHESTRATOR's.** Deleting the `ulimit -l` stopgap from
  `.github/workflows/ci.yml` and seeing CI stay green needs a CI run; it is untouched here, as
  instructed. The demand measurement above is evidence *for* it, not it.
- **`after`'s ring-refusal is still unfaceable.** `runtime.rs:27821` still raises
  `MalformedForm`, which no arm can face. Separate ruling. `timer()` no longer creates a ring, so
  that raise's *ring* cause is gone — but the raise, its text and its unfaceability are exactly as
  they were.
- **`ATTACH_WQ`, registered files, SQPOLL, real batching: out of scope**, as the DESIGN says. Every
  wait is still `submit_and_wait(1)`. ⭑ The NOTE's deeper point stands unaddressed: the count was
  the visible bill, and the missing amortization is the actual inversion. The corrected principle
  is now written in the module header so the next reader starts from it.
- **`ProcessSelectable`'s `Box`es are now headroom, not lint fixes** (56-byte `Receiver`). Left
  alone; unboxing touches drop-order-sensitive field order.
- ⚠ **A bounded leak replaced an unbounded one; it is not zero.** Up to `WITHDRAW_WATERMARK = 512`
  armed `PollAdd`s per thread can be outstanding, each holding a `struct file` reference — so a
  pipe read-end whose last fd is closed can stay referenced until the next withdrawal pass. **For
  comparison, the pre-stone code had no bound at all**: `Select`'s persistent ring (Stone E-2)
  never withdrew a non-firing arm either, and a serve loop's `Select` lives as long as the
  program. This is a bound where there was none, and the watermark is the dial. **Not driven** —
  no probe asserts that a closed pipe's writer sees EOF within one watermark period.
- ⚠ **The abandoned-`Read` hazard is mitigated, not closed.** A bare `Read`'s buffer is on the
  caller's stack; the error paths now `withdraw` the `Read` before returning, but `AsyncCancel` is
  asynchronous, so a bailed-out `Read` is *asked* to stop rather than guaranteed to have stopped.
  This is pre-existing (Stone E-1's persistent ring had the same shape) and is called out at both
  sites rather than fixed here.
- ⚠ **`getpid()` is now called once per ring borrow** — one of the cheapest syscalls, on a path
  that already makes an `io_uring_enter`, and the circuit's system time is unchanged (0.215 s →
  0.211 s). It is the price of the fork guard being checked rather than assumed.
- **The `select` fan-in ceiling is checked, not assumed.** `select_arm_ceiling` refuses a fan-in
  wide enough for a data arm's tag to collide with the reserved listener tag (65533). Unreachable
  in practice — `IORING_MAX_ENTRIES` is 32768, so the kernel refuses the ring first — and checked
  anyway, because the alternative is a silent mis-demux and *"the ring would have failed first"* is
  a claim about another subsystem's limit.

---

## ⭑ REGRADED BY THE ORCHESTRATOR ON HIS OWN RUNS, 2026-09-16/17 — all load-bearing rows hold

```
floor    Summary [ 552.896s] 5267 tests run: 5267 passed (9 slow), 22 skipped
         .floor/2026-09-17T00-43-48Z/ · exit=0 · NO ARM.txt · inside the 540.4–561.5 s band
clippy   0 · nextest --no-run clean
```

| row | the orchestrator's own result |
|---|---|
| 1 | ✅ **re-measured EXTERNALLY**, sampling `anon_inode:[io_uring]` in `/proc/<pid>/fd` through a full circuit run: **max 1** ring fd in any one `wat` process (pre-stone 125), **max 29 simultaneous** across all 31 processes (pre-stone 500). Independent of the in-process census. |
| 4 | ✅ `git status`: `M src/comms/process.rs`, `M src/kernel/spawn.rs`, plus the new probe and this SCORE. **Zero `wat/` files.** The surface freeze held. |
| 5 | ✅ 552.896 s, inside the band. |
| 6 | ✅ both new probes PASS: `select_over_many_arms_then_a_receiver_read_shares_one_ring`, `a_fork_child_rebuilds_the_thread_ring_and_never_reuses_it`. |
| 7 | ✅ `distinct=8000;dup=0` **and** `timeout=yes;discarded=yes;redial=Connected;retry-on=fresh`. |
| 12 | ✅ **the regression check re-taken independently** — see below. |

### ⭐⭐ THE REGRESSION CHECK IS THE ONE THAT MATTERED, AND IT WAS RE-TAKEN

The executor's first version made the circuit **3× slower** — 70.3 s wall, `sys` **0.215 s → 28.2 s** —
while *every unit test passed, clippy was clean, the ring census was flat, and `distinct=8000;dup=0`
was CORRECT*. Orchestrator's own post-fix measurement, three runs:

```
wall 23.43 s · 23.15 s · 23.50 s          (pre-stone control: 23.98 s)
sys  0.176 s                               (pre-stone control: 0.215 s — the stone is BELOW it)
```

★★ **The generalisable finding, and it outlives this stone:** on a shared ring, *"a completion is
ready"* stops meaning *"**your** completion is ready."* Code that assumes otherwise degrades into a
**SPIN, not a wrong answer** — so it is invisible to every correctness gate. `select` read another
generation's straggler as "no arm fired", took its re-poll path, re-armed every fd, and became an
arm/cancel storm. ⛔ **The circuit caught it; the floor and the unit tests did not.** That is
`[[feedback_using_it_in_anger_finds_what_tests_cannot]]` with a number attached, and it is the
argument for keeping a whole-system run in the loop for every substrate change.

### Three findings accepted as reported, each worth more than the diff

1. **`select`'s code shape was already correct; only its COMMENT was wrong.** It credited safety to
   "different `RefCell`s" when the actual safety came from releasing before calling. A comment that
   attributes correctness to the wrong mechanism is a trap for exactly this kind of change.
2. ⛔ **`wait_for_data_or_cascade`'s `_ => Err(Disconnected)`** — commented *"Unreachable: we only push
   two SQEs"* — would have become **a silent channel death on a healthy channel** on a shared ring.
   An "unreachable" arm is a claim about the surrounding world, and the world moved.
3. ⭐ **`new_ring` is now module-private inside `mod ring_door`** — the one-ring invariant is
   **rustc-enforced, not observed.** That is the pattern this campaign keeps rediscovering: make
   something that cannot be careless do the checking.

### Gaps, accepted as declared

- **Row 10 (four-way concurrency) is ABSENT** and correctly so: this kernel does not charge rings to
  memlock, so `ring-ceiling` finds no ceiling at 5000 and there is nothing here to make reachable. The
  substituted demand measurement (500 → 29) is the right instrument and the orchestrator re-took it.
- **Trap-door 4** was made moot rather than surveyed (thread-local *lookup*, no owner field) — a
  legitimate resolution, stated as such rather than claimed as a survey.
- **Not driven:** the bounded ≤512 armed-`PollAdd` residue per thread, and the abandoned-`Read`
  hazard. Named, not hidden.

### ⛔ WHAT THIS DOES NOT YET PROVE — the acceptance test is the orchestrator's

`DESIGN-one-ring-per-thread.md` sets acceptance as **deleting the `ulimit -l` stopgap from
`.github/workflows/ci.yml` and CI staying green.** That needs a CI run and is the orchestrator's to
take. Demand fell 500 → 29 per run (≈116 across four concurrent processes, against ~1024), which is
the arithmetic reason to expect it to pass — **but expecting is not measuring**, and until that run is
green the stone is unproven on the only box that ever refused a ring.
