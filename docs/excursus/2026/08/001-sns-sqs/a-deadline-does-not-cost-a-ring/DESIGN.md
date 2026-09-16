# DESIGN — a deadline does not cost a ring

**Drawn 2026-09-15, builder-directed** (*"get it drawn"*), from a CI red he surfaced: *"we've got a
failing test on ci … idk how long that's been an issue."* **NOT STRUCK.** Its first step is
REPRODUCTION, not a fix (§STEP 1), and one part of it is a ruling (§THE RULING OWED).

## The red, and how long

**Since 2026-09-13 04:36 UTC — ~3 days, 173 consecutive failing runs.** `gh run list --branch sns-sqs
--limit 200` holds exactly **one** green→red transition, at **`bb993ffd5`** — *"STRUCK: the chaos gate
has teeth — an inert injector is now a red"*, the commit that **ADDED** the two failing tests. Last
green: `0b87331fa`. ⭑ **Nothing regressed. The new chaos tests exposed something already true**
(`[[feedback_using_it_in_anger_finds_what_tests_cannot]]`).

Failing (`tests/services/probe_chaos_gate_has_teeth.rs:44`), both in `wat::services`:
`chaos_gate_full_rate_fires_eq_draws` · `chaos_gate_shipped_rate_draws_and_hits_eq_fires`.
Run 35049309366: `Summary [ 725.260s] 5251 tests run: 5249 passed (1 slow), 2 failed, 22 skipped`.

The arm, verbatim:

```
:user::chaos-fires raised: #wat.runtime/MalformedForm {
  :message "malformed :wat::kernel::after form: after: timerfd creation failed:
            IoUring::new(4) failed at timer(): Cannot allocate memory (os error 12)"
  :location #wat.core/Span {:file "wat/service.wat" :line 4292 :col 14 …}
  :head ":wat::kernel::after"}
```

⚠ At the **transition** run (34738278374) the same root cause appeared at a *different* construction
point: `IoUring::new(4) failed at Sender construction: Cannot allocate memory (os error 12)`. Same
resource, two sites — which is the first hint that the site is not the bug.

⛔ **And `wat/service.wat:4292` is inside `race-reply`** — today's `e03fc7f46` extraction. It is **not**
the cause: the identical `(:wat::kernel::after kind …)` call has been on that path all along and the red
predates it by three days. Named here because a reader who greps the line number will land on a
one-hour-old defn and draw the wrong conclusion.

## The mechanism, measured

| fact | where |
|---|---|
| **every `Receiver` owns an io_uring** — `ring: RefCell<IoUring>`, built with `IoUring::new(4)` | `src/comms/process.rs:315`, `:1104` |
| `timer()` **mints a `Receiver` per call** (`Source::Timer`), so **one ring per `after`** | `:1470`, `:1509` |
| ring creation failure has **two** shapes: a hard `.expect` panic, and an `io::Error` | `:1105` (panic) · `:1509` (Err) |
| the `Err` becomes a **raise**, not an outcome: `RuntimeErrorKind::MalformedForm` | `src/runtime.rs:27821` |
| **thread tier is unaffected** — `crossbeam::after`, futex-based, no ring | `src/runtime.rs:27796` |

⚠ **A CORRECTION to the orchestrator's first reading, made in public.** I told the builder *"the timer
path never got the Stone E-1 treatment."* That is wrong as stated. Stone E-1 (`:1149`, `:1358`) retired
the per-**call** ring of the *poll/select* operations, which now borrow the calling Receiver's ring. The
per-**Receiver** ring is by design and was never in E-1's scope. The real defect is one level up:
**`after` mints a whole Receiver — and therefore a kernel ring — per deadline.**

⭐ **Which puts it on the hottest path there is.** `call-by-deadline` is what every generated client
method uses, and it calls `after` on **every round-trip**. So on the process tier, *each deadline-bearing
call allocates and frees an io_uring*. Rings pin locked pages; `ulimit -l` on this box is **8192 KB** and
CI runners are tighter. Enough concurrent deadlines — which is precisely what the chaos gate creates —
and the kernel says ENOMEM.

## ⛔ STEP 1 IS REPRODUCTION. NO FIX MAY BE PROPOSED BEFORE THE RED IS LOCAL.

A CI-only red cannot be proven fixed; a re-run that goes green destroys the evidence and proves nothing.
This one should be reproducible on demand, because the constraint is a **soft limit**:

```
ulimit -l            # 8192 (KB) on this box
( ulimit -l 64 ; cargo nextest run --release -E 'test(chaos_gate)' )
```

**The gate: the local failure must carry the SAME arm** — `IoUring::new(4) … Cannot allocate memory (os
error 12)` — not merely *a* failure. A different arm means a different bug and this DESIGN is wrong.

⭑ That reproduction is also the **negative control for the fix**: after the change, the same command at
the **same lowered limit** must pass. A fix proven only at 8 MB is a fix proven on a roomier box, which
is the thing that let this hide for three days.

⛔ **If it does NOT reproduce**, say so and STOP — report what the runner has that this box does not
(concurrency, cgroup memory, kernel version, `RLIMIT_MEMLOCK`). An honest ABSENT is the finding
(`[[feedback_permit_the_null_and_it_gets_used]]`).

## The fix, four candidates — not ranked, because the repro decides

| # | shape | cost |
|---|---|---|
| a | **A thread-local ring shared by timer Receivers** | smallest change; needs care that a shared ring's CQEs are demultiplexed per-Receiver |
| b | **One ring per thread for ALL Receivers** | fixes the class, not just timers; largest blast radius |
| c | **Timers stop using io_uring** — timerfd read directly, or one timer wheel per thread | removes the resource from the hot path entirely; changes how `select` polls a timer |
| d | **Keep the ring, make exhaustion FACEABLE** | does not fix the cost; see the ruling below |

(d) is **not an alternative to (a)–(c)** — it is the crusade half, and it is needed whatever the
allocation shape, because a kernel can always refuse.

## ⛔⛔ THE RULING OWED — what does `after` return when the kernel says no?

This is the **third unfaceable failure** this session, the same shape each time: a blocked `send` has no
`SendOutcome` variant · `recv-all`'s timeout has no form in `Result :- [(Vector O), LociDiedError]` ·
and now **`after` raises `MalformedForm`, which no arm can face.** Under the standing rule — *"no one is
allowed to crash on a recoverable error"* — kernel resource exhaustion is recoverable: a caller could
back off, or fall back to the thread tier. But `after` returns a `Peer`, not an outcome, so making it
faceable changes its type and every call site. ⭑ **Same decision class as the send mechanism** (item
**3**, the checker lint, would have caught none of these — they are missing forms, not wildcards).

And the `.expect` at `src/comms/process.rs:1105` is a **hard panic** on the same condition: not even a
raise. That one is unambiguous and should become an `io::Error` regardless of the ruling.

## Trap-doors

1. ⛔ **Do not "fix" it by raising memlock in the workflow.** That hides a substrate cost on the hot path
   of every deadline-bearing call, and the next constrained host finds it again.
2. ⛔ **Do not re-run CI to check.** The red is 173 runs deep; it is not in question.
3. **`wat/service.wat:4292` is a red herring** (see above).
4. **The thread tier is the control**: if a change makes thread-tier timers slower or ring-backed, it is
   the wrong change — that tier has no ring today and must keep none.
5. **This stone does not touch `wat/`.** It is `src/comms/process.rs` and possibly `src/runtime.rs`.
