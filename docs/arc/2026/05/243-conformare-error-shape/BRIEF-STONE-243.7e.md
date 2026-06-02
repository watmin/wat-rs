# BRIEF — Stone 243.7e — rolling-audit Group B (5 location-needing error types)

The five error types whose location must be *decided* per-type (unlike Group A's uniform per-variant-span reshape). Every decision is pinned below — apply them; do not re-derive.

## Read first (templates)
- `src/types/error.rs`, `src/check/error.rs` — shipped Pattern-A error types (`struct { span/location, kind } + enum XKind`, Display split).
- `docs/arc/2026/05/243-conformare-error-shape/SCORE-STONE-243.7d.md` — the Group-A batch + the cascade method + in-tool content gate.

## The five (each decision pinned)

### 1. `LexError` (src/lexer.rs) — reshape, location is `Position`
It already carries `Position` (line/col), a legitimate domain location per CONFORMARE. Reshape to `pub struct LexError { pub position: Position, pub kind: LexErrorKind }` + `pub enum LexErrorKind` (each variant's `Position` moves to the outer struct; data stays on the kind). Display split as usual (Kind = message; outer = prefixes the position). Use `position`, not `span` — Position is the honest location here.

### 2. `StdlibError` (src/stdlib.rs) — reshape, trivial
One variant (`ParseFailed { path, source }`), never fires in production (baked stdlib). `pub struct StdlibError { pub span: Span, pub kind: StdlibErrorKind }`; construct with outer `Span::unknown()` (no wat-source span — a baked-file parse failure); keep `path` + `source` on the kind. Display elides the unknown span.

### 3. `LoadError` (src/load.rs) — reshape, location is the `load!`-form span
`pub struct LoadError { pub span: Span, pub kind: LoadErrorKind }` + `pub enum LoadErrorKind`. Outer span = the load form's span: `resolve_loads`/`match_load_form`/`scan_for_setter` have `form: &WatAST` in scope → use `form.span().clone()`. For chain variants (`CycleDetected`, `DuplicateLoad`) use the triggering load-form's span (the form currently being processed). Keep all payload (path/cycle/reason/nested `err`) on the kind. Display split.

### 4. `ResolveError` (src/resolve.rs) — locate the ITEMS, not the collection
`ResolveError::UnresolvedReferences(Vec<UnresolvedReference>)` is a collection (like `CheckErrors`) — it does NOT get a single outer span. Instead, make each item located: **add `pub span: Span` to `struct UnresolvedReference`** (it currently has `path` + `context`). Populate it at the resolution site where the unresolved reference is detected (the reference's AST node is in scope there → `node.span().clone()`; if a site genuinely lacks the node, thread it or use `Span::unknown()` and note it). `ResolveError` stays a flat 1-variant enum wrapping the now-located Vec — diagnostic-complete because every ref carries its span.

### 5. `HashError` (src/hash.rs) — LEAVE the payload; locate via the WRAPPERS
HashError is a Rust-internal payload: returned only by `verify_source_hash`/`verify_ast_signature`/`verify_program_signature`, **always wrapped** (`RuntimeError::EvalVerificationFailed { err: HashError }` + `LoadError::VerificationFailed { err: HashError }`) — never tossed to wat directly. Zero-exceptions governs wat-tossable diagnostics; HashError is not one. So: **leave `HashError` as its flat enum** (no Pattern-A reshape, no span) and instead ensure its wrappers carry the location:
- `LoadError::VerificationFailed` — already gets the load-form span via #3 (it's a LoadError variant).
- `RuntimeError::EvalVerificationFailed` — currently constructs with outer `Span::unknown()` (the 243.7c freeze-pair default). Thread the real eval-digest/verify call-site span where it is constructed (grep its construction sites in `src/freeze.rs` — if a call span is in scope, use it; if genuinely none, leave `Span::unknown()` and note it).
Document HashError's wrapped-only status in the SCORE (affirmative scope: out of wat-tossable conformance because it is never tossed; located by its wrappers).

## Method
For the reshape cascades (#1/#2/#3) build one small Rust Cargo tool under `tools/<name>/` (parameterized by error-type), run per type, delete when done — `std::fs::read_to_string` → targeted `str::replace`/regex preserving all other bytes → `std::fs::write`; never rebuild a file char-by-char. The ResolveError field-add (#4) + the wrapper threading (#5) are small hand edits.

## Content gate — in the tool, in Rust
Before writing each file, the tool compares `original.chars().filter(|c| !c.is_ascii()).count()` to `rewritten.chars().filter(|c| !c.is_ascii()).count()`; if they differ, do not write — print the path + counts and stop. Report per-file before/after counts in the SCORE.

## Verify — ONE simple command per line (vanilla cargo/git/grep; no chains, loops, `<(...)`, multi-pipe)
- `cargo build --release -p wat`
- `cargo build --release --tests`
- `cargo test --release --lib -p wat`  (expect 895 / 0 / 1)
- `cargo clippy --release -p wat`  (read for result_large_err; box a large kind payload if it fires)
- `grep -c "pub struct LexError" src/lexer.rs`  (→ 1; one simple grep per reshaped type)
- `ls tools`  (gone)

Do NOT commit; leave the tree dirty. Write `docs/arc/2026/05/243-conformare-error-shape/SCORE-STONE-243.7e.md` (per-type decision + outcome, the HashError affirmative-scope note, per-file non-ASCII before/after). Final message: what you did per type, verify results verbatim, any blocker.
