# EXPECTATIONS — Stone 234.4.match

Mode A: 11/11 PASS.

## Scorecard

| # | Row | Verification | Expected |
|---|---|---|---|
| 1 | Compile clean | `cargo build --release -p wat 2>&1 \| tail -5` | 0 errors |
| 2 | **NEW probe 6/6 PASS** (LOAD-BEARING) | `cargo test --release --test probe_arc234_stone4_match_hash_destructure 2>&1 \| tail -3` | `6 passed; 0 failed` |
| 3 | 234.4 let-binding probe regression | `cargo test --release --test probe_arc234_stone4_hash_destructure 2>&1 \| tail -3` | `6 passed; 0 failed` |
| 4 | 234.3c regression | `cargo test --release --test probe_arc234_stone3c_keyword_accessor 2>&1 \| tail -3` | `6 passed; 0 failed` |
| 5 | 234.3b regression | `cargo test --release --test probe_arc234_stone3b_record_assoc 2>&1 \| tail -3` | `6 passed; 0 failed` |
| 6 | 234.3a regression | `cargo test --release --test probe_arc234_stone3a_record_read_verbs 2>&1 \| tail -3` | `6 passed; 0 failed` |
| 7 | 236.0 CheckResult probe regression | `cargo test --release --test probe_arc236_stone0_check_result 2>&1 \| tail -3` | `6 passed; 0 failed` |
| 8 | **Lib baseline** (LOAD-BEARING) | `cargo test --release --lib -p wat --no-fail-fast 2>&1 \| tail -3` | `827 passed; 0 failed` |
| 9 | 232.0a regression | `cargo test --release --test probe_diagnostic_typed_entities_reflection 2>&1 \| tail -3` | `7 passed; 0 failed` |
| 10 | 233.3 errors-as-EDN regression | `cargo test --release --test probe_stone_233_3_runtime_error_edn 2>&1 \| tail -3` | `5 passed; 0 failed` |
| 11 | Clippy | `cargo clippy --release --lib -p wat -- -D warnings 2>&1 \| grep -c "warning"` | ≤ 54 |

## Prediction

**Target:** 60-90 min Mode A. **Upper:** 120 min (STOP-3).

Surface:
- **Parser:** 0-30 lines (likely zero-touch; Stone 234.4's `BraceKind::HashDestructure` recognition may already fire in match position; verify probe-first per BRIEF Step 1)
- **check.rs `infer_match`:** ~30-50 lines (mirror process_let_binding hash-destructure detection in match-StructPattern arm)
- **runtime.rs `try_match_pattern`:** ~60-120 lines (3-receiver dispatch + conditional bind; reuse Stone 234.4 helpers)
- **runtime.rs `try_match_pattern_ast`:** 0-30 lines (verify whether parallel update needed; document either way)
- **New probe file:** ~150-200 lines (6 contracts; mirror Stone 234.4 probe structure)

Net: ~300-400 lines across 3-4 file touches.

Cascade depth: 1-2 compile rounds expected. Extension is localized; should not ripple beyond match-arm pattern infrastructure.

Risks:
- Parser may need extension (Step 1 verification first; small if needed)
- AST-level mirror `try_match_pattern_ast` parity (T9; verify; parallel update if needed)
- arc-169 struct-destructure preservation (T7; ensure all-Symbol StructPattern path unchanged)
- HashMap missing-key semantic decision (D3 locks Option<V> per let-binding parity)

Pre-emption evidence (rank-up vs prior arc 234 stones):
- Stone 234.4 (let-binding) shipped ~90 min (Mode A target 90-120)
- Stone 234.4.match (match-arm) target tighter (60-90) because infrastructure exists; helpers reusable; probe-first verifies parser zero-touch
- Stone 236.3 (sum-type refactor) shipped ~6 min — small parity stones with clear infrastructure ship fast

## Out-of-scope (REJECTED)

- Nested-pattern hash-destructure (T6; future feature; not currently planned)
- New `:wat::*` verbs (D7; the match-arm shape IS the surface)
- Transitional macro / aliasing for the match-arm form (D8 HARD CUT)
- Touch any file other than the 3 substrate files + the new probe file (STOP-5)
- holon-rs touched (STOP-4)
- Stone 234.4.match's own type-system improvements beyond polymorphic-T (per-class TypeDef is arc 232.1 future-lift; D4 locks polymorphic-T per Stone 234.4 D4)

## SCORE

`SCORE-STONE-234.4.match.md` (NEW). Capture:
- 11-row scorecard verbatim
- Receivers shipped (3)
- Three-file change summary (line counts per file)
- Implementation notes:
  - Parser zero-touch verified? (Step 1 result)
  - AST mirror `try_match_pattern_ast` parity needed? (T9 verdict)
  - Empty-pattern `{}` decision (T5; recommend ALLOW)
  - arc-169 struct-destructure preservation verified
- Cascade depth: compile rounds + iteration cycles
- Honest deltas
- Rank-up evidence — predecessor SCORE template effectiveness; probe-first parser verification payoff
- Closing note: arc 234 named follow-up from Stone 234.4 D8 CLOSED. Arc 234 is now one decision (234.6 fate) + one stone (234.7 INSCRIPTION) from closure.
