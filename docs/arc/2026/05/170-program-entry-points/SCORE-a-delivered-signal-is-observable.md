# SCORE — a delivered signal is observable

**SCORED.** Executor: grok, 2026-09-08. Did not commit.

```
Summary [ 500.875s] 5235 tests run: 5235 passed (7 slow), 22 skipped
```

`.floor/2026-09-08T18-51-03Z/`

5232 + **3** tests = **5235**. 0 FAIL, 0 TIMEOUT.

The regression floor (captured, not re-run) remains `.floor/2026-09-08T18-08-14Z/` —
`5231 passed, 1 failed` on `deftest_wat_tests_service_signal_observer_measures_itself`.

The previous stone's drain classifier is still dirty in `src/runtime.rs` and is
not this strike.

## WHAT LANDED

Flag readers are the authority on "has this process received X". Atomic first,
pending-set second. The union has no gap after `Delivered`:

```
t0  kill(SIGHUP)   → pending gains SIGHUP,  atomic false  → union TRUE
t1  worker drains  → pending clears,        atomic true   → union TRUE
```

The two families differ because a reset must be able to clear the record, and
the worker must still see a stop:

| reader | probe | consumes? | why |
|---|---|---|---|
| `sighup?` `sigusr1?` `sigusr2?` | `sigtimedwait({that one}, 0)` then latch | **yes** | `reset-*!` clears the latch; a still-pending signal would re-report |
| `stopped?` | `sigpending()` membership of SIGINT/SIGTERM | **no** | a reader that dequeued SIGTERM would leave the worker asleep and shutdown would not cascade |

`sigtimedwait` uses a **zero timespec**, not NULL. EAGAIN/EINTR are "nothing
observed this call", not errors. The set contains exactly one signal.

Live path was `src/runtime.rs` (`eval_kernel_stopped` / `eval_user_signal_query`);
`ambient.rs` still only delegates. Worker, mask, signalfd, demux, drain
classifier: untouched.

## THE TESTS

- `never_signalled_process_reads_false` — all four false; < 50 ms
- `reset_sighup_clears_with_no_drain_between` — SIGHUP → true → store false → false, no worker
- `consuming_sighup_leaves_sigterm_pending` — child (no worker): consume SIGHUP, SIGTERM still pending, `stopped?` true, `KERNEL_STOPPED` still false

`wat-tests/` is empty. No sleeps, no retries.

## ROW 1 — 20 consecutive runs of the failing arm

20/20 PASS. Each isolated nextest run of
`deftest_wat_tests_service_signal_observer_measures_itself` passed in ~0.85 s.
Floor instance: **2.122 s PASS**.

## THE LOAD-BEARING ARMS

| probe | isolated | floor |
|---|---|---|
| `a_signal_is_an_fd` | 0.017 s PASS | **0.092 s** PASS |
| `partial_frame_residue` | **3.020 s** PASS | **3.033 s** PASS |
| `send_poll_arm` | 0.016 s PASS | **0.018 s** PASS |
| `shutdown_priority_is_the_ruling` | 0.004 s PASS | **0.008 s** PASS |

## THE ROWS

| # | row | result |
|---|---|---|
| 1 | ⛔ failing arm, 20 consecutive | ✅ 20/20 PASS |
| 2 | ⛔ `stopped?` never consumes | ✅ `sigpending`; debug_assert on the consuming helper refuses SIGINT/SIGTERM |
| 3 | ⛔ shutdown still cascades | ✅ residue **3.033 s**; send_poll_arm / shutdown_priority PASS |
| 4 | ★ `reset-sighup!` still clears | ✅ no drain between observe and reset |
| 5 | ★ resettable readers consume selectively | ✅ SIGHUP consumed, SIGTERM still pending |
| 6 | latched fast path costs no syscall | ✅ atomic tested first in both helpers |
| 7 | never-signalled reads false | ✅ all four false |
| 8 | worker still dispatches | ✅ `a_signal_is_an_fd` PASS |
| 9 | test was not made patient | ✅ `git diff -- wat-tests/` empty |
| 10 | blast | ✅ reader path + 3 tests. Worker/mask/demux/classifier not this stone |
| 11 | the floor | ✅ `Summary [ 500.875s] 5235 tests run: 5235 passed (7 slow), 22 skipped` |

## BLAST

This stone:

```
 src/intrinsic/kernel/ambient.rs                         |  23
 src/runtime.rs                                          |  helpers + two evals
 tests/kernel/probe_arc170_delivered_signal_is_observable.rs | 154 (new)
```

`src/runtime.rs` still also carries the previous stone's 117-line drain
classifier (uncommitted). The worker loop, the blocked mask, the signalfd,
and the demux were not edited for this strike.
