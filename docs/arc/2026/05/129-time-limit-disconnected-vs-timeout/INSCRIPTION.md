# Arc 129 — INSCRIPTION

## Status

**Shipped + closed 2026-05-01.** Single substrate fix to
`crates/wat-macros/src/lib.rs:658-715`; ~25 LOC of pure code +
~14 LOC of load-bearing comments. Sonnet shipped slice 1 in 2.5
min on a 14-of-14 scorecard (8 hard + 6 soft); commit `406d124`.
Arc 126 slice 2 immediately landed atop it (commit `3ab8700`),
validating the fix end-to-end via 6 deadlock-class tests now
passing through `:should-panic`-substring matching.

## What this arc fixes

Arc 123's `:time-limit` wrapper conflated `RecvTimeoutError::Timeout`
with `RecvTimeoutError::Disconnected` via a catch-all `Err(_)`
arm. When the spawned deftest thread panicked fast (well under
the time budget), the panic-unwind dropped the mpsc sender;
`recv_timeout` returned `Disconnected` IMMEDIATELY (not after
the budget elapsed); the wrapper's `Err(_)` arm matched and
synthesized a fake `"exceeded time-limit"` panic — overwriting
the inner panic's message before cargo libtest could
substring-match on it.

`:should-panic` and `:time-limit` were silently
non-composable on the same deftest. Arc 129 makes them compose.

## The rule

> The wrapper around a `:time-limit`-annotated deftest MUST
> distinguish the two `RecvTimeoutError` variants. On
> `Timeout`: synthesize the existing timeout panic (real
> timeout; thread leaks per arc 123's existing UX). On
> `Disconnected`: join the spawned thread's `JoinHandle` and
> re-raise the captured panic via
> `std::panic::resume_unwind`. The original panic message
> flows through verbatim.

## The fix

Two edits in the proc macro's `body` quote:

1. **Keep the JoinHandle.** `let _ = ::std::thread::spawn(...)`
   becomes `let __wat_handle = ::std::thread::spawn(...)`.
2. **Split the recv_timeout match.** `Err(_) => panic!(...)`
   splits into:
   - `Err(::std::sync::mpsc::RecvTimeoutError::Timeout)` →
     synthesized timeout panic (unchanged behavior).
   - `Err(::std::sync::mpsc::RecvTimeoutError::Disconnected)` →
     `__wat_handle.join()`, on `Err(payload)` call
     `::std::panic::resume_unwind(payload)`. On `Ok(())` (rare
     defensive case: thread completed cleanly but somehow
     didn't send) fall back to the synthesized timeout panic.

`std::panic::resume_unwind(payload)` is the standard idiom for
"rethrow a panic captured from a thread." It takes the
`Box<dyn Any + Send>` returned by
`JoinHandle::join().unwrap_err()` and re-raises a panic with
that exact payload. The Display impl produces the original
message verbatim. Cargo libtest catches the parent's panic;
`#[should_panic(expected = "...")]` substring-matches on the
preserved message.

## Why a wrapper bug, not a runner bug

The substrate's `run_single_deftest` panics correctly with the
inner panic's full message. The bug was at the proc-macro
emission layer — specifically the wrapper that arc 123
introduced for `:time-limit` annotations. The wrapper sat
BETWEEN `run_single_deftest`'s panic and cargo libtest's
panic-catching, transforming the panic message in the process.

Arc 123's original Display message used `Err(_)` defensively.
The intent was correct ("on any wait failure, signal a
timeout"). The implementation was structurally wrong because
`recv_timeout`'s two variants are not interchangeable: Timeout
means "still running"; Disconnected means "already done"
(possibly via panic).

## What this arc closes

- The non-composability of `:should-panic` + `:time-limit` on
  a single deftest. Arc 122's `:should-panic` and arc 123's
  `:time-limit` now compose correctly.
- Arc 126 slice 2's blocking dependency. The 6 deadlock-class
  tests carrying both annotations now PASS via
  `:should-panic` substring matching, validating the arc 126
  channel-pair-deadlock check end-to-end at runtime.
- The latent "panic message gets eaten silently" failure mode
  in any future test that combines both annotations.

## Limitations

- **Tight time-limits could race.** If `:time-limit` is
  shorter than the panic-unwind takes (sub-millisecond
  budgets), the parent's `recv_timeout` might fire Timeout
  before the spawned thread's panic-unwind drops the sender,
  leading to a Timeout panic instead of the inner panic
  message. In practice the panic-fast path completes in
  single-digit ms; arc 126's 200ms budget provides ample
  headroom. Tests with sub-100ms budgets should validate
  empirically.
- **The `Disconnected → Ok(())` defensive case is rare.** It
  fires only if the spawned thread completes WITHOUT
  panicking AND fails to send (e.g. parent's recv was already
  dropped — impossible in our wrapper shape, but defensive).
  Treated as a synthesized timeout panic; preserves arc-123
  UX for the unreachable path.

## Verification — slice 2 of arc 126 IS the test

After arc 129 ships, arc 126 slice 2's six `:should-panic("channel-pair-deadlock")`
+ `:time-limit "200ms"` tests pass:

```
test deftest_wat_lru_test_cache_service_put_then_get_round_trip
  - should panic ... ok
test deftest_wat_tests_holon_lru_HologramCacheService_test_step3_put_only
  - should panic ... ok
test deftest_wat_tests_holon_lru_HologramCacheService_test_step4_put_get_roundtrip
  - should panic ... ok
test deftest_wat_tests_holon_lru_HologramCacheService_test_step5_multi_client_via_constructor
  - should panic ... ok
test deftest_wat_tests_holon_lru_HologramCacheService_test_step6_lru_eviction_via_service
  - should panic ... ok
test deftest_wat_tests_holon_lru_proofs_arc_119_step_B_single_put
  - should panic ... ok
```

Workspace test green: `cargo test --release --workspace` exit=0;
100 `test result: ok` lines; 0 failed; 1 ignored (the wat-sqlite
arc-122 mechanism test, intentional).

Per-test runtime: single-digit ms each (wat-holon-lru's 14 tests
aggregate 0.06s; wat-lru's 8 aggregate 0.03s) — well under the
200ms budget. No race condition observed.

## The four questions (final)

**Obvious?** Yes. `recv_timeout` returns `Result<T,
RecvTimeoutError>` where `RecvTimeoutError` has two variants.
Matching both with `Err(_)` is a Level 1 lie about what
happened. Splitting the variants is the structural truth.

**Simple?** Yes. ~25 LOC of pure code change in one proc-macro
function. No new types, no new doctrine, no new wat syntax.
Reuses the existing `JoinHandle` (just stops dropping it) and
the standard-library `std::panic::resume_unwind`.

**Honest?** Yes. The fix names the structural truth: Disconnected
means the thread is gone; surface its actual panic, not a
synthesized timeout message. The diagnostic stops lying.

**Good UX?** Phenomenal. After the fix, `:should-panic` and
`:time-limit` compose as their docs imply they should. Future
authors writing both annotations get correct behavior; they
don't have to know the bug existed.

## Failure-engineering record

Arc 129 was the second substrate-fix arc surfaced by arc 126's
chain:

| # | Sweep | Slice | Hard rows | Substrate gap |
|---|---|---|---|---|
| 1 | arc 126 slice 1 | first sweep | 5/6 | arc 128 (boundary guard) |
| 2 | arc 126 slice 1 | reland | 14/14 | none (clean) |
| 3 | arc 126 slice 2 | first sweep | 6/8 | **arc 129 (this)** |
| 4 | arc 129 slice 1 | first sweep | **14/14** | none (clean) |

Each non-clean sweep precisely diagnosed a substrate gap; each
follow-on arc landed cleanly. The artifacts-as-teaching
discipline carries across distinct substrate layers (arc 128 — a
check walker in `src/check.rs`; arc 129 — a proc macro in
`crates/wat-macros/src/lib.rs`). The sweep timings compound:

| Sweep | Wall-clock |
|---|---|
| arc 126 slice 1 first sweep | 13.5 min |
| arc 126 slice 1 reland | 7 min |
| arc 126 slice 2 first sweep | 5.3 min |
| arc 129 slice 1 first sweep | **2.5 min** |

The artifacts keep teaching after they're written.

## Cross-references

- `docs/arc/2026/05/123-time-limit/DESIGN.md` — the arc this
  amends. Arc 123's `Err(_)` fix-shape is now corrected by
  arc 129. The post-fix amendment is recorded here.
- `docs/arc/2026/05/123-time-limit/INSCRIPTION.md` — the
  closed-out status from 2026-05-01. Arc 129 amends this
  retroactively.
- `docs/arc/2026/05/126-channel-pair-deadlock-prevention/SCORE-SLICE-2.md`
  — the score doc where this bug was first surfaced and
  diagnosed to file:line.
- `docs/arc/2026/05/126-channel-pair-deadlock-prevention/REALIZATIONS.md`
  — the discipline (failure engineering + artifacts-as-teaching)
  that produced this fix.
- `crates/wat-macros/src/lib.rs:658-715` — the modified function
  body quote.
- `std::sync::mpsc::RecvTimeoutError` — the two-variant enum.
- `std::panic::resume_unwind` — the standard idiom for
  re-raising captured panics.

## Queued follow-ups

None. The fix is complete; the verification (arc 126 slice 2)
is also complete. Future tests with `:should-panic` +
`:time-limit` combinations inherit the correct behavior
automatically.
