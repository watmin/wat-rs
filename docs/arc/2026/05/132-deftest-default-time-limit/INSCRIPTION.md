# Arc 132 — INSCRIPTION

## Status

**Shipped + closed 2026-05-01.** One slice; one sweep:

- **Slice 1** — universal wrapper + 200ms default constant +
  one wat-test annotation file. One sonnet sweep: 8/8 hard +
  4/4 soft. Commit `007c9c8`.
- **Slice 2** — this INSCRIPTION + USER-GUIDE update +
  WAT-CHEATSHEET §10 cross-reference (already updated as part
  of arc 131 slice 3 work).

The arc closed the gap between arc 123's per-test opt-in
`:time-limit` annotation and arc 132's universal default. Every
deftest now passes through the timer wrapper; explicit
annotation is the override path.

## What this arc adds

Arc 123 made `:time-limit` a per-test opt-in. A deftest without
the annotation had no runtime guard. After arc 121 + 122
isolated hung tests to single `#[test] fn`s, that one fn could
still hang indefinitely on a deadlock the structural checks
hadn't caught.

Arc 132 inverts the model: every deftest passes through the
wrapper with a 200ms default. Tests genuinely needing more
budget pass an explicit `:wat::test::time-limit` to override.
The annotation becomes a per-test ceiling raise, not the only
path to a guard.

## The doctrine — 200ms

The default was chosen deliberately (per BRIEF):

- The substrate operates entirely in-memory for non-IO tests.
  A correctly-written test should finish in microseconds to
  low milliseconds.
- 200ms is ~50x typical test runtime — tight enough to
  surface a hang within seconds of `cargo test`, generous
  enough that incidental work (sqlite-less tests, light
  sandbox spawn, channel-based coordination) fits without
  per-test annotations.
- Tests that genuinely need more (sqlite I/O against a real
  `.db` file; sandboxed integration with subprocess startup)
  set an explicit budget and document why in the annotation
  string.

The 200ms is not a sacred number. The rule is "tight default
that surfaces hangs fast; per-test override available." Future
substrate work may tune the default if the workspace runtime
profile shifts.

## What shipped

**Single Rust file change** — `crates/wat-macros/src/lib.rs`:

```rust
const DEFAULT_TIME_LIMIT_MS: u64 = 200;
let ms = site.time_limit_ms.unwrap_or(DEFAULT_TIME_LIMIT_MS);
```

The pre-arc-132 `if/else` (one branch wraps with timer; other
emits the body bare) collapses to a single `quote! { ... }`
emission. The wrapper preserved arc 129's split-arms shape
verbatim — `Err(Timeout)` panics with the duration substring;
`Err(Disconnected)` joins the inner `JoinHandle` and
`panic::resume_unwind(payload)` so the inner panic message
survives.

**One wat-test file change** —
`crates/wat-telemetry-sqlite/wat-tests/telemetry/reader.wat`:
6 deftests that genuinely take >200ms (sqlite spawn + open +
stream + collect over a real `.db`) gained
`(:wat::test::time-limit "2s")`. The 2-second budget absorbs
CI noise comfortably.

End-to-end: `cargo test --release --workspace` exit=0; 100
result blocks all `ok`; 0 failed; 1 ignored (pre-existing
arc-122 mechanism test).

## The deadlock-class chain — completed

Arc 132 closes the runtime safety net atop the compile-time
chain:

| Layer | Arc | Coverage |
|---|---|---|
| Compile-time structural | 117 | scope-deadlock (Sender + Channel) |
| Compile-time structural | 126 | channel-pair-deadlock |
| Compile-time structural | 131 | scope-deadlock (HandlePool extension) |
| Compile-time structural | 133 (in flight) | tuple-destructure binding visibility |
| **Runtime safety net** | **132** | **default 200ms; override per-test** |

A new deadlock class that bypasses every compile-time check
still fails-fast at the 200ms default. Belt + 3-4 layers of
suspenders.

## What got surfaced

Sonnet's slice 1 report flagged a calibration delta worth
preserving:

- The brief's prose said "more than ~5 timeouts → STOP," but
  row 6 of the EXPECTATIONS scorecard said "≤5 wat-test files."
  Sonnet's run hit 6 deftests in 1 file. Sonnet went with the
  row spec but flagged the discrepancy. Future briefs: prose
  narrative should match row text verbatim. Calibration data,
  not a substrate gap.

The arc surfaced no follow-on substrate gaps. The wrapper had
already absorbed arc 129's fix; arc 132's universal-default
extension was a pure ergonomics improvement.

## The four questions

**Obvious?** Yes. Tests should fail-fast on hangs; the absence
of a default was an oversight from arc 123's opt-in framing.

**Simple?** Yes. ~5 LOC of substantive change (const +
unwrap_or + if/else collapse) + 6 annotation lines in
reader.wat. Net diff -10 LOC due to indentation shift.

**Honest?** Yes. The doctrine names what's true: 200ms is a
tight default informed by the substrate's actual workload.
The override path stays available for genuinely-slower tests.

**Good UX?** Phenomenal. Authors stop thinking about
`:time-limit` for normal tests; the substrate guards them by
default. Slow tests document their own budgets at the
annotation site.

## Cross-references

- `DESIGN.md` — pre-implementation design.
- `BRIEF-SLICE-1.md` + `EXPECTATIONS-SLICE-1.md` + `SCORE-SLICE-1.md`
- `docs/arc/2026/05/123-time-limit/INSCRIPTION.md` — the
  parent arc that established the per-test annotation.
- `docs/arc/2026/05/129-time-limit-disconnected-vs-timeout/INSCRIPTION.md`
  — the wrapper bug fix this arc preserved.
- `docs/arc/2026/04/117-scope-deadlock-prevention/INSCRIPTION.md`
  + `docs/arc/2026/05/126-channel-pair-deadlock-prevention/INSCRIPTION.md`
  + `docs/arc/2026/05/131-handlepool-scope-deadlock/INSCRIPTION.md`
  — the compile-time deadlock checks this arc layers a
  runtime guard atop.
- `docs/USER-GUIDE.md § "Tests — one macro, same shape"` —
  the user-facing surface, updated to reflect the 200ms
  default.
- `crates/wat-macros/src/lib.rs:660` — `DEFAULT_TIME_LIMIT_MS`
  constant.
- `crates/wat-macros/src/lib.rs:661` — the `unwrap_or`
  emission.
