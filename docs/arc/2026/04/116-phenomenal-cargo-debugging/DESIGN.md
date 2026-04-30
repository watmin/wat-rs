# Arc 116 — phenomenal cargo debugging via data-first diagnostics

## Status

Drafted 2026-04-30 immediately after arc 115 slice 1 shipped. Lifts
the data-first `Diagnostic` schema arc 115 minted into the test
runner's panic path so `cargo test` failures surface structured
fields (location, actual, expected, frames, hint) verbatim instead
of as flattened text.

User direction (2026-04-30):

> ok... how do we make :wat::test use this.. how can we make cargo
> debugging phenominal?..
>
> ok - go make 116 and get this in place then we'll circle back to
> 115

## The pathology

Today's `cargo test` failure surface for a wat-test:

```
thread '<unnamed>' panicked at file.wat:42:13:
assert-eq failed
  actual:   1
  expected: 2
```

This IS partially structured (arc 064's `:wat::kernel::Failure`
carries actual / expected / location / frames as data; the test
runner's `format!`-based renderer at `src/test_runner.rs:537`
walks those fields). But:

1. The panic message is built by string concatenation; tooling
   consumers (CI, agents, editor LSP) parse strings.
2. Test-DISCOVERY freeze failures (when a `.wat` test source
   doesn't compile) panic the whole test suite with a Debug print
   of the StartupError. Cryptic.
3. Cross-host failures (a wat-test that forks a Program that
   panics) bottom out at "child exited code 1" — the structured
   `Failure` is text-rendered into stderr and lost. Arc 113 fixes
   this with the `Vec<ProgramDiedError>` chain; arc 116 lands the
   FRAMING the chain renders into.
4. There's no machine-readable mode — agents running `cargo test`
   in CI parse stderr text.

## The shape

Arc 115 minted the data-first foundation: `Diagnostic` struct,
`StartupError::diagnostics()`, `CheckError::diagnostic()`, render
helpers. Arc 116 extends:

- **`Failure::diagnostic()`** — produces a `Diagnostic` with
  `kind: "AssertionFailed"` (or `kind: "Panic"` for non-assertion
  panics) + structured fields mirroring arc 064's Failure shape:
  `location`, `actual`, `expected`, `frames` (Vec via repeated
  field entries), and the existing `message`.
- **Test-discovery freeze surface** — `test_runner.rs` already
  catches StartupError during discovery; the panic message uses
  `StartupError::diagnostics()` to render structured per-error
  blocks (same shape `wat --check --check-output edn` produces).
- **`WAT_TEST_OUTPUT=edn|json` env var** — when set, the test
  runner emits one `Diagnostic` record per failure to stdout (line-
  delimited per arc 092 v4) BEFORE panicking with the (still
  human-readable) text rendering. CI / agents read the structured
  records; humans see the text.

## The four-layer rendering

```
LAYER                          RESPONSIBILITY
───────────────────────────────────────────────────────────────
Substrate (data)               build Diagnostic with structured fields
   │
   ├─ Failure::diagnostic()    arc 064 Failure → Diagnostic
   ├─ StartupError::diagnostics() arc 115 (already)
   └─ CheckError::diagnostic() arc 115 (already)
   ▼
test_runner consumer           consumes Vec<Diagnostic>
   │
   ├─ Text mode (default)      walks fields; produces human-readable
   │                           panic message (arc 116 reformat — same
   │                           data, cleaner layout, hint surfaced)
   ├─ EDN mode (env)           render_edn each Diagnostic to stdout
   └─ JSON mode (env)          render_json each Diagnostic to stdout
   ▼
cargo test displays            stderr text → human; stdout structured
                               → CI / agents
```

The substrate produces ONE source of truth; renderers layer.

## What this arc does NOT do

- Does NOT change wat-side test surface (`:wat::test::deftest`,
  `assert-eq`, `assert-stdout-is`, etc.). Their behavior is
  unchanged; only the test runner's failure-rendering pipeline
  reshapes.
- Does NOT add cross-host chain rendering. That's arc 113's
  `Vec<ProgramDiedError>`; arc 116 ships the SINGLE-frame
  rendering with structured data. Once arc 113 lands, the chain
  cleanly drops into the same Diagnostic-walking framework.
- Does NOT introduce new test-runner output formats beyond
  `text` (default), `edn`, `json`. Future tooling formats (LSP
  protocol, GitHub Actions annotations) layer on top.
- Does NOT touch arc 064's `:wat::kernel::Failure` shape — only
  adds a `diagnostic()` conversion method.

## The four questions

**Obvious?** Yes. Test failures are data; cargo's panic is one
renderer; CI's structured consumption is another. Same source.
Naming the layer split is the win.

**Simple?** Yes. ~50 LOC for `Failure::diagnostic()`; ~30 LOC for
the env-var-gated structured emission in test_runner; the text
renderer is already structured-by-fields per arc 064 — we just
re-route through Diagnostic.

**Honest?** Yes. The substrate's Failure already IS structured;
arc 116 stops flattening it and propagates the data shape outward.
Cross-host chains stay as a known limitation until arc 113.

**Good UX?** Phenomenal. The same `cargo test` invocation gives
humans the readable text output today's wat-test users expect;
agents pipe `WAT_TEST_OUTPUT=edn cargo test` and consume records
without parsing stderr.

## Implementation slices

### Slice 1 — `Failure::diagnostic()`

`src/runtime.rs` (or wherever `Failure` lives — probably `types.rs`
field def + `runtime.rs` constructor):

- Add `pub fn diagnostic(&self) -> Diagnostic` method on the
  Failure StructValue.
- Maps to:
  ```
  Diagnostic::new("AssertionFailed" | "Panic")
    .field("message", failure.message)
    .field("location", failure.location)   // Optional
    .field("actual",   failure.actual)     // Optional
    .field("expected", failure.expected)   // Optional
    .field("frame_N",  failure.frames[N])  // Repeated entries
  ```
- Determines AssertionFailed vs Panic by whether actual/expected
  are populated.

Probe: unit test that constructs a Failure (via existing helpers)
and asserts the resulting Diagnostic's kind + fields.

### Slice 2 — test runner uses Diagnostic for runtime failures

`src/test_runner.rs` lines 530-560 (the `format!`-based renderer):

- Replace direct `format!("\n    actual: {}", a)` etc. with a
  walk over `failure.diagnostic().fields`.
- Same human-readable text output (the text renderer's job is to
  format; the data shape is upstream).

No behavior change for cargo test users; substrate-internal
refactor that paves the way for slice 3.

### Slice 3 — `WAT_TEST_OUTPUT=edn|json` env var

`src/test_runner.rs`:

- Read `WAT_TEST_OUTPUT` at the failure-emission site.
- When set, each failure also emits one EDN/JSON record to stdout
  via `render_edn` / `render_json`.
- Default (unset) preserves today's text-only behavior.

For test-discovery freeze failures (currently a panic at
`run_and_assert_with_loader`), surface the same structured
records BEFORE the panic — so even when discovery dies, the
records are already on stdout.

Documentation: USER-GUIDE callout for `WAT_TEST_OUTPUT`. Used by
CI / agents that want structured failure data without parsing
cargo-test text output.

### Slice 4 — test-discovery freeze failures use Diagnostic

`src/test_runner.rs::run_and_assert_with_loader`:

- Today: `panic!("test discovery failed: {:?}", err)` (Debug form).
- After: `for diag in err.diagnostics() { ... render structured }`
  + final panic with concise summary.

Visible improvement: when a `.wat` test source has a type error,
the cargo test output shows the same structured TypeMismatch
blocks `wat --check` produces — same diagnostic stream, same hints.

### Slice 5 — INSCRIPTION + USER-GUIDE + 058 row

Standard closure. USER-GUIDE adds:

- A "Debugging cargo test" section showing the new structured
  output and the `WAT_TEST_OUTPUT` env var.
- A "Test failure rendering" subsection naming the four-layer
  architecture.

## Cross-references

- `docs/arc/2026/04/115-no-inner-colon-in-parametric-args/DESIGN.md`
  — the data-first foundation (`Diagnostic` struct, render helpers,
  `StartupError::diagnostics()`) arc 116 builds on.
- `docs/arc/2026/04/064-assertion-payload/INSCRIPTION.md` (or
  wherever) — arc 064's `:wat::kernel::Failure` shape arc 116
  exposes via `diagnostic()`.
- `docs/arc/2026/04/105-spawn-error-as-data/INSCRIPTION.md` —
  arc 105c's `ThreadDiedError::Panic { message, failure }` —
  the `failure` field carries the same Failure arc 116 renders.
- `docs/arc/2026/04/113-cascading-runtime-errors/DESIGN.md` —
  the future `Vec<ProgramDiedError>` chain arc 116's framework
  consumes verbatim once 113 lands.

## Queued follow-ups

- **Cross-host chain rendering** — arrives with arc 113. Test
  runner walks `Vec<ProgramDiedError>` and emits one Diagnostic
  per chain entry; together they give the full "FAILED at layer-3
  because layer-2 died because layer-1 died" picture.
- **Editor LSP integration** — once `WAT_TEST_OUTPUT=json` is
  shipped, an LSP server can subscribe to cargo-test output and
  surface structured diagnostics inline. Out of scope for arc 116;
  the data foundation makes it possible.
- **GitHub Actions annotations** — render JSON failures into
  `::error file=...,line=...,col=...::message` comments. Trivial
  consumer of arc 116's output.
