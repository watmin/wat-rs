# EXPECTATIONS — Arc 224 Stone 224.5 — Group A L1 fixes

Mode A target: **15/15 PASS**. Every row binds to a specific verification command. No row marked PASS without naming the verification.

## Scorecard

| # | Row | Binding verification | Expected |
|---|---|---|---|
| 1 | Compile clean | `cargo build --release -p wat 2>&1 \| tail -5` | 0 errors |
| 2 | Lib tests match baseline | `cargo test --release --lib -p wat --no-fail-fast 2>&1 \| tail -5` | `827 passed; 0 failed; 1 ignored` |
| 3 | Clippy no new warnings on src/ | `cargo clippy --release --lib -p wat -- -D warnings 2>&1 \| grep -c "warning"` (compare to pre-stone baseline) | Same count or fewer |
| 4 | L1-runtime-2 fix #1 — type_name Sender | `sed -n '1105p' src/runtime.rs` | Returns `"wat::kernel::Sender"` |
| 5 | L1-runtime-2 fix #2 — type_name Receiver | `sed -n '1106p' src/runtime.rs` | Returns `"wat::kernel::Receiver"` |
| 6 | L1-runtime-2 fix #3 — 5 expected-strings updated | `grep -c 'expected: "rust::crossbeam_channel::' src/runtime.rs` | 0 hits |
| 7 | L1-runtime-2 fix #4 — new expected-strings present | `grep -c 'expected: "wat::kernel::' src/runtime.rs` | ≥ 5 hits |
| 8 | L1-check-A fix #1 — function rename | `grep -c "fn sender_kind_in_type" src/check.rs` | 1 hit |
| 9 | L1-check-A fix #2 — old name purged | `grep -c "type_contains_sender_kind" src/check.rs` | 0 hits |
| 10 | L1-check-A fix #3 — all callers updated | `grep -c "sender_kind_in_type(" src/check.rs` | ≥ 8 hits (was 8 callers of old name) |
| 11 | L1-check-B fix — QueueSender/QueuePair doc vocab purged | `grep -cE "QueueSender\|QueuePair" src/check.rs` | 0 hits |
| 12 | L1-check-C fix #1 — closure rename | `grep -c "let keyword_ty" src/check.rs` | 1 hit |
| 13 | L1-check-C fix #2 — old name purged | `grep -c "symbol_ty" src/check.rs` | 0 hits |
| 14 | L1-check-C fix #3 — citation sites updated | `grep -c "keyword_ty()" src/check.rs` | ≥ 4 hits |
| 15 | Holon-rs untouched | `git -C /home/watmin/work/holon/holon-rs/ status --short` | empty output |

## Independent prediction (calibration record)

**Target runtime:** 60-120 min Mode A
**Upper bound:** 150 min
**Confidence:** high (mechanical renames + doc rewrites; no semantic change; cascade risk LOW per audit)

**Rationale:**
- 4 fixes, each ~10-25 min: ~60-90 min mechanical work
- Doc rewrites for Fix 2 + Fix 3 may take longer due to careful canonical-vocabulary substitution — add buffer
- Test suite re-run after each fix to catch ripple early
- Final verification grep + clippy ~5 min

**Risks:**
- Fix 1's 5 expected-string sites may have line drift between Sonnet's first read and final commit — Sonnet should grep by content not by line number
- Fix 2's doc rewrite touches a justification block that explains alias resolution — needs careful preservation of the structural-tier-distinction semantic; that's the load-bearing reasoning, not just vocabulary
- L1-check-C closure rename may have additional citation sites if the closure is called more than 4 times — Sonnet should grep-verify exhaustively, not trust the audit's count

**Calibration check (fill in at completion):**
- Actual runtime: [TBD]
- Within prediction band? [TBD]

## Out-of-scope rows (REJECTED if attempted)

- Stale-vocab L2 doc mumbles (NOT enumerated by audit; out of arc 224 scope)
- L1-runtime-3 re-do (already absorbed by arc 225)
- Type-equality check renames in check.rs (those check alias-resolved heads; correct as-is)
- types.rs type registrations (correct as-is)
- Any holon-rs touch
- Any aliases / deprecation shims (HARD CUT)
- Any "stub", "future arc", "deferred to" language in SCORE

## Honesty deltas accepted

- Closure citation count may be ≥4 not exactly 4 if there are more `symbol_ty()` call sites than the audit enumerated — exhaustive grep wins
- Some `wat::kernel::Sender` may already appear in runtime.rs in other contexts (e.g., comments); row 7's "≥ 5" accommodates that without false positives
- Fix 3's doc text spans more than one line at check.rs:143 — Sonnet picks the structural extent honestly

## Honesty deltas NOT accepted (STOP triggers fire)

- Baseline test count regresses (827 → less than 827 PASS) — STOP-2
- Old name appears anywhere in src/ post-rename — STOP-6
- "Mostly done; a few expected-strings deferred" — REJECT; ship all 5 or STOP
- "Doc rewrite incomplete; partial scope" — REJECT; ship the full rewrite or STOP
- "Some clippy warnings appeared from the rename" — STOP-5; investigate and resolve, not accept
- Historical artifact rewritten — STOP-9 per `feedback_inscription_immutable`

## STOP triggers (cross-ref from BRIEF — all REJECTION criteria)

- **STOP-1:** unexpected compile errors
- **STOP-2:** test regression from 827 baseline
- **STOP-3:** 150 min elapsed
- **STOP-4:** holon-rs touched
- **STOP-5:** new clippy warning introduced
- **STOP-6:** stale-vocab L2s touched (out of scope)
- **STOP-7:** L1-runtime-3 re-done

If any STOP fires: ship NOTHING beyond the clean-stoppable state; surface in SCORE as honest delta.

## SCORE doc

SCORE will live at `docs/arc/2026/05/224-substrate-naming-honesty-audit/SCORE-STONE-224.5.md`. Per `feedback_inscription_immutable`, that's a NEW file. Body cites each row's verification command + result.
