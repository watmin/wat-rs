# Arc 116 — INSCRIPTION

## Status

Shipped 2026-04-30. Phenomenal cargo debugging via data-first
diagnostic propagation. cargo test --release green throughout
slices; 740 lib tests (6 new arc 116 tests + 3 arc 115 carry-over)
+ 97 integration test result rows; 0 failures.

Pushed:
- Slices 1-3: `dc0c017` (Failure→Diagnostic + render_failure_text +
  WAT_TEST_OUTPUT env var + 6 unit tests)
- Slice 4: this slice (test-discovery freeze failures use
  `StartupError::diagnostics()`)
- Slice 5: this INSCRIPTION + USER-GUIDE update + 058 row

## What this arc adds

A single source of truth for test-failure data. The substrate's
`:wat::kernel::Failure` (arc 064) was already structured;
`StartupError::diagnostics()` (arc 115) gave us structured freeze
errors. Arc 116 routes both through the test runner's panic path
without flattening — humans see the same text output today; tooling
consumers opt in via `WAT_TEST_OUTPUT=edn|json` and consume one
record per failure.

| Surface | Pre-arc-116 | Arc 116 |
|---|---|---|
| Runtime assertion failure (text) | `format!`-built string | walk-Diagnostic-fields render |
| Runtime assertion failure (structured) | not available | `#wat.diag/AssertionFailed {:test "..." :location "..." :actual "..." :expected "..."}` (one EDN/JSON record per failure) |
| Test-discovery freeze failure (text) | flattened `e.to_string()` | preserved |
| Test-discovery freeze failure (structured) | not available | one Diagnostic per CheckError, same shape `wat --check` produces |
| Runtime panic (non-assertion) | text only | `#wat.diag/Panic` record + text |
| Test panic escape | text only | `#wat.diag/TestPanicEscaped` record + text |

### The four-layer rendering pipeline

```
LAYER                          RESPONSIBILITY
───────────────────────────────────────────────────────────────
Substrate (data)               build Diagnostic with structured fields
   │
   ├─ Failure::diagnostic()    arc 064 Failure → Diagnostic (arc 116)
   ├─ StartupError::diagnostics() arc 115 (used at slice 4)
   └─ CheckError::diagnostic() arc 115 (composed via above)
   ▼
test_runner consumer           consumes Diagnostic
   │
   ├─ Text mode (default)      render_failure_text walks fields
   ├─ EDN mode (env)           render_edn each Diagnostic
   └─ JSON mode (env)          render_json each Diagnostic
   ▼
cargo test displays            stderr text → human; stdout structured
                               → CI / agents
```

The substrate produces ONE source of truth; renderers layer.

### Three Diagnostic kinds for runtime failures

- `AssertionFailed` — `actual` and `expected` populated; arc 064's
  `assert-eq` pathway.
- `Panic` — `message` populated; `actual`/`expected` absent. Plain
  `panic!` calls.
- `MalformedTestResult` — RunResult shape unexpected; substrate-
  internal diagnostic.

Plus the runtime-error and test-panic-escape paths emit
`RuntimeError` and `TestPanicEscaped` diagnostics respectively.

### `WAT_TEST_OUTPUT` env var

```
WAT_TEST_OUTPUT=edn cargo test    # one EDN record per failure to stdout
WAT_TEST_OUTPUT=json cargo test   # one JSON record per failure to stdout
cargo test                        # text-only (default; pre-arc-116 behavior)
```

EDN format example:
```
#wat.diag/AssertionFailed {:test ":proof::004::step-A" :message "assert-eq failed" :location "step-A.wat:42:13" :actual "1" :expected "2"}
```

JSON format example:
```
{"kind":"AssertionFailed","test":":proof::004::step-A","message":"assert-eq failed","location":"step-A.wat:42:13","actual":"1","expected":"2"}
```

Each record arrives on stdout as the failure happens; consumers
stream the records without waiting for the test suite to end.

## Why

The user direction:

> ok... how do we make :wat::test use this.. how can we make cargo
> debugging phenominal?..

Pre-arc-116 surface for a wat-test failure was already partially
structured (arc 064 surfaces actual/expected/location/frames in the
panic message), but the rendering was done by direct `format!`
strings inside the test runner. Tooling consumers parsed text.

Arc 116 names the architecture: substrate produces data; renderers
consume. The same Failure that lands in cargo test's panic message
becomes a single EDN record CI / LSP / agents read directly. The
text rendering stays as-is for humans; the structured stream is
opt-in for tools.

## What this arc does NOT do

- Does NOT change wat-side test surface. `:wat::test::deftest`,
  `assert-eq`, `assert-stdout-is`, etc. are unchanged. Only the
  test runner's failure-rendering pipeline reshapes.
- Does NOT introduce cross-host chain rendering. That's arc 113's
  `Vec<ProgramDiedError>`. When arc 113 lands, the test runner
  already walks Diagnostics — the chain becomes one diagnostic per
  cause.
- Does NOT add per-test-pass success records to the structured
  stream. Tooling that wants pass tracking reads cargo test's own
  output. Arc 116's structured stream is failure-only.
- Does NOT touch arc 064's `:wat::kernel::Failure` shape — only
  adds the conversion to Diagnostic at the test_runner seam.

## Slice walkthrough

### Slice 1 — `failure_to_diagnostic`

`src/test_runner.rs::failure_to_diagnostic(value: &Value) ->
Option<Diagnostic>` walks RunResult.failure → Failure struct fields
→ Diagnostic with named fields (message, location, actual,
expected, frame_0..frame_N).

Discriminates `AssertionFailed` (when actual + expected populated)
from `Panic` (otherwise). Frames render as repeated structured
fields — each consumer (LSP, CI annotation tool) decides layout.

### Slice 2 — `render_failure_text` walks Diagnostic

`render_failure_text(diag) -> String` produces the human-readable
text block the pre-arc-116 path produced inline. Same shape; data
flows through Diagnostic now.

`failure_frames_vec` helper replaces the pre-rendered string form
so the Diagnostic stores frames as repeated structured fields.

The pre-arc-116 `extract_failure(v) -> Option<String>` retired —
its callers now go via `failure_to_diagnostic` + `render_failure_text`
directly.

### Slice 3 — `WAT_TEST_OUTPUT` env var

`structured_output_format()` reads the env var.
`emit_structured_diagnostic(label, diag)` injects the test label as
the first field and writes one render_edn / render_json line to
stdout.

Three failure call sites in `run_tests_from_dir_with_loader` wired
through:
- assertion / panic with structured Failure
- runtime error
- test-body panic escape

### Slice 4 — test-discovery freeze failures

The per-file freeze loop's `Err(e) => { ... }` arm now also walks
`e.diagnostics()` (arc 115's StartupError method) and emits each
Diagnostic to stdout when WAT_TEST_OUTPUT is set. Text rendering
preserves the pre-arc-116 "test-runner: file: startup: <error>"
shape so cargo test users see no change.

### Slice 5 — closure (this slice)

INSCRIPTION + USER-GUIDE update + 058 row.

## What this arc closes

- **The flatten-to-string boundary.** Pre-arc-116, the substrate's
  structured Failure became a string at the test runner's panic
  boundary; tooling consumers parsed strings. Post-arc-116,
  consumers opt into structured emission and read fields directly.
- **The discovery-failure cliff.** Pre-arc-116, a `.wat` test file
  that fails to compile produced a flattened text error in cargo
  output. Post-arc-116, structured Diagnostics surface every
  TypeMismatch / CommCallOutOfPosition / migration hint —
  the same data shape `wat --check` produces.
- **The data-first principle gap.** Arc 115 minted the foundation;
  arc 116 propagates it through the test runner's last text-only
  surface.

## The four questions (final)

**Obvious?** Yes. Test failures are data; cargo's panic is one
renderer; CI's structured consumption is another. Same source.
The `WAT_TEST_OUTPUT` env var name reads naturally; the EDN/JSON
choice mirrors `wat --check --check-output`.

**Simple?** Yes. ~150 LOC for `failure_to_diagnostic` +
`render_failure_text` + `failure_frames_vec`. ~50 LOC for
`emit_structured_diagnostic` + env var reader. Six unit tests
exercising the data shape. The four-layer architecture has one
point of truth and three renderers — collapse-resistant.

**Honest?** Yes. The substrate's Failure already IS structured;
arc 116 stops flattening it. Cross-host chains stay as a known
limitation until arc 113 lands. Frames get distinct field names
(`frame_0`, `frame_1`, ...) rather than a list — keeping
DiagnosticValue minimal (String / Int).

**Good UX?** Phenomenal. Same `cargo test` invocation; humans see
the same readable output; agents pipe `WAT_TEST_OUTPUT=edn cargo
test` and consume records. The arc-064 actual/expected/location
data the user came to expect is preserved verbatim — just typed
now.

## Cross-references

- `docs/arc/2026/04/115-no-inner-colon-in-parametric-args/DESIGN.md`
  — the Diagnostic schema + render_edn/render_json arc 116 builds
  on; same `wat --check --check-output` pattern at the CLI surface.
- `docs/arc/2026/04/064-...` (or wherever) — arc 064's
  `:wat::kernel::Failure` shape with actual/expected/location/frames
  arc 116 surfaces via `Failure::diagnostic()`.
- `docs/arc/2026/04/105-spawn-error-as-data/INSCRIPTION.md` — arc
  105c's `ThreadDiedError::Panic { message, failure }` — the
  `failure` field carries the Failure arc 116 renders.
- `docs/arc/2026/04/113-cascading-runtime-errors/DESIGN.md` — the
  `Vec<ProgramDiedError>` chain that drops cleanly into arc 116's
  diagnostic-walking framework once landed.

## Queued follow-ups

- **Arc 113 chain rendering** — when `Vec<ProgramDiedError>`
  arrives, the test runner walks the chain and emits one
  Diagnostic per layer. The "FAILED at layer-3 because layer-2
  died because layer-1 died" picture becomes data; the renderer
  iterates.
- **GitHub Actions annotations** — render JSON failures into
  `::error file=...,line=...,col=...::message` comments. ~30 LOC
  consumer of the structured stream.
- **Editor LSP integration** — JSON consumer running as a cargo
  test subprocess; LSP server exposes failures inline at the
  source location. Out of scope for arc 116; the data foundation
  makes it possible.
- **Per-test-pass records** (if ever wanted) — extend
  `WAT_TEST_OUTPUT` to also emit `Pass` records when a deftest
  succeeds. Today's stream is failure-only; rationale: cargo test
  already handles the pass-tracking surface.
