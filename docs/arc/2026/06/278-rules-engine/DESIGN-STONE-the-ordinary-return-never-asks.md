# DESIGN STONE — the ordinary return never asks, and never tells

> **Status: DRAWN 2026-08-05.** Task #79. Surfaced by an intermittent floor failure the builder read
> as *"this screams we've violated lockstep somewhere — go review zero mutex."* He was right, and
> the asymmetry is on the disk. **Race conditions are maximal priority when we encounter them.**

## What is CERTAIN (grounded, quoted, not inferred)

There are two teardown paths for the stdio defservices and **only one is a protocol.**

```
SIGTERM (KERNEL_STOPPED true)   ask <fqdn>/stop → AWAIT Status::Stopped → collect → publish → exit non-zero
ordinary return                 drop(runtime) → Handle drop → :Shutdown (SEVER)     → nothing
```

The code says it outright:

- `freeze.rs:1374-1381` — *"MAIN creates the stdio services, so MAIN stops them, on its way out,
  **ONLY when a stop was actually requested** (`KERNEL_STOPPED`, set synchronously by the signal
  handler): **an ordinary return never asks anything**, it falls straight through to the same
  `drop(runtime)` below it always has."*
- `freeze.rs:113-116` — the Handle is *"kept alive — dropping it, **on an ordinary return**, is what
  signals the service to `:Shutdown`"*, while the `/stop` caller is used *"on a **SIGTERM** return."*

### ★ The certain defect is the HONESTY half, not the timing half

The SIGTERM path implements the builder's ruling — *"any failure must be loud and obvious"* —
collecting every failure into `StopFailed`, publishing it, exiting non-zero. **The ordinary path
collects nothing.** It runs in `Drop`, which cannot return `Result`; `freeze.rs:109-111` concedes the
errors are *"logged to stderr via `eprintln!` and do not propagate."*

**This is R59's mask, alive on the path R59 never examined.** R59 (`NISI FRANGAS, NIHIL PROBAS`)
found `Admin::Stop` had never once been delivered — every ask failed on `ThreadOwnedCell`'s owner
check, invisible behind a `let _ =` — and the floor was green throughout, *because nothing in it
depended on the mechanism.* **Nothing depends on an ordinary return's stop succeeding either.** Same
shape, one path over. R57's sentence, again: the LAW's "complete" was HALF.

## What is SUSPECTED (and must not be reported as cause)

`Drop`'s entire justification is *"the services are idle in `poll'`, so shutdown wakes them cleanly —
no deadlock"* (`freeze.rs:220-222`) — **an assumption about being idle, asserted on the path that
never asks.** Thirty lines above, the SIGTERM path documents why severing is dangerous:

> *"a service's serve loop blocks in `select'`, and `Select::select` registers `shutdown_rx()` as an
> INTERNAL arm that returns `Shutdown` **regardless of which user receivers are pending** — so a
> severed service wakes and exits **WITHOUT ever draining** the `Admin::Stop` sitting in its queue"*

That reasoning was aimed at `Admin::Stop`. **It generalises to any pending work.**

### Evidence AGAINST the simplest story, recorded so it is not re-derived

- `write_via_stdout` (`services/verbs.rs:70-85`) blocks until the line is *"emitted + acked on
  success."* So by the time `println` returns, the bytes are through. **The naive
  last-frame-lost story does NOT hold.**
- `stdout-svc` owns a **dup** of fd 1 — that keeps the pipe open *longer*, not shorter. Not a
  data-loss path.
- `Drop` uninstalls only the CURRENT thread's `ThreadIO`. Another thread's cached stdio peer outlives
  it. **A real hazard for a multi-threaded program — irrelevant to the single-threaded child in the
  failing test**, but worth its own line.

## ⛔ THE INSTRUMENT — a deliberate BREAK, never another re-run

~15 floor re-runs have not reproduced the intermittent failure. **More runs cannot answer this.**
R59 earned its finding by closing the harness's stdout pipe **on purpose** and reading where the
failure landed. Reuse that instrument — it is the one this exact subsystem has already yielded to.

### Probe 1 — prove the MASK (the certain half). This does not need the race.

Make a stop FAIL, then return **normally**. Compare against the same break on the SIGTERM path.

| path | broken stop | expected today |
|---|---|---|
| SIGTERM | pipe closed on purpose | `StopFailed` naming the service, on stderr, non-zero exit |
| **ordinary return** | **the same break** | **silence, exit 0** |

**Same broken state, one path loud, one mute.** That differential IS the finding, it is
deterministic, and it needs no race at all. If it comes back symmetric, the mask does not exist and
this whole stone retracts — which is exactly what a disconfirming probe is for.

### Probe 2 — the ARM, for when the race next fires

The intermittent failure's arm is **still unknown** — the first investigation truncated the log, then
re-ran in isolation where it passed. Each arm of `wat-tests/test.wat:290` carries a distinct message
(`Closed` / `Stopped` / `Lost` / a plain `assert-eq` mismatch) and **each predicts a different
mechanism.** The fix here is process, not code: when it next fires, capture the full failure text
unpiped and ANSI-stripped. Do not re-run before reading it.

### Probe 3 — the TWIN, to locate the race if it is real

The failing test's own header says it is a *"Duplicate of `:wat-tests::test::test-assert-stdout-is-matches`
at line 132 — same hermetic-print-and-capture pattern."* **If the race lives in the shared mechanism,
the failure should MOVE between the two under repeated load.** If only one ever fails, the diff
between the two files is the lead. (`[[feedback_the_mirror_is_an_instrument_not_a_fix]]`)

## The disposition owed — the builder's

The ordinary-return path should either:

- **(a)** run the same ask-and-await protocol as SIGTERM — symmetry, and the failure becomes loud; or
- **(b)** carry a **written derivation** for why severing is safe there.

**Today it has neither. It has an assumption.** And per `ZERO-MUTEX.md`'s own empirical section, the
only bug class that has ever actually bitten this architecture is *"an ordering bug (shutdown
cascade)"* — which is precisely the shape an unasked, unawaited sever produces.

## STOPs

- **⛔ Do not close the intermittent failure on a green re-run.** "Not reproducible" describes your
  SEARCH, not the bug. Two prior dispositions of this exact passes-isolated/fails-under-load shape
  (seams 24v, 24w) were both wrong; 24x root-caused it as real.
- **⛔ Do not report the suspected half as the cause.** Probe 1 proves the mask; only the arm proves
  the race.
- **⛔ Do not "fix" it by making `Drop` louder.** `Drop` cannot return `Result` — that is the
  structural reason the mask exists, and it is why (a) means moving the ask OUT of `Drop`, not
  decorating it.
- **⛔** No `_` wildcard arm on an enum scrutinee.
