# Arc 123 — Per-test `:time-limit` annotation

**Status:** **shipped + closed 2026-05-01.**
**Post-fix amendment 2026-05-01 via arc 129:** the
`recv_timeout` `Err(_)` arm conflated `Timeout` with
`Disconnected`, eating panic substrings from the spawned
thread when `:should-panic` was also annotated. Arc 129 split
the match arms and added `JoinHandle::join()` +
`std::panic::resume_unwind` on Disconnected to preserve the
inner panic message verbatim. See
`docs/arc/2026/05/129-time-limit-disconnected-vs-timeout/INSCRIPTION.md`.

End-to-end verified in
`crates/wat-sqlite/wat-tests/arc-123-time-limit.wat`:

```
test deftest_wat_tests_sqlite_arc_123_test_arc_123_fast            ... ok
test deftest_wat_tests_sqlite_arc_123_test_arc_123_seconds_suffix  ... ok
test deftest_wat_tests_sqlite_arc_123_test_arc_123_minutes_suffix  ... ok
test result: ok. 3 passed; 0 failed; 0 ignored
```

Three deftests with `100ms`, `5s`, `1m` budgets; all complete
trivially under-budget; all pass. The timer wrapper lands and
doesn't fire. The timeout-firing path will be validated on
arc-119's recovered hanging stepping stones — applying
`(:wat::test::time-limit "30s")` to each will surface the
deadlock as a clean timeout panic instead of an indefinite hang.

11 new scanner unit tests cover `parse_duration_ms` (ms/s/m
suffixes; rejection of missing/finer/long/non-numeric forms),
state-machine attachment, stacking with other annotations, and
clearing on unrelated forms.

## Provenance

Surfaced 2026-05-01 mid-arc-119 stepping-stone debugging. The
recovered HolonLRU step-tests hang (deadlock pattern not
caught by arc 117). With `cargo test` parity from arcs 121 +
122, a hung deftest hangs alone — but it still hangs. We
need a clean failure mode: the hung test should fail with a
clear timeout message, not block forever.

Rust libtest has no `#[timeout]` attribute. arc 123 implements
the per-test timeout ourselves at the proc-macro layer.

User direction:

> the test func we build in rust... at macro time we wrap it
> in a time block and panic out of we hit it...
>
> we should advertise millisecond resolution timeout.. users
> can use minutes and seconds.. but we don't actively
> demonstrate it.. it supported.. not suggested..

## Goal

Wat deftests carry a per-test time budget:

```scheme
(:wat::test::time-limit "500ms")
(:wat::test::deftest :my::potentially-slow ()
  (long-running-work))
```

If the body exceeds the budget, the proc-macro-emitted wrapper
panics with a clear timeout message. Cargo test reports the
test as failed (timeout); other tests continue.

## Doctrine — milliseconds first

Tests should take milliseconds. A test that needs seconds is a
smell. Examples in docs lead with ms (`"500ms"`, `"100ms"`).
The duration parser supports `s` and `m` suffixes too — they
exist for the rare exception (sandboxed integration tests,
genuinely slow file I/O, intentional sleep) — but the
documentation doesn't lead with them.

The resolution is **millisecond**: precision finer than ms is
not supported (ns / µs are not test-scale).

## Wat-side syntax

Sibling-form annotation, same shape as `:ignore` /
`:should-panic`:

```scheme
(:wat::test::time-limit "500ms")
(:wat::test::deftest :my::test () body)
;; → #[test] fn deftest_my_test() {
;;       let (tx, rx) = mpsc::channel();
;;       std::thread::spawn(move || {
;;           ::wat::test_runner::run_single_deftest(...);
;;           let _ = tx.send(());
;;       });
;;       match rx.recv_timeout(Duration::from_millis(500)) {
;;           Ok(_) => {}
;;           Err(_) => panic!("deftest_my_test: exceeded time-limit of 500ms"),
;;       }
;;   }
```

Stacks with `:ignore` and `:should-panic` like the others.

Substrate-side: `:wat::test::time-limit` registers as a no-op
`String -> unit` verb so the file type-checks.

## Duration parser

Suffix-match. Returns `Duration` in milliseconds-resolution.

| Input | Parsed |
|---|---|
| `"500ms"` | `Duration::from_millis(500)` |
| `"30s"` | `Duration::from_secs(30)` |
| `"5m"` | `Duration::from_secs(300)` |
| `"500"` (no suffix) | error: explicit unit required |
| `"500us"` / `"500ns"` | error: ms is the finest resolution |
| `"30sec"` / `"5min"` | error: short suffixes only (ms, s, m) |

Errors are compile-time (proc macro emits clear `compile_error!`).

## What gets wrapped

ONLY deftests with a `:time-limit` get the wrapper. Untimed
deftests stay as the simple direct-call shape (no
thread-spawn overhead for the common case).

## Leak honesty

When the timeout fires:

- The recv_timeout returns `Err`.
- The `#[test] fn` panics with the timeout message.
- **The spawned worker thread keeps running.** Rust threads
  cannot be safely killed from outside. The thread holds:
  - The wat substrate's frozen world (Arc; cleaned at process
    exit)
  - Possibly forked subprocesses (zombie children — reaped at
    process exit)
- When `cargo test` as a whole exits, the OS cleans up.

This is best-effort. The leak is contained to the cargo-test
process lifetime. We're honest about it in the panic message
("test thread leaked — process exit will reap").

A future arc could add SIGKILL semantics for hermetic forked
deftests, but arc 123 stays simple.

## Substrate work

### 1. Register the no-op verb

In `wat/test.wat`:

```scheme
(:wat::core::define
  (:wat::test::time-limit (_dur :wat::core::String) -> :wat::core::unit)
  ())
```

### 2. Extend DeftestSite

```rust
pub struct DeftestSite {
    pub file_path: PathBuf,
    pub name: String,
    pub ignore: Option<String>,
    pub should_panic: Option<String>,
    pub time_limit: Option<Duration>,   // arc 123
}
```

### 3. Extend the scanner

Recognize `(:wat::test::time-limit "<dur>")` as a sibling-form
annotation. Parse the duration string. Attach to next deftest.
Same state-machine extension as `:ignore` and `:should-panic`.

### 4. Emit the timer wrapper

Update the proc macro's per-deftest emission. When `time_limit`
is present, wrap the `run_single_deftest` call in
thread::spawn + recv_timeout. When absent, emit the direct
call (today's shape).

## Tests

Scanner: extend the existing 8 annotation tests with three more:
- `scan_attaches_time_limit_to_next_deftest`
- `scan_time_limit_with_seconds_suffix`
- `scan_time_limit_with_minutes_suffix`

Plus duration-parser unit tests (already validated by the
scanner accepting / rejecting various formats).

End-to-end: add a wat-side smoke test (similar to
`arc-122-attributes.wat`) that:
- Has a deftest with `:time-limit "100ms"` doing trivial work →
  passes
- Has a deftest with `:time-limit "1ms"` deliberately sleeping
  longer → fails with timeout
- Has a deftest with `:time-limit "5s"` (seconds; rare) → still
  works syntactically

## Execution checklist

| # | Step | Status |
|---|---|---|
| 1 | `parse_duration_ms` in `crates/wat-macros/src/discover.rs` — ms/s/m suffixes; rejects missing suffix, finer-than-ms, long suffixes (`sec`, `min`), non-numeric | ✓ done |
| 2 | `DeftestSite` gains `time_limit_ms: Option<u64>` | ✓ done |
| 3 | Scanner state-machine extended to recognize `(:wat::test::time-limit "<dur>")` | ✓ done |
| 4 | Scanner + parser unit tests — 11 new (3 valid suffixes, 4 rejection paths, 4 state-machine scenarios) | ✓ done |
| 5 | `wat::test!` proc macro emits `thread::spawn + recv_timeout` wrapper when `time_limit_ms` is `Some`; direct call otherwise | ✓ done |
| 6 | `:wat::test::time-limit` no-op verb registered in `wat/test.wat` with ms-first doctrine in the comment | ✓ done |
| 7 | End-to-end smoke test (`arc-123-time-limit.wat`) — 3 deftests, all suffixes exercised, all pass | ✓ done |
| 8 | INSCRIPTION-style closure block at top of this DESIGN | ✓ done (this commit) |

## Cross-references

- `docs/arc/2026/05/121-deftests-as-cargo-tests/DESIGN.md` —
  the per-deftest emission machinery.
- `docs/arc/2026/05/122-per-test-attributes/DESIGN.md` — sibling
  annotations that established the pattern this arc follows.
- `crates/wat-macros/src/discover.rs` — scanner extended here.
- `crates/wat-macros/src/lib.rs::test` — proc macro extended.
- `wat/test.wat` — no-op verb registered here.
- `crates/wat-sqlite/wat-tests/arc-122-attributes.wat` — model
  the smoke test on this file's pattern.
