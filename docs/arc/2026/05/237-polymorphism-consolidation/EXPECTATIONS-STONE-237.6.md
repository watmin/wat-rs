# EXPECTATIONS — Stone 237.6

Mode A: 10/10 on the probe + clean baseline + all `is-<Name>?` composing conforms?.

## Scorecard

| # | Row | Verification | Expected |
|---|---|---|---|
| 1 | Compile clean | `cargo build --release -p wat 2>&1 \| tail -5` | 0 errors |
| 2 | **is-predicate probe 10/10** (LOAD-BEARING) | `cargo test --release --test probe_arc237_stone6_is_predicate 2>&1 \| tail -3` | `10 passed; 0 failed` |
| 3 | Lib baseline held | `cargo test --release --lib -p wat --no-fail-fast 2>&1 \| tail -3` | ≥ 827 passed; 0 failed |
| 4 | 237.5 conforms? regression | `cargo test --release --test probe_arc237_stone5_conforms 2>&1 \| tail -3` | `12 passed; 0 failed` |
| 5 | 237.5.fix regression | `cargo test --release --test probe_arc237_stone5fix_nominal 2>&1 \| tail -3` | `12 passed; 0 failed` |
| 6 | 234.0 `type` + Record.wat (the body-switch) | `cargo test --release --test probe_diagnostic_polymorphic_type 2>&1 \| tail -3` | `8 passed; 0 failed` |
| 7 | 237.1–237.4 regression | per-probe | 14/12/14/10, 0 failed |
| 8 | union payload green | probe_07/08/09 (is-Shape? member→true, non-member→false) | conforms? unwraps membership |
| 9 | Record.wat composes conforms? | grep wat/Record.wat is-predicate body | `(:wat::core::conforms? …)`, NOT `(= (type v) …)` |
| 10 | typealias has NO predicate | the pass skips `TypeDef::Alias` | no `is-<Alias>?` minted |

**Clippy NOT a ceiling concern** per user direction.

## Prediction

**Target: 30–55 min Mode A. STOP: 90 min.**

Surface: `src/runtime.rs` ~50–90 lines (`register_type_predicates` pass + naming helper + WatAST body builder, mirroring `register_struct_methods`); `src/freeze.rs` ~3 lines (call the pass); `wat/Record.wat` ~1 line (body switch). Three files.

Confidence: HIGH. The synthesis machinery exists (`register_struct_methods` is the template); the body is a trivial 3-node `WatAST` calling the already-shipped conforms?; the naming mirrors Record.wat. The only new judgment is param-typing the `v` arg (∀T) — mirror conforms?'s arg.

## Risks

1. **Dispatch-as-call vs field-access.** `(:ns::is-Name? record)` must resolve as a function call. Landing the predicate in `sym.functions` (like accessors) fixes the pre-stone `UnknownField` fallback. Trap-door 1.
2. **Predicate param type (∀T).** The predicate accepts any value; type its `v` param the way conforms?'s value-arg is typed, or the checker rejects calls on arbitrary values. Trap-door 2.
3. **Naming derivation.** FQDN → `is-<LastSegment>?` within the same namespace. Must match Record.wat's existing rule so the family is uniform. Trap-door 3.
4. **Record.wat regression.** Switching its body to conforms? must keep its existing tests + probe_10 green (conforms? handles records via class_fqdn — proven in 237.5).

## Out-of-scope (REJECTED)

- typealias predicate; arc 226 built-in `is-Map?` reconciliation (separate stone if they re-compute); ✅✅✅✅ encapsulation; Dispatch/arithmetic migration (237.7/237.8); holon-rs (STOP-5).

## SCORE

`SCORE-STONE-237.6.md` (NEW). 10-row scorecard + naming rule + conforms?-composition confirmation (incl. Record.wat) + line counts + honest deltas. Mirror Stone 237.2 SCORE shape.
