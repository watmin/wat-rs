# Arc 138 F-NAMES-1c — SCORE (PARTIAL — assertion hook gap surfaced)

**Written:** 2026-05-03 AFTER sonnet's report.
**Agent ID:** `a73dd85d92765effd`
**Runtime:** ~9 min (566 s).

## Verification

| Claim | Disk-verified |
|---|---|
| Files modified | 1 (crates/wat-macros/src/lib.rs) ✓ |
| diff stat 4+/2- | ✓ |
| Thread::Builder::new().name(format!("wat-test::{}", deftest_name)) emit | ✓ at lines 672-690 |
| Rust-default panic hook path shows real name | ✓ (sonnet spot-check confirmed `thread 'wat-test:::wat-tests::sqlite::arc-122::test-arc-122-should-panic'`) |
| All 7 arc138 canaries | 7/7 PASS ✓ |
| Workspace tests | empty FAILED ✓ |

## Substrate observation — TWO panic-display paths, only ONE fixed

Sonnet's spot-check revealed the actual UX surfaces in TWO places:

1. **Rust-default panic hook path** (test_runner.rs:487 area, the `RUST_BACKTRACE=1` standard hook): `std::thread::current().name()` returns the Builder name. **FIXED by F-NAMES-1c.** Now reads `thread 'wat-test::<deftest>' panicked at...`.

2. **Custom assertion-failure hook** (src/panic_hook.rs:103-120, `write_assertion_failure` function): `std::thread::current().name()` returns None in this hook's execution context — so the rendered output STILL shows `<unnamed>`. **NOT FIXED by F-NAMES-1c.** Pre-existing gap that F-NAMES-1c didn't touch.

This means the user's original observation:
```
thread '<unnamed>' panicked at <test>:10:19:
assert-eq failed
```
is rendered by `write_assertion_failure` — the assertion hook path. Even after F-NAMES-1c, that exact rendering still shows `<unnamed>` because the gap is in `write_assertion_failure`, not the bare-thread-spawn site we fixed.

## New crack to add to NAMES-AUDIT — F-NAMES-1d-asserthook

The `write_assertion_failure` at src/panic_hook.rs:103 uses `std::thread::current().name()` which returns None inside this hook context. Investigation needed:
- Why does std::thread::current().name() return None here when it returns the Builder-set name in the Rust-standard hook?
- Possibility: the assertion hook runs BEFORE the worker's name is fully set, OR runs on a different thread than expected, OR there's a panic-hook-context quirk we don't yet understand.
- Fix: either pass the deftest name through some side channel (thread-local, payload field), or diagnose why thread::current().name() is empty here.

This is the user's actually-visible UX issue. F-NAMES-1c fixed half; F-NAMES-1d-asserthook fixes the half that the user actually sees most often.

## Calibration

Predicted 5-10 min; actual 9 min. In-band for the ONE fix sonnet was scoped to. The substrate observation about the second hook path is honest delta — sonnet correctly identified the gap rather than papering over it.

## Hard scorecard

| # | Criterion | Result |
|---|---|---|
| 1 | File scope (only wat-macros/src/lib.rs) | **PASS** |
| 2 | Thread::Builder + .name() emit | **PASS** |
| 3 | Rust-default hook shows name | **PASS** |
| 4 | Workspace tests + canaries | **PASS** |
| 5 | No commits | **PASS** |

**HARD: 5/5 PASS.** F-NAMES-1c IS done per its scope.

## Soft observation

The user-visible `<unnamed>` complaint is NOT fully resolved — the assertion-hook path is the more common panic surface and still leaks. Per no-deferrals doctrine, F-NAMES-1d-asserthook gets attacked NEXT.

## Ship decision

**SHIP partial.** F-NAMES-1c closes the Builder-spawn gap. F-NAMES-1d-asserthook (new crack) gets queued immediately — same engagement should fix it before slice 6 closure.

## Next

F-NAMES-1d-asserthook — diagnose + fix the assertion hook's `<unnamed>` rendering. Investigation pass (~5 min) then fix (~10 min).
