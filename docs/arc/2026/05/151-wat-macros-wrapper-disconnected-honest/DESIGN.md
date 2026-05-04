# Arc 151 — wat-macros wrapper: honest Disconnected-with-clean-thread message

**Status:** SCRATCH — small foundation crack surfaced 2026-05-03;
not yet implemented; spawns when the queue clears.

User direction 2026-05-03 (mid-arc-148-slice-5 investigation):

> *"we mask it with a constant string....
> > panic!(#timeout_msg);"*

## The lie

`crates/wat-macros/src/lib.rs:698-729` is the per-deftest
thread-spawn + recv_timeout wrapper (arc 132 belt-and-suspenders
default; arc 129's split-arms for Timeout vs Disconnected).

The wrapper has TWO panic sites that share the SAME message:

```rust
match __wat_rx.recv_timeout(...) {
    Ok(_) => {}                                        // test passed
    Err(RecvTimeoutError::Timeout) => {                // genuine timeout
        panic!(#timeout_msg);                           //   "exceeded time-limit"
    }
    Err(RecvTimeoutError::Disconnected) => {           // thread terminated early
        match __wat_handle.join() {
            Ok(()) => {                                 // ← the lie
                panic!(#timeout_msg);                   //   reports "exceeded time-limit"
                                                        //   when actual: thread completed
                                                        //   without signalling
            }
            Err(payload) => {
                resume_unwind(payload);                 // ← honest: re-raises real panic
            }
        }
    }
}
```

The defensive `Ok(())` arm fires when:
1. recv_timeout returned Disconnected (sender dropped)
2. join() returned Ok(()) (thread completed without panicking)

Per the comment in code: *"Inner thread completed cleanly but didn't
send. Defensive: treat as timeout (rare; defensive case below)."*

But "treat as timeout" is dishonest. The thread DID NOT exceed any
time limit — it completed, just didn't signal. Different failure
mode; different diagnostic; the wrapper conflates them by reusing
the timeout message.

## Why this is a foundation crack

Per the user's "eliminate failure domains; don't bridge" discipline:
if this branch ever fires in practice, real bugs get masked as
timeouts. The substrate's diagnostic surface lies. Tools and humans
see "exceeded time-limit of 200ms" and chase a perf issue when the
actual issue is "a substrate path completed without signalling."

The wrapper IS otherwise correct (the Disconnected → Err(payload) →
resume_unwind path correctly preserves real panic messages). This is
the ONE leaky case.

## What ships

### Single substrate edit

`crates/wat-macros/src/lib.rs:722-725`:

```rust
// BEFORE:
Ok(()) => {
    panic!(#timeout_msg);
}

// AFTER (sketch):
Ok(()) => {
    // Inner thread completed without signalling — neither a
    // timeout nor a panic. This is a substrate inconsistency
    // (the test body's `__wat_tx.send(())` was reached but the
    // value didn't propagate; possible causes include a
    // poisoned channel, a substrate-level race, or a code path
    // that panicked + caught the panic before exiting the
    // closure cleanly). Surface honestly so future-us can find
    // the cause.
    panic!(
        "{}: thread completed but did not signal — likely a \
         wat-macros wrapper inconsistency or substrate-level \
         race; investigate (deftest {} at {}:{}:{})",
        fn_name, deftest_name, file_path_str, line, col,
    );
}
```

The new message:
- States WHAT happened (thread completed but didn't signal)
- States it's NOT a timeout
- Names possible causes for investigation
- Carries the navigable coordinates (already in scope as
  fn_name, deftest_name, file_path_str, line, col)

### Optional follow-on

Once the honest message is in place, monitor whether the branch
fires in practice. If it ever fires:
- Surface the actual call site
- Investigate the root cause (substrate race; channel state; etc.)
- Either fix the root cause OR document why the defensive case is
  reachable

If it NEVER fires (the comment may be wrong about "rare" — it
might actually be unreachable), consider replacing the panic with
`unreachable!("...")` to encode the invariant.

## What this slice does NOT do

- NO change to the Timeout arm (`exceeded time-limit` message stays
  for genuine timeouts)
- NO change to the Disconnected → resume_unwind arm (correct as-is)
- NO scope expansion into investigating WHY the case might fire
  (that's a follow-on if it ever surfaces)
- NO `wat::test::time-limit` user-facing API change

## Slice plan

### Single slice

Tiny scope (~10 LOC + maybe 1 test that exercises the contrived
path). One commit; one PR-equivalent push.

Predicted ~15-30 min Mode A. Time-box 45 min.

## Cross-references

- arc 129 — time-limit wrapper distinguishes Timeout vs Disconnected
  (the wrapper's original split-arm design)
- arc 132 — default 200ms time-limit on every deftest (the wrapper's
  default)
- arc 132 amend (commit 0a8d6e5) — raised default to 1000ms (different
  fix; same wrapper file)
- COMPACTION-AMNESIA-RECOVERY § 12 — foundation discipline (eliminate
  failure domains; don't bridge)

## Status notes

- DESIGN drafted 2026-05-03 as scratch.
- NOT blocking arc 148 / arc 150 / arc 109 closure.
- Spawn when the queue clears around arc 148 + arc 146 + arc 144 +
  arc 130 + arc 145 + arc 147 + arc 141. Probably late in the arc 109
  wind-down sequence.
- This is a small honest fix, not a feature.
