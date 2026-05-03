# Arc 138 F-NAMES-1d-asserthook — SCORE

**Written:** 2026-05-03 AFTER sonnet's report + orchestrator spot-check.
**Agent ID:** `af2a0aaf3778df919`
**Runtime:** ~6.5 min (394 s).

## Verification

| Claim | Disk-verified |
|---|---|
| Files modified | 3 (src/assertion.rs, src/runtime.rs, src/panic_hook.rs) ✓ |
| diff stat 83+/2- | ✓ |
| `pub thread_name: Option<String>` field added to AssertionPayload | ✓ |
| 3 construction sites capture `thread::current().name().map(String::from)` | ✓ (assertion.rs:145, runtime.rs:6943, runtime.rs:7009) |
| `write_assertion_failure` reads `payload.thread_name.as_deref().unwrap_or("<unnamed>")` instead of fresh thread::current() | ✓ |
| 2 new unit tests (renders_thread_name_from_payload_field, renders_unnamed_when_thread_name_field_is_none) | ✓ both PASS |
| All 7 arc138 canaries | 7/7 PASS ✓ |
| Workspace tests | empty FAILED ✓ |

## Hard scorecard: 6/6 PASS. Mechanism verified.

## Substrate observation — UPSTREAM gap surfaces during spot-check

The mechanism F-NAMES-1d-asserthook implements is correct: payload.thread_name is captured at construction, written through resume_unwind, read by the hook. The unit tests verify this end-to-end.

However, the user-visible `RUST_BACKTRACE=1 cargo test` spot-check still shows `thread '<unnamed>' panicked at <wat-file>:<L>:<C>:` for wat::test! deftest assertion failures.

**Diagnosis:** the wat assertions in those tests are firing on UNNAMED sub-threads spawned by `:wat::kernel::spawn` (and similar wat-side spawn primitives), NOT on the wat::test! deftest worker (which IS named after F-NAMES-1c). thread::current().name() at the AssertionPayload construction site returns None on these sub-threads.

Three unnamed `thread::spawn` sites in the substrate:
- src/spawn.rs:183 — `:wat::kernel::spawn` thread/process worker
- src/runtime.rs:12421 — Thread<I,O> spawn primitive worker
- src/runtime.rs:18780 — service spawn worker

The fix path: name these threads (e.g., `wat-thread::<derived-name>` from the lambda's name or call-site span). New crack F-NAMES-1e queued in NAMES-AUDIT.

## Substrate observation — F-NAMES-1d-asserthook closes its scope cleanly

The mechanism works. When the AssertionPayload IS constructed on a named thread (e.g., a Rust unit test directly invoking eval_expr), the hook output now reads the captured name. The remaining `<unnamed>` cases reflect the upstream-thread-naming gap, not a bug in F-NAMES-1d-asserthook.

## Calibration

Predicted 10-15 min for diagnosis + fix; actual 6.5 min. Sonnet's diagnosis was on-track (resume_unwind path); the deeper sub-thread gap surfaced during orchestrator spot-check.

## Ship decision

**SHIP.** F-NAMES-1d-asserthook closes the assertion-hook payload mechanism. The remaining `<unnamed>` UX leaks are upstream wat-spawn sites — F-NAMES-1e attacks them next.

## Next

**F-NAMES-1e** — name wat-side spawned threads. 3 sites (src/spawn.rs, src/runtime.rs ×2). Each thread should get a meaningful name (e.g., from lambda name or call-site span). Single slice, ~10-15 min sonnet.
