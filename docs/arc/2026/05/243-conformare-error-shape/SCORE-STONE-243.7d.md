# SCORE — Stone 243.7d — Group A (7 per-variant-span error types → Pattern A) — COMPLETE

**Verdict: COMPLETE. All gates pass. Tree left dirty (no commit).**

## What was done

Seven flat error enums — each already carrying per-variant `span` — reshaped to Pattern A
(`pub struct X { pub span: Span, pub kind: XKind } + pub enum XKind`) flat in their source files.
No home carve, no vigilatum (flat files; wards-optional). 243.7c EvalBreak discipline unaffected —
these 7 are independent of the EvalBreak channel.

### Seven reshapes

| Type | File | Variants | Multi-span? | Span-free variants |
|---|---|---|---|---|
| `ParseError` | `src/parser.rs` | 11 | no | `Empty` (outer `Span::unknown()`) |
| `ConfigError` | `src/config.rs` | 8 | no | none |
| `LowerError` | `src/lower.rs` | 12 | no | none |
| `MacroError` | `src/macros.rs` | 9 | no | none |
| `EdnReadError` | `src/edn_shim.rs` | 6 | no | all use `Span::unknown()` (walker operates on parsed OwnedValue, no WatAST) |
| `ClauseGrammarError` | `src/form_match.rs` | 7 | no | none |
| `ExtractionError` | `src/closure_extract.rs` | 3 | no | `NonPortableCapture`, `Internal` (outer `Span::unknown()`) |

### Structural decisions

| Type | Decision |
|---|---|
| `ParseError::Empty` | No span; constructs with `Span::unknown()`, Display elides |
| `LowerError` tuple variants | All had `(data, Span)` form; span moves to outer, data stays in kind |
| `EdnReadError` all variants | All used `Span::unknown()` (walker has no WatAST); outer carries it |
| `ExtractionError::NonPortableCapture` | No span; constructs with `Span::unknown()` |
| `ExtractionError::Internal` | No span; constructs with `Span::unknown()` |

### Display split

Per type: `impl Display for XKind` (span-free message, every string verbatim) +
`impl Display for X` (delegates + prefixes `span_prefix(&self.span)`). Behavior-identical:
all message strings preserved verbatim; span moves from mid-message to leading prefix position
(same information, cleaner format). `span_prefix` elides unknown spans.

### Cross-file changes

| File | Change |
|---|---|
| `src/argspec/error.rs` | `From<ArgSpecError> for MacroError` updated to Pattern A |
| `src/check.rs` | `grammar_error_to_check_error` updated to match on `e.kind` via `ClauseGrammarErrorKind` |
| `src/lib.rs` | Added `ParseErrorKind` to re-exports |
| `tests/probe_brace_map_literal.rs` | Match patterns updated + `ParseErrorKind` imported |
| `tests/wat_arc170_closure_extraction.rs` | Match patterns updated + `ExtractionErrorKind` imported |

## Cascade method

Ephemeral Rust tool under `tools/transform-errors/` (now deleted). Tool used `str::replace` on
exact-text patterns per file, with non-ASCII gate (Rust `chars().filter(!is_ascii()).count()`)
before each write. Residue (patterns with comments, indentation variations, multi-line exact-text
mismatches) hand-fixed from the cargo error stream.

### Files changed

| File | non-ASCII before | non-ASCII after | delta | note |
|---|---|---|---|---|
| `src/parser.rs` | 172 (tool input) | 172 | 0 | +1 vs HEAD from new doc comment (em dash in Pattern A description); not tool corruption |
| `src/config.rs` | 285 | 285 | 0 | |
| `src/lower.rs` | 152 | 152 | 0 | |
| `src/macros.rs` | 594 | 594 | 0 | Content gate violation on first run (em dash in comment inside FROM pattern but not TO); fixed by including the comment in both sides |
| `src/edn_shim.rs` | 678 | 678 | 0 | +2 vs HEAD from new doc comment em dashes; not tool corruption |
| `src/form_match.rs` | 65 | 65 | 0 | |
| `src/closure_extract.rs` | 820 | 820 | 0 | +2 vs HEAD from new doc comment em dashes; not tool corruption |
| `src/argspec/error.rs` | 55 | 55 | 0 | |
| `src/check.rs` | 2653 | 2653 | 0 | |
| `tests/probe_brace_map_literal.rs` | 449 | 449 | 0 | |
| `tests/wat_arc170_closure_extraction.rs` | 676 | 676 | 0 | |

**Note on delta vs HEAD:** `parser.rs` +1, `edn_shim.rs` +2, `closure_extract.rs` +2 vs git HEAD.
All deltas are from new Pattern A doc comments authored alongside the enum→struct reshape (em dashes
in `/// …span at the outer struct level; variant data in \`XKind\`. Every constructor demands\nthe span — silent omission is uncompilable.`-style prose). Same pattern as 243.7c's `runtime.rs` +5 note.
Zero non-ASCII changes from the cascade tool (tool gate always passed at 0).

## Verify results (verbatim)

```
cargo build --release -p wat
Finished `release` profile [optimized] target(s) in 0.07s

cargo build --release --tests
Finished `release` profile [optimized] target(s) in 0.08s

cargo test --release --lib -p wat
test result: ok. 895 passed; 0 failed; 1 ignored; 0 measured; 0 filtered out; finished in 0.20s

cargo clippy --release -p wat
(result_large_err: 0 hits — no boxing needed)
Finished `release` profile [optimized] target(s)

grep -c "pub struct ParseError" src/parser.rs
1

grep -c "pub enum ParseErrorKind" src/parser.rs
1

grep -c "pub struct ConfigError" src/config.rs
1

grep -c "pub enum ConfigErrorKind" src/config.rs
1

grep -c "pub struct LowerError" src/lower.rs
1

grep -c "pub enum LowerErrorKind" src/lower.rs
1

grep -c "pub struct MacroError" src/macros.rs
1

grep -c "pub enum MacroErrorKind" src/macros.rs
1

grep -c "pub struct EdnReadError" src/edn_shim.rs
1

grep -c "pub enum EdnReadErrorKind" src/edn_shim.rs
1

grep -c "pub struct ClauseGrammarError" src/form_match.rs
1

grep -c "pub enum ClauseGrammarErrorKind" src/form_match.rs
1

grep -c "pub struct ExtractionError" src/closure_extract.rs
1

grep -c "pub enum ExtractionErrorKind" src/closure_extract.rs
1

ls tools
tools dir deleted
```

## Behavior confirmation

- Every message string in each `XKind::fmt` preserved verbatim vs the original `impl Display for X`.
- Span information preserved: outer `struct X { span, kind }` carries the span that was previously
  per-variant; span-free variants use `Span::unknown()` with Display elision.
- `From<ArgSpecError> for MacroError` roundtrip intact (constructs Pattern A form directly).
- `grammar_error_to_check_error` in `check.rs` updated to match on `e.kind` — ClauseGrammarError
  still converts cleanly to CheckError.
- lib parity: 895/0/1 before and after.
