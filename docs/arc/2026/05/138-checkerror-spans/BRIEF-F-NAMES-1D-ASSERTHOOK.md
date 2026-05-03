# Arc 138 F-NAMES-1d-asserthook — Sonnet Brief: assertion-hook shows real thread name

**Goal:** the wat assertion-failure panic hook (`write_assertion_failure` in src/panic_hook.rs) currently renders `thread '<unnamed>' panicked at...` even when the panicking thread IS named (e.g., wat::test! deftest workers got real names from F-NAMES-1c). Diagnose why `std::thread::current().name()` returns None in this hook context and fix it.

**Working directory:** `/home/watmin/work/holon/wat-rs/`.

**Driver:** the user's UX-visible `<unnamed>` complaint. F-NAMES-1c half-fixed it (Rust-default panic path); F-NAMES-1d-asserthook closes the half users actually see most often (every wat assert-eq failure goes through the assertion hook).

## Read in order

1. `docs/arc/2026/05/138-checkerror-spans/SCORE-F-NAMES-1C.md` — the predecessor that surfaced this gap.
2. `docs/arc/2026/05/138-checkerror-spans/NAMES-AUDIT.md` — naming charter.
3. `src/panic_hook.rs` — full file (~250 lines).
4. `src/assertion.rs` lines 40-100 — AssertionPayload struct definition.

## What to investigate

`std::thread::current().name()` should return the panicking thread's name when called from inside `std::panic::set_hook`'s closure. F-NAMES-1c confirmed wat::test! deftest workers ARE named (`Thread::Builder::new().name(format!("wat-test::{}", deftest_name)).spawn(...)`), but `<unnamed>` STILL renders in assertion-hook output.

Possible causes:
1. The hook closure runs on a thread different from the panicking one (unlikely — Rust docs say hooks run on the panicking thread).
2. `panic::resume_unwind(payload)` from the wat::test! parent re-panics on the parent thread; the parent's name lookup behaves differently.
3. Some interaction between `catch_unwind` and `set_hook` losing context.
4. The assertion hook is invoked TWICE — once on the worker (named, but rendered output gets buffered/discarded?) and once on the parent (maybe unnamed in some cases?).

## Proposed fix path (likely)

Rather than rely on `thread::current().name()` at hook time, capture the thread name AT PANIC SITE (where AssertionPayload is constructed in src/assertion.rs). Add a `thread_name: Option<String>` field to AssertionPayload. The construct site captures the name via `std::thread::current().name().map(String::from)`, the payload travels through panic-and-resume-unwind cleanly, and `write_assertion_failure` reads `payload.thread_name` instead of doing a fresh lookup.

This mirrors how AssertionPayload already carries `location`, `message`, `actual`, `expected`, `frames` — all captured at panic site, surviving through the unwind. Adding `thread_name` is the same pattern.

## Investigation tasks

1. **Verify the failure**: write a minimal test that spawns a NAMED thread, panics inside via `panic!("test")`, and check if a custom panic hook sees the name via `thread::current().name()`. If YES → the issue is somewhere else (resume_unwind, etc.). If NO → there's a Rust quirk to work around.

2. **Check the resume_unwind path**: in crates/wat-macros/src/lib.rs:712 the parent calls `resume_unwind(payload)`. After this re-panic, what does the parent's panic hook see for thread::current()?

3. **Implement the fix**: based on diagnosis, either:
   - Capture thread_name at AssertionPayload construction (most likely fix)
   - OR fix the hook context if there's a known Rust quirk
   - OR document why `<unnamed>` is unavoidable (last resort)

## What to do

Based on the investigation above, implement the diagnosed fix. Most likely:
1. Add `pub thread_name: Option<String>` field to `AssertionPayload` (src/assertion.rs).
2. At every AssertionPayload construction site, set `thread_name: std::thread::current().name().map(String::from)`.
3. Update `write_assertion_failure` (src/panic_hook.rs:103) to use `payload.thread_name.as_deref().unwrap_or("<unnamed>")` instead of `thread::current().name().unwrap_or("<unnamed>")`.

After fix, verify spot-check shows `thread 'wat-test::<deftest_name>' panicked at...` in assertion-hook output.

## Constraints

- Files modified: src/panic_hook.rs + src/assertion.rs (+ any AssertionPayload construction sites). Estimated 2-4 files.
- All 7 arc138 canaries continue to pass.
- Workspace tests pass: `cargo test --release --workspace 2>&1 | grep FAILED | grep -v trading` returns empty.
- NO commits, NO pushes.

## Reporting back

Compact (~300 words):

1. Diff stat.
2. Diagnosis — what was the actual cause of `<unnamed>`? Was it resume_unwind, hook context, or something else?
3. Fix shape — payload field added, construction sites updated, hook reads from payload.
4. Verification — spot-check shows `wat-test::<name>` in assertion-hook output.
5. Honest deltas — anything unexpected.
6. Four questions briefly.

## Why this matters

The user's actual UX-visible `<unnamed>` complaint comes from this hook path. F-NAMES-1d-asserthook closes the user-facing crack. After this ships, every wat assertion failure panics with a fully navigable thread name + file:line:col coordinates.
