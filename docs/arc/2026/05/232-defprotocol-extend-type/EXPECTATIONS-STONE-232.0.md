# EXPECTATIONS — Arc 232 Stone 232.0 — mint `:wat::core::apply`

Mode A target: **18/18 PASS**. Every row binds to a specific verification command. No row marked PASS without naming the verification.

## Scorecard

| # | Row | Binding verification | Expected |
|---|---|---|---|
| 1 | Compile clean | `cargo build --release -p wat 2>&1 \| tail -5` | 0 errors |
| 2 | Lib tests baseline match | `cargo test --release --lib -p wat --no-fail-fast 2>&1 \| tail -3` | ≥ 827 passed; 0 failed; 1 ignored (may grow if sonnet adds lib tests) |
| 3 | Clippy no new warnings | `cargo clippy --release --lib -p wat -- -D warnings 2>&1 \| grep -c "warning"` | ≤ 52 (baseline) |
| 4 | `eval_apply` function exists | `grep -c "fn eval_apply" src/runtime.rs` | ≥ 1 |
| 5 | Dispatch arm registered | `grep -c '":wat::core::apply"' src/runtime.rs` | ≥ 1 |
| 6 | TypeScheme registered | `grep -E '":wat::core::apply".into\(\)' src/check.rs` | ≥ 1 hit (or equivalent registration pattern) |
| 7 | **Probe 1 flips FAIL → PASS** | `cargo test --release --test probe_diagnostic_dynamic_keyword_invocation probe_1 -- --nocapture 2>&1 \| tail -3` | `test result: ok. 1 passed` |
| 8 | **Probe 2 flips FAIL → PASS** | `cargo test --release --test probe_diagnostic_dynamic_keyword_invocation probe_2 -- --nocapture 2>&1 \| tail -3` | `test result: ok. 1 passed` |
| 9 | **Probe 3 flips FAIL → PASS** | `cargo test --release --test probe_diagnostic_dynamic_keyword_invocation probe_3 -- --nocapture 2>&1 \| tail -3` | `test result: ok. 1 passed` |
| 10 | New probe 4 — leading args + tail vec | `cargo test --release --test probe_diagnostic_dynamic_keyword_invocation probe_4 -- --nocapture` | PASS |
| 11 | New probe 5 — empty tail vec | `cargo test --release --test probe_diagnostic_dynamic_keyword_invocation probe_5 -- --nocapture` | PASS |
| 12 | New probe 6 — special-form rejection | `cargo test --release --test probe_diagnostic_dynamic_keyword_invocation probe_6 -- --nocapture` | PASS (test verifies the error is raised cleanly) |
| 13 | New probe 7 — non-keyword head rejection | `cargo test --release --test probe_diagnostic_dynamic_keyword_invocation probe_7 -- --nocapture` | PASS (test verifies the error is raised cleanly) |
| 14 | New probe 8 — non-vector last arg rejection | `cargo test --release --test probe_diagnostic_dynamic_keyword_invocation probe_8 -- --nocapture` | PASS (test verifies the error is raised cleanly) |
| 15 | Full probe file green | `cargo test --release --test probe_diagnostic_dynamic_keyword_invocation -- --nocapture 2>&1 \| tail -3` | `test result: ok. ≥ 6 passed; 0 failed` |
| 16 | Holon-rs untouched | `git -C /home/watmin/work/holon/holon-rs/ status --short` | empty output |
| 17 | No new substrate primitives beyond apply | `git diff --stat src/runtime.rs src/check.rs` | ONLY `eval_apply`-adjacent edits + TypeScheme registration (sonnet's diff should be reviewable in ~200 lines or fewer) |
| 18 | No aliases / deprecation shims | grep for `legacy\|deprecated\|alias.*apply` in src/ — | 0 matches relevant to apply |

## Independent prediction (calibration record)

**Target runtime:** 60-90 min Mode A
**Upper bound:** 120 min
**Confidence:** high (the probe is the design substrate; sonnet mirrors)

**Rationale:**
- Implementation maps cleanly to existing dispatch pattern in `eval_list`
- TypeScheme registration mirrors existing variadic primitives (arc 091 struct->form pattern)
- Probe rewrites are mechanical (3 existing + 5 new)
- The probe-as-design-substrate eliminates discovery failure mode (FM 2-bis pre-empted)
- Sonnet's calibration trend this session: under-prediction band (Stone 224.5 ~20 min vs 60-120 predicted; Stone 227.2 v3 ~23 min vs 120-240)

**Risks:**
- TypeScheme expression of "last must be Vector<T>, leading args are T" may need substrate work if rest_param_type doesn't express it directly — Sonnet should ship the cleanest honest type-check and surface the gap in SCORE if a deeper change is needed
- Dispatch lookup path duplication — if `eval_apply` needs to mirror the full literal-keyword head dispatch (substrate verbs + def-bound + Symbol-fn), there may be code reuse opportunities to factor out. Sonnet picks: duplicate-clean OR factor-out, both honest
- Special-form detection: there's a list of reserved special-form keywords; sonnet identifies the canonical set and rejects on match

**Calibration check (fill in at completion):**
- Actual runtime: [TBD]
- Within prediction band? [TBD]

## Out-of-scope rows (REJECTED if attempted)

- fn-value head arm (v2 territory)
- defprotocol macro (arc 232.1+)
- Reflection-layer integration (`:wat::runtime::lookup-fn`)
- Spread for non-Vector collections (List, Set, etc.)
- Variadic head (multiple callables)
- Any holon-rs touch
- Any aliases / deprecation shims (HARD CUT)
- "Stub", "future arc", "deferred to" language in SCORE
- Stale-vocab L2 audit work (out of arc 232's scope)

## Honesty deltas accepted

- New lib test count may grow if sonnet adds internal `runtime::tests::*` for eval_apply — that's fine; row 2 says "≥ 827"
- TypeScheme registration shape may not match arc 091's struct->form exactly; sonnet picks the closest precedent honestly
- Probe naming may differ from sketch; the binding row stays
- The "spread constraint" may need to live in arg-eval logic vs TypeScheme; sonnet picks the cleaner expression

## Honesty deltas NOT accepted (STOP triggers fire)

- Baseline test count regresses below 827 — STOP-2
- Any of probes 1-3 still FAILS post-stone — STOP-7
- Special-form rejection not implemented — STOP-8
- Sonnet adds fn-value arm "while we're here" — STOP-6
- Sonnet edits holon-rs accidentally — STOP-4
- Sonnet edits wat-edn for unrelated reasons — STOP-6
- "Partial — apply works for the 3 existing probes but not the new edge cases" — REJECT; ship all or STOP

## STOP triggers (cross-ref from BRIEF — all REJECTION criteria)

- **STOP-1:** unexpected compile errors
- **STOP-2:** test regression from 827 baseline
- **STOP-3:** 120 min elapsed
- **STOP-4:** holon-rs touched
- **STOP-5:** new clippy warning introduced
- **STOP-6:** scope creep (fn-value / defprotocol / reflection)
- **STOP-7:** existing 3 probes still FAIL
- **STOP-8:** special-form rejection NOT implemented

If any STOP fires: SCORE names it explicitly; ship nothing past the clean-stoppable state.

## SCORE doc

SCORE will live at `docs/arc/2026/05/232-defprotocol-extend-type/SCORE-STONE-232.0.md`. Per `feedback_inscription_immutable`, that's a NEW file. Body cites each row's verification command + result + any honest delta.

## What this unblocks

Arc 232.0 closure → arc 232.1 (defprotocol macro design) becomes possible:
- The dispatcher pattern in defprotocol uses `(apply mangled-keyword [self ...rest])` — the foundational call
- extend-type macro generates the namespaced defns the dispatcher targets
- Convergence with Clojure's four-corner (defrecord + defprotocol + extend-type + satisfies?) becomes structurally available

Convergence #16 (apply-as-universal-escape-hatch-every-Lisp-eventually-mints) inscribes when the stone ships.
