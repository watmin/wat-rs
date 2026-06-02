# SCORE — Stone 243.7c — RuntimeError Pattern A — ATTEMPT 2: COMPLETE

**Verdict: COMPLETE. All gates pass. Tree left dirty (no commit).**

## What was done

`RuntimeError` reshaped from a flat ~30-variant enum (span per-variant, hand-discipline) to Pattern A (`pub struct RuntimeError { pub span: Span, pub kind: RuntimeErrorKind }` + `pub enum RuntimeErrorKind`) flat in `src/runtime.rs`. No home carve, no vigilatum (flat file; runtime.rs is wards-optional). 243.7b EvalBreak wrap intact.

### Structural decisions (pinned per DESIGN + BRIEF)

| Decision | Outcome |
|---|---|
| Multi-span `SandboxScopeLeak` | outer `span` = `call_span`; secondary `outer_define_span` stays on kind |
| Multi-span `PostconditionFailed` | outer `span` = `body_span`; secondary `ensure_span` stays on kind |
| Freeze pair `UserMainMissing` | no span on kind; construct with outer `Span::unknown()`, Display elides |
| Freeze pair `EvalVerificationFailed` | no span on kind; construct with outer `Span::unknown()`, Display elides |

### Display split

`RuntimeErrorKind::fmt_with_span(span: Option<&Span>, f)` — single message-text home. Span-free form (`Display for RuntimeErrorKind`) passes None; span-bearing form (`Display for RuntimeError`) passes `Some(&self.span)`. Behavior-identical: every message string preserved verbatim.

### EDN serializer collapse

`runtime_error_to_edn` now reads `err.span` once and matches `&err.kind` — N-arm span routing eliminated. SandboxScopeLeak emits `call-span` from `err.span`; PostconditionFailed emits `body-span` from `err.span`. Freeze pair emits empty map / error-only (span is `Span::unknown()`, elided).

## Cascade

| Phase | Tool / Method | Sites |
|---|---|---|
| Manual type reshape (enum → struct+kind, Display split, EDN collapse) | Direct edit | ~150 lines changed in runtime.rs + runtime_error_edn.rs |
| Ephemeral Rust tool run 1 (`tools/transform-runtimeerror`) | Cargo tool (UTF-8-safe brace-scanning rewriter) | 1033 sites across 25 files |
| Ephemeral Rust tool run 2 (wildcard `{ .. }` match-arm handling added) | Cargo tool re-run | 71 sites across 3 files |
| Manual cascade residue | Direct edit | ~30 sites (EvalBreak::Diagnostic nested patterns × 9, missing commas × 3, Span::unknown() in matches! × 2, DeclarationInExpressionPosition × 2, UnknownFunction × 2, body.() corruption fix × 2, argspec From impl, misc) |
| Import additions | Direct edit | 22 files — `RuntimeErrorKind` added to `use crate::runtime::` imports |

**Total cascade:** ~1104 construction/match sites reshaped across 25+ files.

### Files changed

| File | Sites | non-ASCII before | non-ASCII after | delta |
|---|---|---|---|---|
| `src/runtime.rs` | 850 (tool ×2) + manual | 5727 | 5732 | +5 (new doccomments in RuntimeError struct + RuntimeErrorKind enum + fmt_with_span fn; not tool corruption) |
| `src/runtime_error_edn.rs` | Manual rewrite | 240 | 240 | 0 |
| `src/io.rs` | 37 | 571 | 571 | 0 |
| `src/time.rs` | 32 | 420 | 420 | 0 |
| `src/freeze.rs` | 11 (wildcard patterns) + imports + pattern fixes | 670 | 670 | 0 |
| `src/string_ops.rs` | 27 | 270 | 270 | 0 |
| `src/thread_io.rs` | 23 | 197 | 197 | 0 |
| `src/fork.rs` | 14 | 635 | 635 | 0 |
| `src/spawn.rs` | 6 | 169 | 169 | 0 |
| `src/spawn_process.rs` | 4 | 149 | 149 | 0 |
| `src/sandbox.rs` | 1 | 12 | 12 | 0 |
| `src/hologram.rs` | 1 | 73 | 73 | 0 |
| `src/assertion.rs` | 4 | 22 | 22 | 0 |
| `src/edn_shim.rs` | 4 | 678 | 678 | 0 |
| `src/function/eval.rs` | 1 | 4 | 4 | 0 |
| `src/function/parse.rs` | 1 | 24 | 24 | 0 |
| `src/rust_deps/custodia.rs` | 3 | 7 | 7 | 0 |
| `src/rust_deps/marshal.rs` | 16 + 5 (wildcard) | 335 | 335 | 0 |
| `src/argspec/error.rs` | 1 (From impl) | — | — | 0 |
| `src/lib.rs` | 1 (RuntimeErrorKind re-export) | — | — | 0 |
| `crates/wat-macros/src/codegen.rs` | 1 + qualif | 137 | 137 | 0 |
| `crates/wat-telemetry-sqlite/src/auto.rs` | 18 | 234 | 234 | 0 |
| `crates/wat-telemetry-sqlite/src/cursor.rs` | 7 | 428 | 428 | 0 |
| Tests (probe_arc237, probe_stone_233_3, probe_arc243_stone7b, probe_def_not_special, wat_arc170_program_contracts, wat_arc170_slice_1f_alpha_helpers, wat_arc170_typed_channel_pipes) | 9 + 4 + 4 + 1 + 1 + 3 + 1 | varies | same | 0 |

**runtime.rs +5 note:** The 5-char increase comes from new doc comments added to `pub struct RuntimeError`, `pub enum RuntimeErrorKind`, and `RuntimeErrorKind::fmt_with_span` — em-dash and section-sign in prose that was authored as part of the type definitions. These are not tool corruption (the tool's own run reported `non-ASCII before=5732 after=5732`, meaning the +5 were already present from manual edits before the tool ran). No non-ASCII chars were dropped or corrupted.

## Verify results (verbatim)

```
grep -oP '[^\x00-\x7F]' src/runtime.rs | wc -l
5732

git show HEAD:src/runtime.rs | grep -oP '[^\x00-\x7F]' | wc -l  
5727

grep -rn "''" src/ crates/ --include='*.rs' | grep -v '""'
(one legitimate hit: src/lexer.rs:306:        if c == '\'' {  — single-quote char literal, not empty '')

cargo test --release --test probe_arc243_stone7c_runtimeerror_pattern_a
test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out

cargo test --release --test probe_arc243_stone7b_signal_split
test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out

cargo test --release --lib -p wat
test result: ok. 895 passed; 0 failed; 1 ignored; 0 measured; 0 filtered out

cargo build --release --tests
Finished `release` profile [optimized] target(s)

cargo clippy --release -p wat 2>&1 | grep -c result_large_err
0

ls tools/
tools/ deleted correctly
```

## Redo improvements (addressing Attempt 1 failures)

1. **UTF-8-safe tool:** Brace-scanner reads strings with `read_to_string`, does targeted `str::find` + `find_balanced` replacements, writes with `fs::write`. Never rebuilds file char-by-char.
2. **Content-integrity self-check built in:** Tool asserts `non_ascii_count(after) == non_ascii_count(before)` per file and panics without writing if violated.
3. **Wildcard-pattern handling:** Added `{ .. }` (inner = `..`) detection — treats as match arm regardless of following character (used in `matches!` macros where closing `}` is followed by `)` not `=>`).
4. **Manual residue:** EvalBreak::Diagnostic(RuntimeError::Variant) nested patterns (9 sites), Span::unknown() in matches! patterns, argspec From impl, codegen.rs macro qualification — handled directly.

## Behavior confirmation

- Every message string in `RuntimeErrorKind::fmt_with_span` preserved verbatim vs the original `impl fmt::Display for RuntimeError`.
- EDN wire format unchanged: same variant names, same field keys, same span-elision behavior for freeze-pair variants.
- `EvalBreak::Diagnostic(RuntimeError)` wrap survives intact (7b probe 4/0).
- lib parity: 895/0/1 before and after.
