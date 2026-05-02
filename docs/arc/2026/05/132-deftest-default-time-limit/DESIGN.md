# Arc 132 — Default time-limit on every deftest

**Status:** drafted 2026-05-01.

## TL;DR

Today: deftests with explicit `:wat::test::time-limit
"<dur>"` get a thread-spawn + recv_timeout wrapper; deftests
without it can hang indefinitely on a deadlock. Arc 132 makes
the wrapper the DEFAULT — every deftest gets it with an
opinionated default of **200ms**. Explicit annotations
override the default for tests that genuinely need longer.

The substrate stops shipping a "tests can hang forever unless
you remember to opt in" model. Future authors writing a
deadlock get a clean timeout panic in <1s regardless of
annotation. The deadlock-guard becomes structural.

## Provenance

Arc 130 slice 1's sonnet sweep deadlocked on a
HandlePool-while-join pattern. The `:time-limit "200ms"`
annotation on the test caught it as a timeout; arc 117 + arc
131 catch the structural shape at compile time. But the
deeper question: what about tests that DON'T have explicit
`:time-limit` annotations? Today those hang the test binary
indefinitely on any deadlock.

User direction (2026-05-01):

> "do you think we should have the timeout be expressed in
> the deftest declaration?... could we do that?... i want to
> guard rogue deadlocks in the future..."
>
> "we have an opinionated default and an escape hatch
> override if a user needs it?"
>
> "200ms - we're doing basically everything in memory - we
> can raise it on a per test case basis as we need later in
> time"

## The rule

> Every `:wat::test::deftest <name> ...` form gets a
> thread-spawn + recv_timeout wrapper at proc-macro emission
> time. The default budget is **200ms**. Explicit
> `:wat::test::time-limit "<dur>"` annotations override the
> default per-test.

Mirrors arc 045's discipline of opinionated defaults: `:error`
is the default capacity-mode; users override only when
necessary. Demos and tests show overrides, not defaults.

## The change

`crates/wat-macros/src/lib.rs`:

```rust
// Before (lines ~653-700):
let body = if let Some(ms) = site.time_limit_ms {
    let timeout_msg = format!(...);
    quote! {
        let __wat_handle = ::std::thread::spawn(...);
        match __wat_rx.recv_timeout(...) {
            Ok(_) => {}
            Err(Timeout) => panic!(#timeout_msg),
            Err(Disconnected) => { ... resume_unwind ... },
        }
    }
} else {
    quote! {
        // direct call, no wrapper
    }
};

// After:
const DEFAULT_TIME_LIMIT_MS: u64 = 200;
let ms = site.time_limit_ms.unwrap_or(DEFAULT_TIME_LIMIT_MS);
let timeout_msg = format!(...);
let body = quote! {
    let __wat_handle = ::std::thread::spawn(...);
    match __wat_rx.recv_timeout(...) {
        Ok(_) => {}
        Err(Timeout) => panic!(#timeout_msg),
        Err(Disconnected) => { ... resume_unwind ... },
    }
};
```

The `else` branch (no-wrapper path) retires entirely. Every
deftest has the same wrapper shape; only the budget value
differs (default vs explicit override).

## Why 200ms

Empirical: observed deftest runtimes in this workspace are
millisecond-scale. Aggregate runtimes per crate:

- wat-holon-lru: 14 tests in 0.06s aggregate (~4ms each)
- wat-lru: 8 tests in 0.03s aggregate (~4ms each)
- wat tests: 743 tests in <2s aggregate (~3ms each)

200ms is **50x typical**. Plenty of headroom for tests under
load (CI noise, slow disk on hermetic spawns). Tight enough
that a hung test surfaces in <1s (5 hung tests would surface
in <1s with parallelism).

The substrate's `wat::test!` infrastructure runs all tests
in-memory via `run-sandboxed-hermetic-ast` — even the
hermetic-fork variant is pure local I/O. There's no test that
should genuinely take >200ms unless it's deliberately slow.

User-chosen value: 200ms. Per arc 023's "opinionated defaults
are functions, not numbers" principle, the value reflects the
substrate's nature (in-memory, fast). Future arcs may
parametrize if integration tests need different defaults.

## Side benefit — universal panic-substring chain

Arc 129 fixed the `:time-limit` wrapper's `JoinHandle::join +
resume_unwind` panic propagation. Today, tests with
`:should-panic` AND `:time-limit` get correct substring
matching. Tests with `:should-panic` BUT NO `:time-limit`
take a different code path (no wrapper) — same correctness,
but inconsistent emission.

After arc 132, every deftest goes through the SAME wrapper.
`:should-panic` matching works uniformly; the panic-substring
chain is a single code path.

## Cost — existing tests

Tests that legitimately take >200ms will false-positive
timeout. We need to identify them in slice 2:

- Hermetic-fork tests with real I/O latency (file reads,
  large EDN parses)
- Tests that spawn many threads / processes
- Long-running subprocess tests (probably none today; the
  workspace is fast)

Sweep: `grep -rn "wat::test::time-limit" wat-tests/ crates/` to
find existing explicit annotations; those stay. Run workspace
test post-arc-132; identify timeouts; add explicit
annotations as needed.

## What this arc closes

- The "rogue deadlock hangs the suite" failure mode. Every
  deftest now has a default 200ms guard.
- The inconsistency between deftests with/without
  `:time-limit`. Single emission code path; uniform
  panic-substring propagation.
- The "did I remember to add `:time-limit`?" anxiety. The
  substrate handles it.

## Implementation plan

### Slice 1 — make the wrapper universal

`crates/wat-macros/src/lib.rs` change:
- Add `const DEFAULT_TIME_LIMIT_MS: u64 = 200;`
- Replace `if let Some(ms)` → `unwrap_or(DEFAULT_TIME_LIMIT_MS)`
- Retire the `else` (no-wrapper) branch
- Update comments at the function

Workspace tests that legitimately need >200ms will surface as
timeouts. Slice 1's verification: add explicit
`:wat::test::time-limit "<longer>"` to any test that times out
genuinely. Goal: workspace stays green post-arc-132.

### Slice 2 — closure

INSCRIPTION + USER-GUIDE update + WAT-CHEATSHEET note (every
deftest has 200ms default; opt out with explicit
`:time-limit`).

## The four questions

**Obvious?** Yes. "Tests should not hang" is universal. The
substrate already has the machinery (arc 123 + arc 129); we
just default-on it.

**Simple?** Tiny. ~5 LOC change in the proc-macro. The if-else
collapses to a single emission path with a default constant.

**Honest?** Yes. The current model lies — "tests don't hang
unless you forget the annotation." The proper model: tests
have a default budget; explicit annotation overrides. Honest +
uniform.

**Good UX?** Phenomenal. Future authors writing a deadlock
get fast structured feedback. They don't need to remember to
opt in. The substrate teaches by failing fast.

## Cross-references

- `docs/arc/2026/05/123-time-limit/DESIGN.md` — the
  annotation that arc 132 makes default-on.
- `docs/arc/2026/05/129-time-limit-disconnected-vs-timeout/INSCRIPTION.md`
  — the panic-propagation fix that makes the wrapper safe to
  default-on.
- `docs/arc/2026/04/045-capacity-mode-rename/INSCRIPTION.md` —
  the precedent for opinionated defaults: `:error` default
  + explicit override per-test.
- `crates/wat-macros/src/lib.rs:653-700` — the function whose
  if-else collapses.

## Failure-engineering record

Arc 132 is a SAFETY-NET arc, not a deadlock-detection arc.
It complements the structural checks (arc 117, arc 126, arc
131) by providing a runtime guard for any pattern those checks
miss. Belt + suspenders: structural enforcement at freeze
time, time-limit guard at runtime.

The chain:
- Arc 117: scope-deadlock structural (closure-captured
  Receiver + Thread/join-result)
- Arc 126: channel-pair-deadlock structural (both halves of
  one channel passed to one call)
- Arc 131 (in flight): HandlePool extension to arc 117
- **Arc 132 (this): runtime safety net for everything else**

Each layer catches a different class. Together they make
deadlock-class failures unmissable.
