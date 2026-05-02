# Arc 132 Slice 1 — Sonnet Brief

**Goal:** make every `:wat::test::deftest` get a 200ms default
time-limit wrapper. Explicit `:wat::test::time-limit` annotations
override the default per-test. The "no wrapper" code path
retires.

**Working directory:** `/home/watmin/work/holon/wat-rs/`.

## Read-in-order anchors

1. `docs/arc/2026/05/132-deftest-default-time-limit/DESIGN.md`
   — the rule + 200ms rationale + four questions.
2. `docs/arc/2026/05/123-time-limit/INSCRIPTION.md` — the
   wrapper this arc default-on's.
3. `docs/arc/2026/05/129-time-limit-disconnected-vs-timeout/INSCRIPTION.md`
   — the panic-propagation fix that made the wrapper safe to
   default-on.
4. `crates/wat-macros/src/lib.rs:653-700` — the function whose
   if-else collapses.

## What changes

ONE file: `crates/wat-macros/src/lib.rs`.

The `if let Some(ms) = site.time_limit_ms { ... } else { ... }`
branch collapses to:

```rust
const DEFAULT_TIME_LIMIT_MS: u64 = 200;
let ms = site.time_limit_ms.unwrap_or(DEFAULT_TIME_LIMIT_MS);
let timeout_msg = format!(
    "{}: exceeded time-limit of {}ms (test thread leaked — process exit will reap)",
    fn_name, ms,
);
let body = quote! {
    let (__wat_tx, __wat_rx) = ::std::sync::mpsc::channel::<()>();
    let __wat_handle = ::std::thread::spawn(move || {
        // ... existing thread body verbatim ...
    });
    match __wat_rx.recv_timeout(::std::time::Duration::from_millis(#ms)) {
        Ok(_) => {}
        Err(::std::sync::mpsc::RecvTimeoutError::Timeout) => panic!(#timeout_msg),
        Err(::std::sync::mpsc::RecvTimeoutError::Disconnected) => {
            match __wat_handle.join() {
                Ok(()) => panic!(#timeout_msg),
                Err(payload) => ::std::panic::resume_unwind(payload),
            }
        }
    }
};
```

The `else` branch (the no-wrapper direct call) retires
entirely. `body` is computed once with the unified shape.

Update the comment block above the body emission to reflect
the new universal-wrapper semantic.

## Constraints

- ONE file changes: `crates/wat-macros/src/lib.rs`. No `.wat`
  files. No other Rust files. No documentation. No commits.
- The `DEFAULT_TIME_LIMIT_MS` MUST be `200`.
- The wrapper code path must be IDENTICAL to today's
  with-`:time-limit` shape (preserves arc 129's
  Disconnected-via-resume_unwind discipline). Only the
  branching collapses.
- ~5-15 LOC change. Mostly deletion (the else branch goes
  away) + a const + an `unwrap_or`. >30 LOC = re-evaluate.

## Expected workspace impact

`cargo test --release --workspace` may surface new timeouts
on tests that genuinely take >200ms. Expected impact:

- Most tests run in single-digit ms — no impact.
- Hermetic-fork tests (run-sandboxed-hermetic-ast) take more
  than direct calls due to fork() overhead but typically
  10-50ms — no impact.
- A handful of integration-shaped tests may need explicit
  `:time-limit "<longer>"` annotations.

If the workspace test fires more than ~5 timeouts, surface in
the report and STOP — slice 2 is needed BEFORE this can
ship cleanly. If 0-5 timeouts, mark them with explicit
annotations in this slice (5 wat-test edits is acceptable
slice-1 scope).

## What success looks like

1. `crates/wat-macros/src/lib.rs` modified: `DEFAULT_TIME_LIMIT_MS`
   const added; if-else collapsed; else branch retired.
2. `cargo test --release -p wat --lib` exit=0 (proc-macro
   self-tests).
3. `cargo test --release --workspace` exit=0 — workspace
   tests that need >200ms have explicit annotations added (if
   5 or fewer); otherwise STOP.
4. The arc-130 LRU test (which has explicit
   `:time-limit "200ms"` already) continues working.
5. No commits.

## Reporting back

~150 words:

1. File:line refs for the `DEFAULT_TIME_LIMIT_MS` const + the
   collapsed body emission.
2. The exact final form of the wrapper emission (post-collapse).
3. Number of workspace tests that needed explicit
   `:time-limit "<longer>"` added (if any). Their names + the
   chosen budget. If you added any, surface the file paths.
4. Workspace test totals (passed / failed / ignored).
5. Honest deltas — anything you needed to invent.

## Why this is a quick arc

The wrapper code already exists (arc 123 + arc 129). The
change is mostly DELETION (collapsing the if-else). Adding the
const + flipping `Option::unwrap_or` is ~3 LOC. The risk is
workspace test fallout (any test that legitimately takes
>200ms). Stop-and-report on >5 timeouts is the calibration
guard.

Begin by reading DESIGN, then arc 123 + arc 129 INSCRIPTIONs.
Then make the change. Then verify with workspace test. Then
report.
