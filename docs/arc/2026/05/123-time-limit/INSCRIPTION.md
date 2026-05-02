# Arc 123 — INSCRIPTION

## Status

**Shipped + closed 2026-05-01.** Same evening as arcs 121 +
122. The runtime safety net for hung deftests.

**Post-fix amendment 2026-05-01:** the original `recv_timeout`
`Err(_)` arm conflated `Timeout` with `Disconnected`, eating
panic substrings from the spawned thread when `:should-panic`
was also annotated. Arc 129 split the match arms and added
`JoinHandle::join()` + `std::panic::resume_unwind` on
Disconnected. See arc 129's INSCRIPTION.

**Generalization 2026-05-01:** arc 132 made every deftest pass
through the wrapper with a 200ms default. The `:time-limit`
annotation is now an OVERRIDE for tests genuinely needing more
budget (e.g. sqlite I/O). See arc 132's SCORE-SLICE-1.md.

## What this arc closes

Pre-arc-123, a hung deftest hung indefinitely. Arc 121 + 122
isolated the hang to one `#[test] fn` (cargo's filter could
skip past it), but `cargo test --include-ignored` or a fresh
agent re-running everything still saw the hang.

User direction:

> the test func we build in rust... at macro time we wrap it
> in a time block and panic out of we hit it...
>
> we should advertise millisecond resolution timeout.. users
> can use minutes and seconds.. but we don't actively
> demonstrate it.. it supported.. not suggested..

The doctrine — millisecond first; tests should take ms; s/m
suffixes exist for exceptional cases. Resolution finer than ms
not supported.

## What shipped

**Two file changes** in `crates/wat-macros/`:

- `discover.rs` — `parse_duration_ms("500ms" | "5s" | "1m")
  -> Result<u64, String>`. Scanner gained
  `pending_time_limit_ms: Option<u64>` field; new annotation
  arm `(:wat::test::time-limit "<duration>")` attaches the
  parsed milliseconds to the next deftest. 11 unit tests
  covering parse / state-attachment / annotation stacking /
  unrelated-form clearing.
- `lib.rs` — codegen emits a wrapper that spawns the deftest
  body on a fresh `std::thread`, calls `recv_timeout(ms)` on
  the result channel, and panics with a clear timeout message
  if the budget expires. The original `:should-panic` /
  `#[ignore]` attributes still work — they ride atop the
  wrapper.

End-to-end verification —
`crates/wat-sqlite/wat-tests/arc-123-time-limit.wat`:

```
test deftest_wat_tests_sqlite_arc_123_test_arc_123_fast            ... ok
test deftest_wat_tests_sqlite_arc_123_test_arc_123_seconds_suffix  ... ok
test deftest_wat_tests_sqlite_arc_123_test_arc_123_minutes_suffix  ... ok
test result: ok. 3 passed; 0 failed; 0 ignored
```

Three deftests with `100ms`, `5s`, `1m` budgets; all complete
trivially under-budget.

## What got surfaced

**Arc 129 — Timeout vs Disconnected conflation.** When the
inner thread panicked from `assert-eq` failing, the channel
disconnected before the timeout fired. The `Err(_)` arm
panicked with the timeout message, swallowing the inner panic
text. Pattern:

```rust
match result_rx.recv_timeout(Duration::from_millis(ms)) {
    Ok(_) => (),
    Err(_) => panic!("test exceeded {}ms time limit", ms),  // BUG
}
```

`:should-panic("inner panic substring")` annotations matched
against the OUTER timeout panic, not the inner test panic;
substring mismatch; test failed unexpectedly.

The fix: split the arms; on Disconnected, join the inner
thread's `JoinHandle` and `panic::resume_unwind(payload)` so
the inner panic message survives verbatim. Surfaced by arc 126
slice 2 sweep at 6/8 hard rows; arc 129 closed the gap; arc
126 slice 2 reland passed 8/8.

**Arc 132 — Universal default.** The `:time-limit` annotation
was opt-in. A deftest without it had no runtime guard. After
arc 132, every deftest passes through the wrapper with a 200ms
default; explicit `:time-limit` is the override.

## The doctrine carrying forward

The wrapper now sits at the heart of the deadlock-class chain:

| Layer | Where | Coverage |
|---|---|---|
| Compile-time structural | arcs 117 / 126 / 131 | scope + channel-pair + HandlePool deadlocks |
| Runtime safety net | arc 123 + 129 + 132 | every deftest, 200ms default, override per-test |

Belt + three layers of suspenders. A new deadlock class
that bypasses every compile-time check still hits the 200ms
runtime guard.

## The four questions

**Obvious?** Yes. Test timeouts are universal in test
frameworks; Rust libtest doesn't have one because cargo test
itself can be stopped. Per-test annotation closes the gap.

**Simple?** Yes. ~60 LOC scanner + ~30 LOC codegen + 11
unit tests. The wrapper is one `thread::spawn` + one
`recv_timeout` + a match.

**Honest?** Yes. The doctrine is honest: tests should take
ms; the parser leads with ms; s/m suffixes are admitted
without being advertised. The original wrapper bug was honest
too — surfaced quickly in arc 126 slice 2 because the discipline
was to test the failure modes, not just the happy path.

**Good UX?** Phenomenal. Authors who know Rust tests don't
need new mechanisms; the annotation reads exactly like a
test budget. Arc 132's universal default makes even
unannotated tests fail-fast.

## Cross-references

- `DESIGN.md` — pre-implementation design.
- `docs/arc/2026/05/121-deftests-as-cargo-tests/INSCRIPTION.md`
  — the parent arc.
- `docs/arc/2026/05/122-per-test-attributes/INSCRIPTION.md` —
  the sibling arc whose state-machine pattern this arc reused.
- `docs/arc/2026/05/129-time-limit-disconnected-vs-timeout/INSCRIPTION.md`
  — the post-ship fix.
- `docs/arc/2026/05/132-deftest-default-time-limit/SCORE-SLICE-1.md`
  — the universal-default extension.
- `crates/wat-macros/src/discover.rs::parse_duration_ms` — the
  duration parser.
- `crates/wat-macros/src/lib.rs` — codegen of the wrapper.
- `crates/wat-sqlite/wat-tests/arc-123-time-limit.wat` —
  end-to-end fixture.
