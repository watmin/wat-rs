# EXPECTATIONS — Stone P12c (independent scorecard, fixed BEFORE the strike)

Enriches the P12b tree with the per-edge payload (the `DerivationStep` edge + `:constraints`/`:bindings`/
`:pattern`/`rule`). The P12 north-star's via-COUNTS are preserved (via length unchanged); P12c adds the payload.

| # | what | command | expected |
|---|------|---------|----------|
| 1 | P12c payload probe green | `cargo test --release -p wat --test probe_arc278_P12c_explain_payload` | **6 passed; 0 failed; 0 ignored** |
| 2 | P12 north-star STILL green (counts preserved) | `cargo test --release -p wat --test probe_arc278_P12_explain_walk` | **2 passed** |
| 3 | P12a substrate green | `cargo test --release -p wat --test probe_arc278_P12a_explain_substrate` | 3 passed |
| 4 | rete differential UNCHANGED | `cargo test --release -p wat --test probe_arc278_P4a_native_fire_rules --test probe_arc278_P4c_native_retraction --test probe_arc278_P2_native_fire_once` | green |
| 5 | lib floor | `cargo test --release -p wat --lib -- --test-threads=1 \| grep "test result"` | **941 / 36** (no new failure) |
| 6 | deftest floor | `cargo test --release --test test \| grep "test result"` | **264 / 1** |
| 7 | deporder / nursery floors | (deporder) `1 / 0`; (nursery) `~893 / 4` (±3) | unchanged |
| 8 | build clean | `cargo build --release` | compiles; ~25 warnings (baseline) |

## The load-bearing assertion (probe test 6)
`constraint_is_the_substituted_form`: the first constraint on the Temperature step is `(:wat::core::< -5 0)` —
operand[1] is `IntLit(-5)` (the substituted `?c`), NOT the symbol `?c`. This proves the `resolve_operand` reuse:
the rendered predicate is what actually fired, with the concrete bound value. If operand[1] is still `?c`, the
substitution didn't happen (the helper isn't resolving against bindings) — the whole payload's point is lost.

## Runtime prediction
~30–45 min. The real work is the `step-payload` Rust helper (reusing `resolve_operand`/classifier + AST-rebuild)
+ the two records + the wat walk restructure + registration. Most of the clock = release builds + floor/differential runs.

## Trap-door risks (named)
- **Duplication drift** — if the helper re-implements operand resolution instead of calling `resolve_operand`,
  `:constraints` can disagree with the actual match. Probe test 6 + the reuse STOP-trigger guard it.
- **AST-rebuild** — turning a resolved `Value` (e.g. `i64(-5)`) into a `WatAST` literal node. Runtime quasiquote
  is proven (R3); a direct constructor also works. Test 6 catches a wrong node.
- **North-star regression** — restructuring `via` from `PV<DerivationNode>` to `PV<DerivationStep>` must keep the
  COUNT (`DerivationNode/via` length = # support edges). Test 2 guards it.
- **`:field` operands** — a constraint operand may be a `:field` ref (not a `?var`); `resolve_operand` needs the
  supporting fact to resolve it. The helper must receive the supporting fact (it does — `sfact`).
- **per-step binding projection** — `:bindings` must be projected to THIS condition's binder vars (test:
  `bindings[?c] = -5` on the Temperature step), not the full token bindings. If full bindings leak, test 2's
  scope is looser but still passes on `?c`; intent is per-step.

## Acceptance
All rows met, weighed against the orchestrator's OWN re-run + a read of the diff: the diff is additive to
`wat/rete.wat` (records + walk) + `src/rete/matcher.rs` (the reusing helper) + registration + un-ignored probe;
NO fire-path change; the `step-payload` helper visibly CALLS `resolve_operand` (not a re-impl). Commit on green + push.
