# BRIEF — Stone 243.7d — rolling-audit Group A (7 per-variant-span error types → Pattern A)

Convert seven flat error enums — each already carrying a per-variant `span` — to Pattern A, the same shape shipped for TypeError (243.3), CheckError (243.6a), and RuntimeError (243.7c). This is mechanical application of a proven pattern at small scale, ×7.

## The seven (each: `pub enum X` → `pub struct X { span, kind } + pub enum XKind`)

| Type | File | variants | sites |
|---|---|---|---|
| `ParseError` | `src/parser.rs` | 11 | 47 |
| `ConfigError` | `src/config.rs` | 8 | 47 |
| `LowerError` | `src/lower.rs` | 12 | 45 |
| `MacroError` | `src/macros.rs` | 9 | 44 |
| `EdnReadError` | `src/edn_shim.rs` | 6 | 31 |
| `ClauseGrammarError` | `src/form_match.rs` | 7 | 21 |
| `ExtractionError` | `src/closure_extract.rs` | 3 | 13 |

## Read first (the proven templates — mirror them)
- `src/types/error.rs` and `src/check/error.rs` — shipped Pattern-A error types: `struct { span, kind }`, `enum XKind` (variants span-free), Display split (Kind = span-free message; outer = delegates + prefixes span via a `span_prefix` helper, eliding unknown).
- `docs/arc/2026/05/243-conformare-error-shape/SCORE-STONE-243.7c.md` — the RuntimeError reshape just shipped; the cascade method + content-integrity discipline.

## The transform (per type, identical shape)
1. `pub struct X { pub span: Span, pub kind: XKind }` + `pub enum XKind` (every variant's `span` field moves to the outer struct).
2. **Multi-span variants** (a variant carrying two spans): outer `span` = the most-actionable location; the secondary span stays as a domain-named field on the kind variant (read the variant's Display message to choose which span the user edits to fix). Mirror the CheckError/RuntimeError multi-span contract.
3. **Any variant with no span** (e.g. one `ExtractionError` variant): kind variant has no span; construct with outer `Span::unknown()` (Display elides it via `span_prefix`).
4. Split Display: `impl Display for XKind` (span-free, per-variant message, every string verbatim) + `impl Display for X` (delegates to kind, prefixes `span_prefix(&self.span)`, elides unknown). Keep any structured/EDN path reading `self.span` once.
5. Preserve every payload field on the kind variant. Behavior-identical — no message rewrites.

## How to run the cascade
For each type's ~13–47 construction/match sites, build one small **generalized Rust Cargo tool** under `tools/<name>/` (parameterized by the error-type name) that does the construction-site rewrites, run it per type, then delete it when done. The tool reads each file with `std::fs::read_to_string`, performs targeted replacements that change only the construction-site text (`str::replace` on exact patterns, or a regex whose replacement preserves all surrounding bytes), and writes with `std::fs::write`. It does not rebuild a file character-by-character and does not modify any line without a construction site. Match-site destructuring + residue: hand-fix from the cargo error stream.

## The gate that matters most (per file)
The transform changes only ASCII syntax, so every file's non-ASCII character count must be identical before and after. For each file you change, confirm:
`grep -oP '[^\x00-\x7F]' <file> | wc -l` equals `git show HEAD:<file> | grep -oP '[^\x00-\x7F]' | wc -l`. If any file's count changes, the tool altered a character it shouldn't have — `git checkout HEAD -- <file>`, fix the tool, re-run. Report the per-file before/after non-ASCII counts in the SCORE.

## Verify (report verbatim)
- Per type: `grep -c "pub struct <X>" <file>` → 1 and `grep -c "pub enum <X>Kind" <file>` → 1.
- `grep -rnE "''" src/ crates/ | grep -v '""'` → empty (no dropped-char empty literals).
- `cargo build --release -p wat` → clean; `cargo build --release --tests` → clean.
- `cargo test --release --lib -p wat` → 895/0/1 (parity — behavior-preserving).
- `cargo clippy --release -p wat 2>&1 | grep -c result_large_err` → 0 (if it fires on a reshaped type, box the large kind payload; mirror 243.7a).
- Every touched file's non-ASCII count matches its `git show HEAD:` count.
- `ls tools/` → the tool is gone.

## Scope
`src/parser.rs`, `src/config.rs`, `src/lower.rs`, `src/macros.rs`, `src/edn_shim.rs`, `src/form_match.rs`, `src/closure_extract.rs` + the cross-file construction/match sites cargo names + the `tools/<name>/` tool (deleted before finishing). Flat files → no home carve, no vigilatum (these are wards-optional). Do NOT commit; leave the tree dirty. These 7 are independent of each other and of the EvalBreak channel — no signal concerns.

## Deliverable
Write `docs/arc/2026/05/243-conformare-error-shape/SCORE-STONE-243.7d.md`: per-type (struct+kind minted, multi-span decisions, site count), the per-file non-ASCII before/after table (proving zero corruption), lib parity, clippy, behavior-identical confirmation. Final message: what you did per type, verify results verbatim, any STOP.

## Calibration
90–180 min Mode A. Seven small reshapes, the proven pattern; the generalized tool does the bulk. Cite `SCORE-STONE-243.7c.md` for the cascade shape.
