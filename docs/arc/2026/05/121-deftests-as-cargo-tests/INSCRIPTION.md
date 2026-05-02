# Arc 121 — INSCRIPTION

## Status

**Shipped + closed 2026-05-01.** Together with arcs 122 + 123
(per-test attributes + time-limit) and arc 124 (hermetic +
alias discovery), this lands 100% cargo-test contract parity
for wat deftests.

The original closure note in `DESIGN.md` § "Closure" called arc
121 "95%" with arc 122 closing the remaining 5%. Sibling arcs
123 + 124 then completed the surface; the timer wrapper of arc
129 + the 200ms default of arc 132 finished the runtime side.
This INSCRIPTION is the durable record.

## What this arc closes

Pre-arc-121, every wat deftest in a test binary was hidden
inside one aggregated `#[test] fn wat_suite()`. From cargo's
perspective each `tests/test*.rs` was one test, regardless of
how many `(:wat::test::deftest ...)` forms it ran. Five
practical consequences fell out:

1. `cargo test <substring>` couldn't filter to a single deftest
   — the filter only saw `wat_suite`.
2. `cargo test --list` showed `wat_suite`, not the deftests.
3. A hanging deftest hung the whole suite; no way to skip it.
4. Output during `--nocapture` was buried inside `wat_suite`'s
   panic payload.
5. `--test-threads N` controlled binary-level parallelism, not
   per-deftest parallelism.

Surfaced 2026-05-01 mid-arc-119 stepping-stone debugging: a
hung step in step 7 made it impossible to run "just step A" of
the proposed sequence. User direction was unambiguous —
`make this work just the way cargo and rust are meant to do...
parity - make us a proper first class citizen`.

## What shipped

**Three pieces in `crates/wat-macros/`** + one runner
function in the substrate:

- `crates/wat-macros/src/discover.rs` (new file) —
  `scan_file(path) -> Vec<DeftestSite>`, a hand-rolled
  paren-balanced scanner that walks `.wat` source text and
  records each `(:wat::test::deftest <name> ...)` site. No
  dependency on the wat parser (avoids a dependency cycle —
  `wat-macros` is consumed by `wat`).
- `crates/wat-macros/src/lib.rs` — `wat::test! {...}` proc
  macro now walks the configured path at expansion time, calls
  `discover_deftests`, and emits one `#[test] fn
  deftest_<sanitized_name>()` per discovered site. Sanitization
  replaces `:`, `-`, `/` with `_`. Collision check at expansion
  time emits a compile error on duplicate sanitized names.
- `crates/wat-macros/src/lib.rs` — emits a hidden
  `const _WAT_FILE_DEPS: &[&[u8]] = &[ include_bytes!(...), ...]`
  per discovered file. The `include_bytes!` registers each
  `.wat` file as a build dependency so cargo re-runs the proc
  macro when wat files change. Standard Rust trick; the bytes
  themselves are never read.
- `src/test_runner.rs::run_single_deftest(path, deftest_name,
  dep_sources, registrars, loader)` — new runner. Loads the
  file via the configured loader, parses + freezes, locates the
  deftest by name, runs just it. Same diagnostics as the
  retired aggregating runner.

The aggregating `run_and_assert_with_loader` retired. Existing
callers (`tests/test*.rs` in each crate) needed zero edits —
the `wat::test! {...}` signature is unchanged.

## What got surfaced

The scanner discovered three deftest-producing forms beyond
`(:wat::test::deftest ...)` that were silently invisible:

1. `(:wat::test::deftest-hermetic ...)` — forked-subprocess
   variant.
2. `(:alias ...)` from `(:wat::test::make-deftest :alias
   <prelude>)` — configured-prelude factory.
3. `(:alias ...)` from `(:wat::test::make-deftest-hermetic
   :alias <prelude>)` — configured-prelude hermetic variant.

→ **Arc 124** extended the scanner with per-file alias-table
state to discover all four shapes.

Per-test attributes (`#[ignore]`, `#[should_panic]`) had no
analog in the wat-side annotations. The 95%-parity gap.

→ **Arc 122** added `:wat::test::ignore` /
`:wat::test::should-panic` annotation handling (sibling forms
preceding a deftest, attached via scanner state).

Hung tests still hung — per-deftest emission isolated the hang
to one `#[test] fn`, but that one fn still blocked indefinitely.

→ **Arc 123** added `:wat::test::time-limit` annotation +
`recv_timeout` wrapper around the spawned thread. Arc 129
fixed the wrapper's Timeout vs Disconnected conflation. Arc
132 made the wrapper universal with a 200ms default.

## Workspace test count change

Before arc 121: each `tests/test*.rs` reported as 1 passed test
(the `wat_suite` aggregator). Workspace total ~24 tests.

After arc 121 + 122 + 123 + 124 + 132: ~1700+ tests across the
workspace, every deftest first-class. `cargo test --list`
enumerates each by name. Filter / parallelism / failure
isolation work by libtest's native machinery.

## The four questions

**Obvious?** Yes. Rust tests have always worked this way; arc
121 is the substrate adopting the host's existing convention.

**Simple?** Medium. Three pieces (scanner, codegen, runner) +
the cache-invalidation `include_bytes!` trick. ~150 LOC across
`wat-macros` + ~80 LOC in `test_runner.rs`. No new wat syntax;
no env vars; no custom harness.

**Honest?** Yes. Pre-arc-121 was a substrate workaround for
something Rust does natively. The original aggregator was
incidental — early-stage scaffolding never updated.

**Good UX?** Phenomenal. Authors stop thinking about the
substrate's special handling; deftests behave like Rust tests.
The machinery becomes invisible.

## Cross-references

- `DESIGN.md` — the pre-implementation design (status section
  records the closure history).
- `AGENT-TRANSCRIPT-RECOVERY.md` — the doctrine for recovering
  agent edits from JSONL transcripts; surfaced during arc 121
  development. Cross-JSONL extension lives next to it as
  `recover-cross-jsonl.py`.
- `docs/arc/2026/05/122-per-test-attributes/INSCRIPTION.md`
- `docs/arc/2026/05/123-time-limit/INSCRIPTION.md`
- `docs/arc/2026/05/124-hermetic-and-alias-deftest-discovery/INSCRIPTION.md`
- `docs/arc/2026/05/129-time-limit-disconnected-vs-timeout/INSCRIPTION.md`
- `docs/arc/2026/05/132-deftest-default-time-limit/SCORE-SLICE-1.md`
- `crates/wat-macros/src/discover.rs` — the scanner
- `crates/wat-macros/src/lib.rs` — the proc macro
- `src/test_runner.rs::run_single_deftest` — the runner
