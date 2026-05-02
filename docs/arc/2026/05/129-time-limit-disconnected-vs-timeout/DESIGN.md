# Arc 129 — `:time-limit` distinguishes Timeout from Disconnected

**Status:** **shipped + closed 2026-05-01.** See INSCRIPTION.md
for the close-out (4 questions + verification + failure-engineering
record). DESIGN below is the as-drafted record kept verbatim.

## TL;DR

Arc 123's `:time-limit` wrapper has a Level-1 bug: it treats a
panicking inner thread as if it had timed out. When `:time-limit`
and `:should-panic` are combined on the same deftest, the
wrapper swallows the inner panic's message and emits a fake
"exceeded time-limit" message — so cargo libtest's
`#[should_panic(expected = "...")]` substring match fails on
panics that DID happen, with substrings that DID match,
because the wrapper rewrote the panic message before libtest
saw it.

Fix: split the recv_timeout error variants. On `Disconnected`
(the inner thread panicked and dropped the sender), join the
spawned thread's `JoinHandle` and re-raise the captured panic
via `std::panic::resume_unwind`. The original panic message
flows through unchanged; libtest sees the truth.

~15 LOC change in `crates/wat-macros/src/lib.rs`. Unblocks arc
126 slice 2 reland.

## Provenance

Surfaced 2026-05-01 by arc 126 slice 2's sonnet sweep
(`ac3c931ccd913ce24`). The sweep converted 6 deadlock-class
deftests from `:ignore` to `:should-panic("channel-pair-deadlock")`,
keeping `:time-limit "200ms"` as a defense-in-depth safety net.
After the conversion, all 6 tests failed with:

```
panic message: "deftest_..._test_step3_put_only: exceeded
                time-limit of 200ms (test thread leaked —
                process exit will reap)"
expected substring: "channel-pair-deadlock"
note: panic did not contain expected string
```

The arc-126 check was firing correctly — the substring
`channel-pair-deadlock` was visible in stderr. But cargo's
libtest `#[should_panic]` machinery never saw it. Sonnet
diagnosed the bug to a specific file:line:
`crates/wat-macros/src/lib.rs:660` and `:679`.

User direction: *"interesting i don't know if i understand the
issue - write the arc and i'll review - time to see if you can
educate me."*

This DESIGN is the educational artifact.

## The bug, walked through

The `:time-limit` wrapper is a proc-macro that emits this code
around each deftest's body when `:time-limit` is annotated:

```rust
// arc 123 — current shape (the buggy one)
let (__wat_tx, __wat_rx) = ::std::sync::mpsc::channel::<()>();

let _ = ::std::thread::spawn(move || {
    // ... loader setup ...
    ::wat::test_runner::run_single_deftest(
        ::std::path::Path::new(<path>),
        <name>, ...
    );
    let _ = __wat_tx.send(());
});

match __wat_rx.recv_timeout(::std::time::Duration::from_millis(<ms>)) {
    Ok(_) => {}
    Err(_) => panic!(<timeout_msg>),
}
```

Two things to notice:

- **Line 1 of spawn**: the JoinHandle returned by
  `::std::thread::spawn` is dropped (`let _ =`). We can never
  rejoin this thread.
- **Line 4 of match**: `Err(_)` is a catch-all. It matches every
  possible error variant of `recv_timeout` indiscriminately.

### Two scenarios

**Scenario A — happy path (no panic, no timeout):**

1. Spawned thread starts.
2. `run_single_deftest` runs to completion, returning normally.
3. The thread reaches `let _ = __wat_tx.send(());` and sends `()`
   on the channel. The send succeeds; the sender clone is now
   handed over to the channel buffer.
4. The thread function returns. Local bindings drop:
   `__wat_tx` (the original sender) is dropped.
5. Parent's `recv_timeout` receives the `()` we just sent →
   returns `Ok(())`.
6. `match` falls into the `Ok(_) => {}` arm. Test passes.

Clean.

**Scenario B — inner thread panics (e.g. arc-126 check fires):**

1. Spawned thread starts.
2. `run_single_deftest` calls into wat's check pass.
3. The check fires `ChannelPairDeadlock` with the substring
   `channel-pair-deadlock`. `run_single_deftest` panics with
   that substring as part of its panic message.
4. The panic begins unwinding the spawned thread's stack.
5. **As the unwind passes through the spawn's local scope, all
   bindings drop — INCLUDING `__wat_tx` (the sender).**
6. The thread terminates with a panic payload. Because we did
   `let _ =` on the spawn handle, this payload has nowhere to
   go — it's lost into the void of the dropped JoinHandle.
7. Parent's `recv_timeout` is still waiting. It notices the
   sender has been dropped (`recv_timeout` polls the channel
   state). With ALL senders dropped, it returns
   **`Err(RecvTimeoutError::Disconnected)` IMMEDIATELY** — not
   after the 200ms timeout elapses. The "Disconnected" variant
   means "no producer can ever send anything; stop waiting."
8. The wrapper's `Err(_) => panic!(<timeout_msg>)` arm matches.
9. The parent thread panics with the WRONG message — synthesized
   "exceeded time-limit" — even though no time elapsed (the
   tests fail in <10ms; the 200ms budget was untouched).
10. cargo libtest catches the parent's panic. It looks for the
    substring `channel-pair-deadlock`. It does not find it (the
    panic message is the synthesized timeout text). The
    `:should-panic` test reports as FAILED.

The inner panic's substring was correct. The outer wrapper
overwrote it.

## Why this is a bug

`recv_timeout` returns `Result<T, RecvTimeoutError>` where
`RecvTimeoutError` is an enum with TWO variants:

```rust
enum RecvTimeoutError {
    Timeout,        // The timeout duration elapsed.
    Disconnected,   // All senders dropped before sending.
}
```

These are SEMANTICALLY DIFFERENT outcomes:

- **Timeout**: the inner thread is taking too long. We don't
  know its state. We synthesize a panic message saying so. The
  inner thread leaks (we can't kill it).
- **Disconnected**: the inner thread is GONE. Either it
  completed without sending (which our spawn body shouldn't do)
  or it PANICKED. If it panicked, the panic is a real event —
  one we should surface, not paper over with a synthesized
  timeout message.

The current wrapper conflates them via `Err(_)`. Both cases
look identical from the outside; both produce the same fake
"timeout" panic. But only one IS a timeout.

This is a **Level 1 lie** in gaze terms: the wrapper says
"timeout" when the truth is "panic." The diagnostic
mis-describes reality.

## The fix

Two edits:

1. **Keep the JoinHandle.** Currently `let _ = ::std::thread::spawn(...)`
   discards it. We need it to capture the panic payload.
2. **Split `Err(_)` into `Err(Timeout)` and `Err(Disconnected)`.**
   On Timeout: keep current behavior (synthesized message + thread
   leak). On Disconnected: join the handle to get the panic
   payload, then re-raise via `std::panic::resume_unwind`.

```rust
// arc 129 — the fix
let (__wat_tx, __wat_rx) = ::std::sync::mpsc::channel::<()>();

let __wat_handle = ::std::thread::spawn(move || {
    // ... loader setup ...
    ::wat::test_runner::run_single_deftest(
        ::std::path::Path::new(<path>),
        <name>, ...
    );
    let _ = __wat_tx.send(());
});

match __wat_rx.recv_timeout(::std::time::Duration::from_millis(<ms>)) {
    Ok(_) => {}
    Err(::std::sync::mpsc::RecvTimeoutError::Timeout) => {
        // Real timeout — the inner thread is still running.
        // We can't safely kill it; process exit will reap it.
        // Synthesized message preserves arc-123's existing UX.
        panic!(<timeout_msg>);
    }
    Err(::std::sync::mpsc::RecvTimeoutError::Disconnected) => {
        // Inner thread terminated before sending. Either it
        // completed normally and the send failed silently
        // (rare; defensive case below), or it panicked and the
        // sender was dropped during unwind.
        match __wat_handle.join() {
            Ok(()) => {
                // Thread completed cleanly but somehow didn't
                // send. Defensive: treat as timeout.
                panic!(<timeout_msg>);
            }
            Err(payload) => {
                // Thread panicked. `payload` is the original
                // panic's `Box<dyn Any + Send>` carrying the
                // panic message. Re-raise with the SAME
                // payload so the parent's panic message IS
                // the original inner panic message — preserving
                // any substring (`channel-pair-deadlock`,
                // assertion failure text, etc.) that libtest
                // matches against.
                ::std::panic::resume_unwind(payload);
            }
        }
    }
}
```

`std::panic::resume_unwind(payload)` is the standard idiom for
"rethrow a panic that was captured from a thread." It takes the
`Box<dyn Any + Send>` returned by `JoinHandle::join().unwrap_err()`
and re-raises a panic with that exact payload. The Display impl
on the panic produces the original message verbatim.

After this fix, scenario B becomes:

1-7. Same as before.
8. The wrapper's `Err(Disconnected)` arm matches.
9. `__wat_handle.join()` blocks briefly (the thread has already
   terminated; join returns immediately) and returns
   `Err(payload)`.
10. `resume_unwind(payload)` re-raises the panic in the parent.
11. The parent panics with the original message — including
    `channel-pair-deadlock`.
12. cargo libtest catches the parent's panic. Substring match
    finds `channel-pair-deadlock`. Test reports as PASSED.

## What this fix preserves

- **Real timeouts still synthesize the timeout message.** A
  test that genuinely runs longer than the budget hits the
  `Timeout` arm; arc-123's existing UX is preserved verbatim.
- **The "test thread leaked" honesty stays.** Arc 123's
  acknowledgement that real timeouts leak the runaway thread
  is unchanged. The new `Disconnected` arm doesn't claim a
  thread leak (the thread already terminated).
- **Defense-in-depth still works.** Tests with both
  `:should-panic` AND `:time-limit` annotations now have BOTH
  guarantees: panic propagates correctly (substring match
  works) AND a time cap exists (real hangs get the synthesized
  message).
- **Arc 126 slice 2 unblocks.** The 6 deadlock-class tests
  pass via `:should-panic` matching once arc 129 ships.

## What this fix does NOT do

- Does NOT change the `:time-limit`-only path. Tests with
  `:time-limit` but no `:should-panic` behave identically.
- Does NOT change the no-`:time-limit` path. Tests without
  `:time-limit` skip the wrapper entirely (per arc 123's
  "ONLY deftests with a `:time-limit` get the wrapper" rule).
- Does NOT change `run_single_deftest`. Substrate behavior is
  unchanged.
- Does NOT introduce a new annotation or syntax. The fix is
  purely in the proc-macro emission.

## The four questions

**Obvious?** Yes once you see it. `recv_timeout` has two
variants. The current code matches both with `Err(_)`. That's
a pattern-match too broad — a Level 1 lie about what happened.
Splitting the variants and surfacing the actual cause is what
the type system is for.

**Simple?** Yes. ~15 LOC. Three edits in one function:
- Replace `let _ = ::std::thread::spawn` with `let __wat_handle =`
- Replace `Err(_) => panic!(...)` with two arms (`Err(Timeout)`,
  `Err(Disconnected)`)
- On Disconnected, join + resume_unwind

No new types, no new helpers, no new doctrine. Surgical at
the proc-macro layer.

**Honest?** Yes. The fix names the structural truth: Disconnected
means thread died; surface its actual panic, not a synthesized
timeout message. The diagnostic stops lying.

**Good UX?** Phenomenal. After the fix, `:should-panic` and
`:time-limit` compose as their docs imply they should. Future
authors don't have to know the bug exists. The substrate stops
silently corrupting the panic-substring chain.

## Implementation plan

### Slice 1 — the fix

`crates/wat-macros/src/lib.rs` — modify the `body` quote at
line 658 onwards (the `if let Some(ms) = site.time_limit_ms`
branch). Two edits:

1. Change `let _ = ::std::thread::spawn(...)` to
   `let __wat_handle = ::std::thread::spawn(...)`.
2. Change the `match __wat_rx.recv_timeout(...)` arms from
   `Err(_)` to the two-variant split shown in § "The fix".

### Slice 2 — verification

- New unit test in `crates/wat-macros/src/lib.rs` testing the
  proc macro's generated code directly is hard (proc-macro
  expansion is at compile time). Instead, **the verification IS
  arc 126 slice 2's reland**: after arc 129 lands, `cargo test
  --release --workspace` ships green with the 6 deadlock-class
  tests passing via `:should-panic` substring matching.
- Add or extend an end-to-end smoke test in
  `crates/wat-sqlite/wat-tests/arc-122-attributes.wat` (or a
  new arc-129-specific file): a deftest with both
  `:should-panic("expected substring")` AND `:time-limit
  "10ms"` AND a body that calls a wat form known to panic
  with that substring (e.g. an arc-126 anti-pattern). The test
  passes iff the substring propagates through the new wrapper.
- Confirm: `cargo test --release -p wat-sqlite -- arc_129` runs
  the new smoke test in <100ms (well under the 10ms budget? —
  see § "Caveat" below); test reports as `... ok`.

### Slice 3 — closure

- INSCRIPTION + cross-reference from arc 123's DESIGN noting the
  bug fix.
- 058 changelog row (lab repo).
- Update arc 126 slice 2 SCORE-doc to note the unblock.
- Optional: update arc 123's INSCRIPTION with a "post-arc-129
  amendment" pointer.

## Verification — slice 2 reland is the natural test

After arc 129 ships:

1. `cargo test --release --workspace` should produce:
   - 6 deadlock-class tests now report as `... ok` (the panic
     matched the `:should-panic` substring).
   - All other tests pass.
   - Workspace exit=0.
2. Specifically: `cargo test --release -p wat-holon-lru` shows
   `passed; 0 failed; 0 ignored` (or however many ignored
   remain from the wat-sqlite arc-122 mechanism test).

If the reland's 6 tests still fail after arc 129, this DESIGN
missed something — probably a layer between `resume_unwind` and
cargo libtest's panic-catching. Open arc 130 with the new
diagnostic.

## Caveat — `:time-limit` fast-fast tests

Arc 126's deadlock tests fail in <10ms (well under 200ms). If
arc 129's smoke test uses a `:time-limit "10ms"` budget, the
parent's `recv_timeout` might race the spawned thread's panic:
the timeout could fire before the panic-unwind completes,
giving a Timeout panic instead of Disconnected. Use a generous
budget (e.g. 500ms or 1s) for the smoke test so the
Disconnected path fires reliably.

This is an arc-129-specific test concern, not a substrate bug.

## Cross-references

- `docs/arc/2026/05/123-time-limit/DESIGN.md` — the arc that
  introduced the wrapper. Arc 129 is the bug fix.
- `docs/arc/2026/05/123-time-limit/INSCRIPTION.md` — closed-out
  status. Arc 129 amends.
- `docs/arc/2026/05/126-channel-pair-deadlock-prevention/SCORE-SLICE-2.md`
  — the score doc that surfaced this bug.
- `crates/wat-macros/src/lib.rs:658-680` — the lines that
  change.
- `std::sync::mpsc::RecvTimeoutError` — the two-variant enum
  that motivates the fix.
- `std::panic::resume_unwind` — the standard idiom for
  re-raising captured panic payloads from joined threads.

## Failure-engineering record

Arc 129 is the third substrate gap surfaced by arc 126's chain:

| Sweep | Slice | Hard rows | Substrate gap |
|---|---|---|---|
| 1 | slice 1 | 5/6 | arc 128 — sandbox-boundary guard |
| 2 | slice 1 reland | 14/14 | none (clean) |
| 3 | slice 2 | 6/8 | **arc 129 — Timeout vs Disconnected** |

Each non-clean sweep produced a precisely-diagnosed gap. Arc
123 had the bug latent for the same reason arc 117 had its
boundary bug latent: no test exercised the failure path
(no `:should-panic` test combined with `:time-limit` to surface
the panic-fast-disconnect race). Arc 126's slice 2 was the
first stress test of the combination; the bug surfaced
immediately.

This is the failure-engineering apparatus working as intended.
