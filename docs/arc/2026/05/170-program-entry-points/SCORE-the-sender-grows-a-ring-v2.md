# SCORE v2 — the sender grows a ring

**SCORED.** Executor: grok, 2026-09-08. v1 is NOT STRUCK. The v1
`Ok(0) => continue` arm could not terminate. It is deleted. Did not
commit.

```
Summary [ 500.563s] 5226 tests run: 5226 passed (7 slow), 22 skipped
```

`.floor/2026-09-08T09-49-05Z/`

## THE ARM THAT COULD NOT TERMINATE

`submit_and_wait` is `enter(sq_len, want)` (`io-uring-0.7.14/src/submit.rs:172-214`).
The `Ok` value is SQEs **submitted**, not completions.

v1:

```rust
Ok(0) => continue,   // thought: 0 CQEs, resume waiting
```

Once the SQ is empty, `submit_and_wait(1)` returns `Ok(0)` immediately
and forever while a CQE sits undrained. The empty-drain path that
caught the first-floor red then re-entered this helper with an empty
SQ and spun.

```
iter 1: SQ has 2 SQEs -> Ok(2) -> drain EMPTY (the red) -> continue
iter 2: SQ empty      -> Ok(0) -> continue -> Ok(0) -> … never returns
```

The green v1 floor did not exercise that second iteration. A live-lock
would have been a 30 s TIMEOUT; the floor had none.

## WHAT REPLACED IT

The `Ok(0) => continue` arm is gone. `Ok(_) => return Ok(())` already
covers it. With `min_complete = 1`, an empty-SQ wait **blocks until a
completion is available**, then returns `Ok(0)`; the helper returns;
the caller drains.

`write_once`'s empty-drain `continue` (`:428`) stays — that is the
guard for a wait that yielded no completion (the signal case). Each
retry now blocks instead of spinning.

Confirmed against the pinned crate: `let len = self.sq_len();` …
`enter(len as _, want as _, …)`.

## THE LOAD-BEARING ARMS

| probe | isolated | floor |
|---|---|---|
| `partial_frame_residue` | **3.02 s** PASS | **3.017 s** PASS |
| `send_poll_arm` | 0.01 s PASS | **0.022 s** PASS |
| `the_senders_tie_break_is_a_property` | 0.01 s PASS | **0.021 s** PASS |

No hang. STOP-1 did not fire. No spin-guard, retry budget, or timeout
was added (STOP-2).

## Wrote(0) — named, not changed

`write_once` still returns `Ok(WriteWait::Wrote(0))` when the Write CQE
is 0, and `send` does `written += 0` and resubmits. If a pipe write of
a non-empty buffer ever reports 0, that is an unguarded loop. POSIX
write of n>0 returning 0 is not expected on a pipe; nothing currently
stops it if it does. v2's blast is one match arm; this stays as a
named follow-up, not a silent spin-guard on this refute.

## THE FLOOR

```
Summary [ 500.563s] 5226 tests run: 5226 passed (7 slow), 22 skipped
```

0 FAIL, 0 TIMEOUT. Quiet box.

## BLAST

One match arm in `src/comms/process.rs` `submit_and_wait_eintr`, plus
the comment that now names the return as submitted-count. `write_once`'s
empty-drain continue is unchanged. Everything else in v1 stands.
