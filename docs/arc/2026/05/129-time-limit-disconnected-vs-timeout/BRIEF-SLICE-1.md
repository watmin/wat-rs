# Arc 129 Slice 1 — Sonnet Brief

**Goal:** fix the arc-123 `:time-limit` wrapper at
`crates/wat-macros/src/lib.rs:658-680` so panics from the
spawned thread propagate to cargo libtest with their original
panic message intact (specifically: the substring that
`#[should_panic(expected = "...")]` matches against).

**Working directory:** `/home/watmin/work/holon/wat-rs/`.

## Read-in-order anchor docs

1. `docs/arc/2026/05/129-time-limit-disconnected-vs-timeout/DESIGN.md`
   — the bug, the two scenarios, the fix shape. The TL;DR
   captures the substance; the scenario walkthrough explains
   WHY each line of the fix is the way it is. Read this fully
   before touching code.
2. `docs/arc/2026/05/123-time-limit/DESIGN.md` — the arc that
   introduced the wrapper. Section "What gets wrapped" explains
   why ONLY deftests with `:time-limit` get the wrapper.
3. `docs/arc/2026/05/126-channel-pair-deadlock-prevention/SCORE-SLICE-2.md`
   — the slice 2 score that surfaced this bug. Names sonnet's
   diagnosis (the file:line that needs changing).
4. `crates/wat-macros/src/lib.rs:658-700` — the body-quote
   block you are modifying.

## What changes

ONE file: `crates/wat-macros/src/lib.rs`. Only the
`if let Some(ms) = site.time_limit_ms { ... }` branch's `body`
quote.

Two edits:

### Edit 1 — keep the JoinHandle

Replace:
```rust
let _ = ::std::thread::spawn(move || {
    // ... loader setup + run_single_deftest + send ...
});
```

With:
```rust
let __wat_handle = ::std::thread::spawn(move || {
    // ... loader setup + run_single_deftest + send ...
});
```

(The body of the spawned thread is unchanged — keep
loader-setup, `run_single_deftest`, `let _ = __wat_tx.send(());`
verbatim.)

### Edit 2 — split the recv_timeout match

Replace:
```rust
match __wat_rx.recv_timeout(::std::time::Duration::from_millis(#ms)) {
    Ok(_) => {}
    Err(_) => panic!(#timeout_msg),
}
```

With:
```rust
match __wat_rx.recv_timeout(::std::time::Duration::from_millis(#ms)) {
    Ok(_) => {}
    Err(::std::sync::mpsc::RecvTimeoutError::Timeout) => {
        // Real timeout: inner thread is still running. We can't
        // safely kill a Rust thread from outside; the runaway
        // worker leaks until process exit. Synthesized message
        // preserves arc-123's existing UX.
        panic!(#timeout_msg);
    }
    Err(::std::sync::mpsc::RecvTimeoutError::Disconnected) => {
        // Inner thread terminated before sending. Either it
        // completed normally and the send failed silently
        // (rare; defensive case below), or it panicked and the
        // sender was dropped during unwind. Join the handle to
        // capture the panic payload, then re-raise so the
        // parent's panic message IS the inner panic's message
        // verbatim — preserving any substring (assertion text,
        // arc-126's `channel-pair-deadlock`, etc.) that
        // `#[should_panic(expected = "...")]` matches against.
        match __wat_handle.join() {
            Ok(()) => {
                // Thread completed cleanly but didn't send.
                // Defensive: treat as timeout.
                panic!(#timeout_msg);
            }
            Err(payload) => {
                ::std::panic::resume_unwind(payload);
            }
        }
    }
}
```

The comments are LOAD-BEARING — they explain WHY the new shape
is correct. Keep them verbatim or improve them; do not delete.

## Constraints

- ONE file changes: `crates/wat-macros/src/lib.rs`. No `.wat`
  files. No documentation. No commits.
- The `else` branch of `if let Some(ms) = ...` (the
  no-`:time-limit` case at lines 682-699) is unchanged.
- The Display message text in `timeout_msg` is unchanged.
- No new public API. No new types. No new helpers in lib.rs.
- ~20-30 LOC change. >50 LOC = stop and report.

## What success looks like

After your changes, **slice 2's working-tree wat-test edits
become the natural verification**. The 6 deadlock-class tests
in slice 2 (currently uncommitted in the working tree) carry
both `:should-panic("channel-pair-deadlock")` AND `:time-limit
"200ms"`. Pre-arc-129, they fail with "panic did not contain
expected string." Post-arc-129, the substring propagates
correctly and they PASS.

Verify with:

```bash
cargo test --release --workspace 2>&1 | tail -30
```

Expected: exit=0; 6 previously-failing tests now report `... ok`;
1 ignored test stays ignored (the wat-sqlite arc-122 mechanism
test); workspace ships green.

If your fix is correct, the 6 tests pass within 10ms each (the
inner panic fires fast; the new Disconnected handler joins the
already-terminated thread and re-raises the panic). Total
workspace test time should be sub-second per crate.

## Honest unknown — race condition for tight time-limits

The DESIGN's § "Caveat" notes a race: if the time-limit budget
is shorter than the panic-unwind takes, the parent's
`recv_timeout` could hit Timeout before the spawned thread's
panic completes its unwind. Slice 2's tests use 200ms; the
panic completes in <10ms; so the race doesn't fire in practice.

You don't need to solve this race in slice 1. Just verify it
doesn't fire for slice 2's tests. If it DOES fire (e.g.
flaky test failures with one Timeout panic and five
Disconnected panics), report it; we'll open arc 130.

## Reporting back

Target ~150 words:

1. File:line refs for the two edits.
2. The exact final form of the match block (so the orchestrator
   can verify shape).
3. Workspace test totals (passed / failed / ignored).
4. **Confirmation that slice 2's 6 tests now PASS via
   :should-panic.** Specifically: list the 6 test names and
   their post-arc-129 status.
5. Per-test runtime data (how fast does each panic-and-match
   complete? — confirms no race).
6. Any honest delta — if you needed `unsafe`, additional
   imports, or a non-trivial change beyond what the DESIGN
   spelled out, surface it.

## What this brief is testing (meta)

Per `REALIZATIONS.md`, the artifacts-as-teaching discipline
says: each delegation measures whether the artifacts teach. This
brief + DESIGN should be enough for sonnet to ship arc 129
without any conversation context. The fix is well-scoped,
file:line-precise, and the DESIGN walks through the bug
visually. If sonnet ships clean, the discipline is intact for
both substrate-bug-fix arcs (this) and structural-rule arcs
(arc 126 + arc 128).

The verification (slice 2's tests passing) is the end-to-end
proof that arc 129's fix is correct.

Begin by reading the DESIGN. Then make the two edits. Then
verify with the workspace test. Then report.
