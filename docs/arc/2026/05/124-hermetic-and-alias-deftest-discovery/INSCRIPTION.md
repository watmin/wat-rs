# Arc 124 — INSCRIPTION

## Status

**Shipped + closed 2026-05-01.** Same evening as arcs 121 +
122 + 123. Closes the cargo-test-parity gap that the arc-121
INSCRIPTION called 100% complete; the gap was real, just not
yet observed.

## What this arc closes

Arc 121 made each `(:wat::test::deftest <name> ...)` form
its own `#[test] fn`. Arcs 122 + 123 added per-test
attributes (`:ignore`, `:should-panic`, `:time-limit`).
Together they were sold as 100% cargo-test contract parity.

Arc 124 closed the silent invisibility of three other
deftest-producing shapes:

1. `(:wat::test::deftest-hermetic <name> ...)` — the
   forked-subprocess sibling of `deftest`. Used by every
   service-spawning test where in-process StringIo stdio
   would silently mask cross-thread driver panics.
2. `(:alias <name> <body>)` from `(:wat::test::make-deftest
   :alias <prelude>)` — the configured-prelude factory
   pattern.
3. `(:alias <name> <body>)` from
   `(:wat::test::make-deftest-hermetic :alias <prelude>)` —
   the configured-prelude hermetic variant.

Pre-arc-124, the proc-macro scanner only matched literal
`(:wat::test::deftest ...)` source text. Tests authored in
the other three shapes were silently undiscovered — no
`#[test] fn` emitted, no cargo test entry, no failure
visibility, no parity.

The gap surfaced during arc 119's stepping-stone
debugging. The brief instructed sonnet to use
`deftest-hermetic` (because HologramCacheService spawns
driver threads). The agent observed that none of the
existing HologramCacheService tests appeared in `cargo
test --list` — they're written as `(:deftest-hermetic ...)`
alias calls. The make-deftest-registered alias expands at
wat-side macro time to `:wat::test::deftest-hermetic`, but
the proc-macro scanner runs at compile time on raw source
text.

## What shipped

**Single file change** — `crates/wat-macros/src/discover.rs`:

- New scanner state: per-file alias table
  (`HashMap<String, ()>`) populated by `make-deftest` /
  `make-deftest-hermetic` calls.
- Three new top-level form arms in `scan_file`:
  - `:wat::test::deftest-hermetic` (mirror of `:wat::test::deftest`)
  - `:wat::test::make-deftest` (register alias keyword)
  - `:wat::test::make-deftest-hermetic` (register alias keyword)
- Catch-all alias arm: when a top-level form's head is in
  the alias table, treat as a deftest call.
- 10 new unit tests covering all four shapes, alias
  ordering, annotation composition, and unknown-alias
  silent drop. Pre-existing `scan_finds_nested_deftest`
  test (which asserted the OPPOSITE — that aliases
  weren't discovered) updated to
  `scan_finds_aliases_and_outer` (asserts both alias and
  direct forms produce sites).

**Zero runner changes.** The wat-side macros encode
hermetic vs in-process via the choice of
`run-sandboxed-ast` vs `run-sandboxed-hermetic-ast` in
the body's expansion. `run_single_deftest` looks up the
function by keyword name and calls it; it doesn't care
which sandbox the body chose. Same for alias forms — they
ultimately expand to the same `:wat::core::define` shape.

## What got surfaced

After arc 124 ships:

- `cargo test --release -p wat-holon-lru -- --list` shows
  **14 tests** (was 8). Six new HologramCacheService
  tests now first-class:
  ```
  deftest_wat_tests_holon_lru_HologramCacheService_test_step1_spawn_join
  deftest_wat_tests_holon_lru_HologramCacheService_test_step2_counted_recv
  deftest_wat_tests_holon_lru_HologramCacheService_test_step3_put_only
  deftest_wat_tests_holon_lru_HologramCacheService_test_step4_put_get_roundtrip
  deftest_wat_tests_holon_lru_HologramCacheService_test_step5_multi_client_via_constructor
  deftest_wat_tests_holon_lru_HologramCacheService_test_step6_lru_eviction_via_service
  ```
- step1 + step2 pass. step3-6 hang (timeout via
  arc-123's safety net at 200ms).
- `proofs/arc-119/step-B-single-put` also hangs — the
  minimal reproduction of the same shape.
- `crates/wat-lru/wat-tests/lru/CacheService.wat`'s
  cache-service round-trip test hangs the same way (was
  visible pre-124 — uses direct `:wat::test::deftest` —
  but was masked because the workspace test never
  reached it through the wat-suite aggregation pre-121).

**The deadlock class is now named.** Every test that
exercises the Pattern B Put-ack helper-verb cycle hangs:

- wat-lru: `test-cache-service-put-then-get-round-trip`
- wat-holon-lru: `test-step3-put-only` through
  `test-step6-lru-eviction-via-service`
- arc-119 proofs: `step_B_single_put` (minimal
  reproduction)

The shape: caller binds `(ack-tx, ack-rx)`, sends
`Request::Put` carrying `ack-tx` to the service, blocks
on `recv ack-rx`. The caller's original `ack-tx` clone
remains alive in scope; if the driver fails to send the
ack (driver panic, channel-in-channel issue), the
caller's `recv` never sees EOF because the caller still
holds a writer. **This is the next arc — a compile-time
deadlock-detection rule, sibling to arc 117's scope-
deadlock check.**

## Workspace hygiene

The six newly-discovered hanging tests carry
`(:wat::test::ignore "arc 119: Put-ack helper-verb cycle
deadlock; step 7 under investigation")` plus
`(:wat::test::time-limit "200ms")` as the safety net for
`cargo test --include-ignored` runs. cargo test reports
them as `ignored` with the reason string; the
deadlock-class signal is preserved without breaking the
workspace. Arc 119 step 7 unignores them as it closes
each scenario.

Workspace test result post-arc-124:

```
$ cargo test --release --workspace
... (every result is `ok`) ...
=== exit=0 ===

Notable counts:
  wat-holon-lru tests/test.rs:  9 passed; 0 failed; 5 ignored
  wat-lru tests/test.rs:        7 passed; 0 failed; 1 ignored
  wat-sqlite arc-122-attributes:1 ignored (intentional — verifies the mechanism)
```

## The four questions

**Obvious?** Yes. The proc-macro scanner needed to know
about three more head keywords + an alias table. The shape
mirrors how the wat-side macros register and expand —
the scanner just learns to recognize what wat already
emits.

**Simple?** Yes. ~50 LOC of scanner extension; one
HashMap; three additional match arms; one alias
catch-all. No new wat syntax, no new runner code, no new
file-format fields.

**Honest?** Yes. The arc names exactly the gap it closes
(four shapes, was matching one) and surfaces the
downstream consequence (six previously-invisible
deadlocking tests now first-class, ignored for now,
tracked toward arc 119). The 058 changelog row records
the mechanism change; the INSCRIPTION records the
diagnostic value.

**Good UX?** Phenomenal. Tests authored in any of the
four shapes now have full cargo-test parity. Authors
don't need to know which discovery path their preferred
syntax takes — the substrate covers them all. The
deadlock surface that was silently invisible became
loud-but-tracked in one commit.

## Cross-references

- `docs/arc/2026/05/121-deftests-as-cargo-tests/DESIGN.md`
  — the original per-deftest emission machinery.
- `docs/arc/2026/05/122-per-test-attributes/DESIGN.md` —
  annotations that now apply uniformly to all four
  shapes.
- `docs/arc/2026/05/123-time-limit/DESIGN.md` — the
  annotation that makes the workspace-hygiene step
  honest.
- `docs/arc/2026/04/119-holon-lru-put-ack/DESIGN.md` —
  the protocol fix that surfaced the discovery gap; arc
  124 made the fix observable.
- `docs/arc/2026/04/117-scope-deadlock-prevention/DESIGN.md`
  — the precedent for the next arc (compile-time
  detection of the Put-ack deadlock class).
- `crates/wat-macros/src/discover.rs` — the file that
  changes.
- `wat/test.wat` — the four wat-side macros the scanner
  now recognizes (lines 304-414).

## Queued follow-up

**Arc 125** — compile-time detection of the Pattern B
Put-ack helper-verb cycle deadlock. The structural
shape now has six concrete examples and one minimal
reproduction. The check belongs in `src/check.rs`
alongside arc 117's `validate_scope_deadlock`.
