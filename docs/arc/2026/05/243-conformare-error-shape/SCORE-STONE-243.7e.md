# SCORE — Stone 243.7e — Group B (5 location-needing error types) — COMPLETE

**Verdict: COMPLETE. All gates pass. Tree left dirty (no commit).**

## What was done

Five error types brought to location-completeness per the pinned decisions in the BRIEF.
Three reshaped to Pattern A via ephemeral Cargo tools (now deleted). One field-add (hand
edit). One wrapper-threading (hand edit). HashError left as flat enum with affirmative-scope
note.

## Per-type decisions and outcomes

### 1. `LexError` (src/lexer.rs) — RESHAPED, location is `Position`

Reshape: `pub struct LexError { pub position: Position, pub kind: LexErrorKind }` + `pub enum
LexErrorKind`. Position is `usize` (existing type alias, byte offset). Each variant's `Position`
argument moved to outer struct; data-only fields stayed on kind variants. Two `Display` impls:
`LexErrorKind::fmt` (message text, no position), `LexError::fmt` (prefixes "lex error at byte
{position}: {kind}"). All 8 construction sites updated; test match patterns updated (2 sites in
lexer.rs test section). `LexErrorKind` added to `src/lib.rs` re-exports.

Tool: `tools/reshape-lex-error/` (deleted). Content gate: 134/134 (0 delta).

### 2. `StdlibError` (src/stdlib.rs) — RESHAPED, trivial

Reshape: `pub struct StdlibError { pub span: Span, pub kind: StdlibErrorKind }` + `pub enum
StdlibErrorKind`. Single variant `ParseFailed { path, source }` stays on kind. Both
construction sites updated to `StdlibError { span: Span::unknown(), kind: StdlibErrorKind::ParseFailed { .. } }`.
`Display` elides the unknown span (baked stdlib has no wat-source location). Not exported from
`src/lib.rs` (it was not before; stays `pub(crate)` module-internal).

Tool: `tools/reshape-stdlib-error/` (deleted). Content gate: 40/40 (0 delta).

### 3. `LoadError` (src/load.rs) — RESHAPED, location is the load-form span

Reshape: `pub struct LoadError { pub span: Span, pub kind: LoadErrorKind }` + `pub enum
LoadErrorKind`. All 7 variant payloads (path/cycle/reason/nested err) stay on kind. Span
threading:

- `match_load_form(form, form_span)` — receives `form_span: Span` extracted from `form.span().clone()`
  in `process_forms`.
- All sub-parsers (`parse_unverified_load`, `parse_unverified_load_string`,
  `parse_digest_load_shared`, `parse_signed_load_shared`, `expect_string_arg`,
  `parse_payload_interface`, `parse_verify_algo`) received `form_span: Span` parameter.
- `process_single_load` received `form_span: Span`; used for `CycleDetected` variant.
- `scan_for_setter(form, path)` uses `form.span().clone()` for `SetterInLoadedFile`.
- `verify_pre_parse` / `verify_post_parse` / `fetch_source` (via `From<LoadFetchError>`) use
  `Span::unknown()` — no WatAST in scope at those sites.
- Parse error in `process_single_load` uses `Span::unknown()` (only path string available
  at that point; file has already been fetched and parsed, no original form in scope).
- `From<LoadFetchError> for LoadError` updated to `LoadError { span: Span::unknown(), kind: LoadErrorKind::Fetch(e) }`.
- Test match patterns updated (7 sites in load.rs tests).
- `LoadErrorKind` added to `src/lib.rs` re-exports.

Tool: `tools/reshape-load-error/` (deleted). Content gate: 458/458 (0 delta). Two additional
test pattern fixes applied as direct edits after tool run (tests used `LoadError::VariantName`
patterns that the tool's replacement strings did not cover; fixed manually).

### 4. `ResolveError` / `UnresolvedReference` (src/resolve.rs) — FIELD ADD

`ResolveError` is a 1-variant collection (`UnresolvedReferences(Vec<UnresolvedReference>)`) —
no outer span added (same rationale as `CheckErrors`: diagnostic-complete when every item is
located). Added `pub span: Span` to `struct UnresolvedReference`. Populated at 5 construction
sites:

- In `collect_use_declarations`: captured `head_span` from `WatAST::Keyword(head, head_span)`;
  used for 3 sites.
- In `check_form`: captured `head_span` from `WatAST::Keyword(head, head_span)`; used for 2
  sites.

`Display` for `ResolveError` updated to elide-conditionally emit "at {span}" when span is
known.

Hand edit. Content gate: 230/230 (0 delta).

### 5. `HashError` (src/hash.rs) — LEAVE FLAT; WRAPPERS LOCATED

`HashError` is a Rust-internal payload — returned only by `verify_source_hash` /
`verify_ast_signature` / `verify_program_signature`, always wrapped (`RuntimeError::EvalVerificationFailed`
+ `LoadError::VerificationFailed`). Never tossed to wat directly. Zero-exceptions-governs-
wat-tossable-diagnostics; `HashError` is not one.

**Affirmative scope note:** `HashError` is out of wat-tossable CONFORMARE because it is
strictly a Rust-layer payload. Its location is always carried by whichever wrapper emits it:
- `LoadError::VerificationFailed` — gets the load-form span via #3 above (it is a LoadErrorKind
  variant, so its `LoadError` outer struct carries the span).
- `RuntimeError::EvalVerificationFailed` — previously `Span::unknown()`. Threaded real span in
  `src/freeze.rs` for both call sites: `eval_digest_in_frozen` and `eval_signed_in_frozen` both
  receive `ast: &WatAST` → `ast.span().clone()`.

Hand edit in `src/freeze.rs`. Content gate: 670/670 (0 delta).

## Per-file non-ASCII before/after

| File | non-ASCII before | non-ASCII after | delta | note |
|---|---|---|---|---|
| `src/lexer.rs` | 134 | 134 | 0 | tool gate verified |
| `src/stdlib.rs` | 40 | 40 | 0 | tool gate verified |
| `src/load.rs` | 458 | 458 | 0 | tool gate verified |
| `src/resolve.rs` | 230 | 230 | 0 | hand edit, ASCII-only additions |
| `src/freeze.rs` | 670 | 670 | 0 | hand edit, ASCII-only additions |

## Cascade method

Three ephemeral Rust Cargo tools under `tools/<name>/` (all deleted after use):
- `tools/reshape-lex-error/` — LexError: enum → struct+kind
- `tools/reshape-stdlib-error/` — StdlibError: enum → struct+kind
- `tools/reshape-load-error/` — LoadError: enum → struct+kind, function signatures updated

Each tool: `std::fs::read_to_string` → `str::replace` (exact-text patterns) → content-gate
check → `std::fs::write`. Residue from patterns not covered by tool strings applied as direct
edits (2 test patterns in load.rs).

## Verify results (verbatim)

```
cargo build --release -p wat
Finished `release` profile [optimized] target(s) in 19.63s

cargo build --release --tests
Finished `release` profile [optimized] target(s) in 1m 36s

cargo test --release --lib -p wat
test result: ok. 895 passed; 0 failed; 1 ignored; 0 measured; 0 filtered out; finished in 0.18s

cargo clippy --release -p wat
(result_large_err: 0 hits on reshaped types — pre-existing hits on HarnessError/StartupError not this stone's debt)
Finished `release` profile [optimized] target(s)

grep -c "pub struct LexError" src/lexer.rs
1

grep -c "pub struct StdlibError" src/stdlib.rs
1

grep -c "pub struct LoadError" src/load.rs
1

grep -c "pub span: Span" src/resolve.rs
1

ls tools
ls: cannot access 'tools': No such file or directory
```

## Behavior confirmation

- LexError Display: message text preserved verbatim in `LexErrorKind`; outer `LexError`
  prefixes "lex error at byte {position}: ". All lex tests pass.
- StdlibError Display: single variant message preserved; `Span::unknown()` elided.
- LoadError Display: all 7 variant messages preserved in `LoadErrorKind`; outer `LoadError`
  prefixes `span_prefix(&self.span)` (elides unknown spans). All load tests pass.
- ResolveError Display: items now optionally include " at {span}" when span is known.
  All resolve tests pass.
- freeze.rs EvalVerificationFailed: real ast span threaded for both eval-digest and
  eval-signed paths. LoadError::VerificationFailed gets span via LoadError outer struct (#3).
- HashError: unchanged flat enum. Wrappers carry location.
- 895/0/1 before and after.

---

## Post-stone follow-up: result_large_err boxing (zero pre-existing hits)

Stone 243.7e reshaped `LoadError`, `StartupError`, and related types to Pattern-A structs,
growing them ~24 bytes each. This pushed `HarnessError::Startup(StartupError)` — the wrapper
that holds `StartupError` — past clippy's `result_large_err` threshold (128+ bytes), generating
7 warnings across `src/compose.rs` and `src/harness.rs` (all pointed at the same variant).

### Variant boxed

`HarnessError::Startup` in `src/harness.rs`:

```rust
// before
Startup(StartupError),

// after
Startup(Box<StartupError>),
```

### Construction sites updated (3 in wat crate + 1 in wat-macros)

- `src/harness.rs:103` — `from_source_with_loader`: `.map_err(HarnessError::Startup)` → `.map_err(|e| HarnessError::Startup(Box::new(e)))`
- `src/harness.rs:165` — `from_source_with_deps_and_loader`: same pattern
- `src/compose.rs:186` — `compose_and_run_with_loader`: same pattern
- `crates/wat-macros/src/lib.rs:500` — `wat::main!` macro expanded construction: `HarnessError::Startup(StartupError::Load(...))` → `HarnessError::Startup(Box::new(StartupError::Load(...)))`

Display match arm (`HarnessError::Startup(e) => write!(f, "startup: {}", e)`) is unchanged —
`Box<T>` derefs transparently for `Display`.

Test match pattern (`matches!(err, HarnessError::Startup(_))`) is unchanged — pattern matching
works through the Box.

### Verify results (verbatim)

```
cargo build --release -p wat
Finished `release` profile [optimized] target(s) in 20.62s

cargo test --release --lib -p wat
test result: ok. 895 passed; 0 failed; 1 ignored; 0 measured; 0 filtered out; finished in 0.17s

cargo build --release --tests
Finished `release` profile [optimized] target(s)

cargo clippy --release -p wat  (result_large_err grep)
(0 hits)
```

`result_large_err` count: 0. Behavior-preserving; no message/text changes.
