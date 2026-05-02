# Arc 121 — Deftests as first-class cargo tests

**Status:** **shipped + closed 2026-05-01** (100% complete after
arc 122 closed the per-test-attribute gap).

## What's shipped

Each `(:wat::test::deftest <name> ...)` form is now its own
`#[test] fn`. Cargo sees deftests as first-class tests:

- `cargo test <substring>` filters via libtest's native argument
  parsing.
- `cargo test --list` shows every deftest by name.
- `cargo test --nocapture` / `--show-output` stream per-deftest.
- `cargo test --test-threads N` controls per-deftest parallelism.
- `cargo test --exact` matches the sanitized name.
- A hanging deftest hangs alone, not a whole suite.
- Non-zero cargo exit on any deftest failure.
- `#[ignore = "reason"]` and `#[should_panic(expected = "...")]`
  per deftest (closed by arc 122 — see § "Closure" below).

Pre-arc-121 callers are unaffected: existing `wat::test!`
invocations with default path or string-literal path continue
working with no edits.

## Closure (2026-05-01)

The audit immediately following arc 121's substrate landing
asked: "did we satisfy cargo test contract completely?" Honest
answer was 95% — major behaviors complete (filter, list,
parallel, isolation, exit codes), minor extension points open
(`#[ignore]`, `#[should_panic]`).

Arc 122 closed the gap. After arc 122 ships:

- `(:wat::test::ignore "<reason>")` preceding a deftest →
  `#[ignore = "<reason>"]` on the generated `#[test] fn`.
- `(:wat::test::should-panic "<expected substring>")` preceding
  a deftest → `#[should_panic(expected = "<...>")]`.

End-to-end verified in
`crates/wat-sqlite/wat-tests/arc-122-attributes.wat`:

```
test deftest_wat_tests_sqlite_arc_122_test_arc_122_plain ... ok
test deftest_wat_tests_sqlite_arc_122_test_arc_122_should_panic
  - should panic ... ok
test result: ok. 2 passed; 0 failed; 1 ignored
```

100% cargo test contract parity. First-class citizen.

User direction (2026-05-01):
> if its a new arc, then its a new arc - go make it, then we
> close this out as dependent on the new one — 100% close out

## Provenance

Surfaced 2026-05-01 mid-arc-119 stepping-stone debugging.

The current `wat::test!` proc-macro emits a single `#[test] fn
wat_suite()` that aggregates every `(:wat::test::deftest ...)` it
discovers in the configured path. From cargo's perspective there
is one test per `tests/test*.rs` binary, regardless of how many
deftests live under it. The aggregation hides every deftest from
cargo's machinery:

- `cargo test step-A` cannot filter to a single deftest — the
  filter only sees `wat_suite`.
- `cargo test --list` shows `wat_suite` as one line, not the
  deftests inside.
- Output during `cargo test --nocapture` is buried inside the
  aggregated `wat_suite` panic payload on failure.
- Parallelism happens between `wat_suite`s (test binaries),
  not between deftests.
- A hanging deftest hangs the entire `wat_suite` and there's no
  way to skip it.

That last bullet is what surfaced the need: arc 119's step 7
agent introduced a hang in one wat-test, and the wat-suite
aggregation made it impossible to run "just step A" of a
proposed stepping-stone debugging sequence. The user's
direction:

> make this work just the way cargo and rust are meant to do...
> no extra fancy things - parity - make us a proper first class
> citizen. the rust ecosystem shouldn't view it any other way

## Goal

Each `(:wat::test::deftest <name> ...)` form is a proper Rust
`#[test] fn`. Cargo's libtest sees them as first-class tests.
Native cargo behavior — no env vars, no custom harness, no
extra mechanisms.

After this arc:

- `cargo test step-A` runs only deftests whose sanitized name
  contains "step-A".
- `cargo test --list` shows every deftest by name.
- `cargo test --nocapture` streams output per deftest.
- Parallelism: libtest runs deftests in parallel across cargo
  threads.
- A hanging deftest hangs only its own `#[test] fn`, not the
  whole binary.

## Non-goals

- **No libtest_mimic** or custom test harness. Pure libtest;
  pure `#[test] fn`.
- **No new env vars** (`WAT_TEST_FILTER` etc). Cargo's native
  filter mechanism is the only filter.
- **No new test syntax in wat.** The deftest form stays as it
  is. Behavior change is invisible to wat code authors.
- **No selective re-discovery.** Adding a deftest forces a
  recompile of the test binary. That's how Rust tests work
  today; we match.

## Architecture

Three pieces.

### 1. Deftest discovery in `wat::test!` proc macro

The macro walks the configured path at expansion time using
`std::fs`. For each `.wat` file found, it scans the file
contents for `(:wat::test::deftest <name> ...)` forms.

Scanner shape: hand-rolled paren-balanced reader. Skip
comments (lines starting with `;` or sub-line `;;`). Find
`(:wat::test::deftest`; read the next keyword as the test
name; that's the discovered deftest. ~30-50 lines; no
dependency on the wat parser.

Why hand-rolled instead of calling the wat parser: avoids a
dependency cycle (`wat-macros` cannot easily depend on `wat`
because `wat` uses `wat-macros`). The discovery pattern is
unambiguous textually and a small scanner is robust against
comments without needing the full type-checker.

### 2. Per-deftest `#[test] fn` emission

For each (file, deftest_name) pair, emit:

```rust
#[test]
fn deftest_<sanitized_name>() {
    ::wat::test_runner::run_single_deftest(
        ::std::path::Path::new(<file_path>),
        <deftest_name>,
        &[ <wat_sources>... ],
        &[ <register_paths>... ],
        <loader>,
    );
}
```

Naming: replace `:` with `_`, `-` with `_`, `/` with `_`.
Collision check at macro expansion time; emit a compile error
on duplicate sanitized names.

Each `#[test] fn` calls a NEW runner function that loads the
substrate, parses ONE file, finds ONE deftest by name, runs
just it.

### 3. `run_single_deftest` in `src/test_runner.rs`

New runner function. Loads the file via the configured loader,
parses + freezes, locates the deftest by name, runs it,
panics with the structured failure summary if it fails. Same
diagnostics as today's aggregated runner; just one test at a
time.

The aggregating `run_and_assert_with_loader` retires (or stays
as a deprecated shim).

## Cache invalidation

Proc macros run once per crate compile. Cargo's incremental
compilation only re-runs them when the macro's input source
code changes — NOT when external files (the `.wat` files) the
macro reads change.

Solution: emit `include_bytes!` for each discovered `.wat`
file's path INSIDE the macro's expansion. `include_bytes!`
registers the file as a build dependency; if the file changes,
Cargo recompiles the test binary which re-runs the proc macro
which re-discovers deftests.

Even though the bytes aren't used for anything, the
`include_bytes!` is the side-effect that makes cargo track the
file. Standard Rust trick.

```rust
// emitted inside the macro's expansion, hidden from users
const _WAT_FILE_DEPS: &[&[u8]] = &[
    include_bytes!("../wat-tests/proofs/arc-119/step-A.wat"),
    // ... one per discovered file
];
```

## Migration

Existing `tests/test*.rs` files use `wat::test! { deps: [...] }`
or `wat::test! { path: ..., deps: [...] }`. The macro signature
stays unchanged. Behavior change is invisible to callers — they
get more `#[test] fn`s instead of one. Tests pass as before;
output changes shape.

The single existing call site that needs review:
- Each consumer crate's `tests/test.rs`. Inspection only —
  expect zero edits.

Test count change: the workspace had 1476 passed (1475 baseline
+ vocare audit's three) before arc 121. After arc 121 each
deftest counts independently. Total passed count grows by the
sum of deftests minus the wat-suite aggregators that retire.

## What changes in the test output

Before:

```
running 1 test
test wat_suite ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; ...
```

After (e.g. wat-holon-lru with N deftests):

```
running 12 tests
test deftest_holon_lru_HologramCache_test_capacity_returns_max ... ok
test deftest_holon_lru_HologramCache_test_get_bumps_lru ... ok
test deftest_holon_lru_HologramCacheService_test_step1_spawn_join ... ok
... etc

test result: ok. 12 passed; 0 failed; 0 ignored; ...
```

## Execution checklist (compaction-amnesia-resistant)

| # | Step | Status |
|---|---|---|
| 1 | Add `discover_deftests(path: &Path) -> Vec<DeftestSite>` to `wat-macros` — paren-balanced scanner; ~30-50 lines | pending |
| 2 | Rewrite `wat::test!` proc macro to call `discover_deftests`, emit per-deftest `#[test] fn`, emit `include_bytes!` cache-invalidation list | pending |
| 3 | Add `pub fn run_single_deftest(path, deftest_name, dep_sources, registrars, loader)` to `src/test_runner.rs` — load + parse + locate + run one deftest | pending |
| 4 | Decide: retire `run_and_assert_with_loader` or keep as shim? Recommendation: retire (no callers post-step-2) | pending |
| 5 | `cargo test --release --workspace` — verify per-deftest test counts and passing | pending |
| 6 | Sweep test output references in docs (USER-GUIDE, CONVENTIONS, README) — anywhere that talks about `wat_suite` | pending |
| 7 | INSCRIPTION + 058 changelog row | pending |

## Discipline anchors

- **No env vars.** This arc's whole point is parity. Cargo's native filter is the only filter.
- **No new wat syntax.** Deftests stay declared as they are; behavior change is purely Rust-side.
- **`#[test] fn` per deftest.** One Rust test per wat deftest. No aggregation, no batching, no sub-tests.
- **Native libtest.** Not libtest_mimic, not a custom harness. Standard `#[test]`.

## Cross-references

- `crates/wat-macros/src/lib.rs::test` — the proc macro to
  rewrite.
- `src/test_runner.rs::run_and_assert_with_loader` — the
  aggregating runner that retires.
- `src/test_runner.rs::is_test_function` (line ~513) — the
  existing deftest discovery shape; reuse the criteria.
- `docs/arc/2026/04/119-holon-lru-put-ack/DESIGN.md` § "Realization"
  — the use case that surfaced this gap.
- `docs/arc/2026/04/119-holon-lru-put-ack/BRIEF-CONSUMER-SWEEP.md`
  — the in-flight arc that needs this prerequisite.

## Sequencing

Arc 121 is a prerequisite for resuming arc 119's stepping-stone
debugging. After arc 121 ships:

1. Arc 119 step 7 resumes — but with stepping stones (`step-A.wat`
   through `step-E.wat`) each runnable individually via
   `cargo test step-A`.
2. The hanging deftest from the agent's earlier work surfaces
   as a single hung `#[test] fn` instead of a hung whole-suite.

Arc 119 picks up cleanly atop arc 121.
