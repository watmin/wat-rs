# EXPECTATIONS — Stone 234.6

Mode A: 11/11 PASS.

## Scorecard

| # | Row | Verification | Expected |
|---|---|---|---|
| 1 | Compile clean | `cargo build --release -p wat 2>&1 \| tail -5` | 0 errors |
| 2 | **Defensive grep: 0 callers of OLD macro** (LOAD-BEARING) | `grep -rn ":wat::holon::defrecord" src/ wat/ wat-tests/ tests/ crates/ examples/ \| wc -l` | `0` |
| 3 | OLD macro source DELETED | `test ! -f wat/holon/defrecord.wat && echo "DELETED" \|\| echo "PRESENT"` | `DELETED` |
| 4 | arc 227 probe regression | `cargo test --release --test probe_arc227_stone2_defrecord 2>&1 \| tail -3` | `N passed; 0 failed` (where N is the probe's current contract count; likely 28-30) |
| 5 | 234.4 let-binding regression | `cargo test --release --test probe_arc234_stone4_hash_destructure 2>&1 \| tail -3` | `6 passed; 0 failed` |
| 6 | 234.4.match regression | `cargo test --release --test probe_arc234_stone4_match_hash_destructure 2>&1 \| tail -3` | `6 passed; 0 failed` |
| 7 | 234.2b regression | `cargo test --release --test probe_arc234_stone2b_defrecord_macro 2>&1 \| tail -3` | `N passed; 0 failed` (current contract count) |
| 8 | **Lib baseline** (LOAD-BEARING) | `cargo test --release --lib -p wat --no-fail-fast 2>&1 \| tail -3` | `827 passed; 0 failed` |
| 9 | 232.0a regression | `cargo test --release --test probe_diagnostic_typed_entities_reflection 2>&1 \| tail -3` | `7 passed; 0 failed` |
| 10 | 236.0 regression | `cargo test --release --test probe_arc236_stone0_check_result 2>&1 \| tail -3` | `6 passed; 0 failed` |
| 11 | Clippy | `cargo clippy --release --lib -p wat -- -D warnings 2>&1 \| grep -c "warning"` | ≤ 54 |

**Note on Row 4:** arc 227 probe is the heaviest caller (56 references). After find-replace, the probe should pass UNCHANGED in test assertions (T1; per Stone 234.5 auto-dispatch semantic equivalence). If it FAILS, investigate per T1 protocol — test-body adjustment may be acceptable if traceable to macro shape change; UNJUSTIFIED test changes are STOP-11.

**Note on Row 2:** Defensive grep is LOAD-BEARING because it proves the HARD CUT landed structurally. ANY non-zero count means a caller was missed OR a comment/string still references the OLD name (acceptable if in INSCRIPTION/SCORE historical context; review with orchestrator).

## Prediction

**Target:** 60-90 min Mode A. **Upper:** 120 min (STOP-3).

Surface:
- Find-replace: 7 files; ~69 references (75 total minus 6 in deleted wat/holon/defrecord.wat); mechanical
- D12 comment update: 1 line in wat/Record.wat
- File delete: 1 (`wat/holon/defrecord.wat`)
- Registry retirement: 2 sites in `src/stdlib.rs`
- Optional loader file-list update (T9; if discovered)
- Probe docstring updates (T5; ~2-3 file headers)

Net: ~150-200 line touch across ~10 files (find-replace + small surgical edits).

Cascade depth: 1-2 compile rounds expected. Substrate-as-teacher cascade fires at step 6 (`cargo build` after registry retirement) if any caller missed.

Risks:
- T1 (arc 227 probe behavior preservation) — most likely surprise; investigate per T1 protocol
- T9 (file-list loader) — may or may not exist; sonnet checks
- T6 (cross-probe regression) — full test suite is the safety net

Pre-emption evidence (rank-up vs predecessor stones):
- Stone 236.2 (sibling-flip sweep; 47 fns; cascade depth 1; ~57 min) — comparable mechanical-sweep precedent
- Stone 234.4.match (parity stone; ~16 min) — recent arc 234 ship; tight discipline + probe-first verification
- Substrate-as-teacher cascade pattern: step 6 (`cargo build` after retirement) IS the cascade; sonnet iterates if cascade surfaces missed callers
- Probe behavior preservation is the only non-mechanical risk; auto-dispatch (Stone 234.5) provides strong semantic-equivalence guarantee

## Out-of-scope (REJECTED)

- Transitional alias / deprecation form / "defrecord-deprecated" macro (D2 HARD CUT)
- Preserving `wat/holon/defrecord.wat` as deprecated-stub (D2 — full delete)
- Touch any file outside the migration scope (D6 — strict file list)
- holon-rs touched (STOP-4)
- Lab repos touched (D3 workspace boundary; lab-repo migration is lab-repo work)
- Probe arc 227 test ASSERTION modifications beyond what's required by macro shape change (STOP-11)
- Renaming the arc 227 probe file (T5 — git history preservation > naming cleanliness)
- New `:wat::Record::def` features beyond what Stone 234.2b shipped (out of scope; arc 235 PROPOSED for richer features)

## SCORE

`SCORE-STONE-234.6.md` (NEW). Capture:
- 11-row scorecard verbatim outputs
- File-by-file migration summary (references before/after per file)
- D12 comment update verbatim (before/after)
- Registry retirement sites (which lines removed in src/stdlib.rs)
- T1 outcome: arc 227 probe passed on first try? OR test-body adjustment surfaced?
- T9 outcome: loader file-list needed update?
- Defensive grep result post-stone
- Cascade depth: compile rounds; iterations if any
- Honest deltas
- Rank-up evidence — Stone 236.2 cascade precedent + Stone 234.4.match parity-stone discipline effective?

Closing note: **Arc 234 substrate work COMPLETE.** Stone 234.7 INSCRIPTION + arc 234 closure is the next move.
