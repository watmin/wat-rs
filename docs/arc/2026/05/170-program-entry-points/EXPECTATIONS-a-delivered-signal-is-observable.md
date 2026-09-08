# EXPECTATIONS — a delivered signal is observable

Written **before** the strike. Repairs a regression already in the tree at `5094221aa`.

Rows state what must be true, not where to look.

## Rows

| # | what must be true | how it is checked | expected |
|---|---|---|---|
| 1 | ⛔ **the failing arm passes, REPEATEDLY** | `deftest_wat_tests_service_signal_observer_measures_itself`, **20 consecutive runs** | 20/20 PASS. One green run cannot prove a race fixed |
| 2 | ⛔ **`stopped?` never consumes a stop signal** | read the diff | it uses non-destructive `sigpending()`; nothing on a reader path dequeues `SIGINT`/`SIGTERM` |
| 3 | ⛔ **shutdown still cascades** | `probe_arc278_send_poll_arm`, `partial_frame_residue`, `shutdown_priority_is_the_ruling` | PASS; residue in **~3 s** — read the duration |
| 4 | ★ **`reset-*!` still clears** | new test | after `SIGHUP` → `(sighup?)` true → `(reset-sighup!)` → `(sighup?)` **false**, with no drain in between |
| 5 | ★ **the resettable readers consume selectively** | new test | consuming `SIGHUP` at the reader leaves a pending `SIGTERM` untouched |
| 6 | **the latched fast path costs no syscall** | read the diff | the atomic is tested first; the kernel is consulted only when it is false |
| 7 | **a never-signalled process reads false** | new test | all four readers false on a fresh process; no blocking, no spurious true |
| 8 | **the worker still dispatches** | `a_signal_is_an_fd` | PASS — SIGUSR1 flips its flag and does not stop; SIGTERM stops |
| 9 | **the test was not made patient** | `git diff -- wat-tests/` | **empty**, or a change that strengthens rather than relaxes. No sleeps, retries, or loosened asserts |
| 10 | **blast radius** | `git diff --stat` | the reader path (`ambient.rs` and/or `runtime.rs`) plus tests. Not the worker, not the mask, not the demux |
| 11 | **the floor holds** | `scripts/floor.sh` | **read the Summary line**: 5232 + any tests you add (state the number), 22 skipped, **0 FAIL, 0 TIMEOUT**, quiet box |

## The rows that carry it

⛔ **Row 1 is the stone, and 20 runs is the row — not a formality.** This defect survived two
independent green floors. A single pass proves nothing about a scheduling race; if 20 is impractical,
say so and give the number you ran.

⛔ **Row 2 is the catastrophe guard.** If a reader dequeues `SIGTERM`, the worker never wakes, the
broadcast never fires, and the process becomes unstoppable — a far worse bug than the one being fixed,
and one no existing test would catch.

⛔ **Row 9 exists because relaxing the test is the cheap way out.** The contract is what is broken.
A `wat-tests/` diff that adds patience would pass rows 1 and 3 while leaving the defect in place.

★ **Row 5 is the derivation, tested.** `sigtimedwait` with a single-signal set was measured selective
on this box; the test pins that the implementation actually relies on that and not on a wider set.

## Runtime prediction

**60–100 minutes.** A small reader change, three or four new tests, and 20 repeat runs of a 2 s test.
The floor is the long pole.

## Trap-doors named in advance

- **`sigtimedwait` needs a zero `timespec`, not `NULL`.** `NULL` blocks forever.
- **`EAGAIN` from `sigtimedwait` is the normal "nothing pending" answer** — not an error to report.
- **`EINTR` from `sigtimedwait` is possible**; treat it as "nothing observed this call", never as fatal.
- **The set must contain exactly one signal** for the resettable readers. A wider set could dequeue a
  sibling and lose it.
- **Latch the atomic when the reader consumes**, or the signal is consumed and forgotten — the reset
  contract needs something to clear.
- **Order matters:** atomic first. Checking the kernel first would add a syscall to every latched read.

## What this stone does NOT claim

⚠ It does **not** revisit the mask, the signalfd, the demux, or the drain classifier — those are STRUCK
and correct.
⚠ It does **not** change `Delivered`'s meaning at the sender; it makes the receiver's observation
match what `Delivered` always implied.
