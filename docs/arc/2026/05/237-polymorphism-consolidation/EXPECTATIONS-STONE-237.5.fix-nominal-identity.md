# EXPECTATIONS — Stone 237.5.fix-nominal-identity

Mode A: 12/12 on the fix-probe + clean baseline + both consumers agreeing.

## Scorecard

| # | Row | Verification | Expected |
|---|---|---|---|
| 1 | Compile clean | `cargo build --release -p wat 2>&1 \| tail -5` | 0 errors |
| 2 | **fix-probe 12/12** (LOAD-BEARING) | `cargo test --release --test probe_arc237_stone5fix_nominal 2>&1 \| tail -3` | `12 passed; 0 failed` |
| 3 | 237.5 conforms? probe still green | `cargo test --release --test probe_arc237_stone5_conforms 2>&1 \| tail -3` | `12 passed; 0 failed` |
| 4 | Lib baseline held | `cargo test --release --lib -p wat --no-fail-fast 2>&1 \| tail -3` | ≥ 827 passed; 0 failed |
| 5 | arc 234.0 `type` probe (the other consumer) | `cargo test --release --test probe_diagnostic_polymorphic_type 2>&1 \| tail -3` | `8 passed; 0 failed` |
| 6 | 237.1–237.4 regression | `for n in 1_typeunion_substrate 2_defclause_substrate 3_guard_ensure 4_rich_errors; do cargo test --release --test probe_arc237_stone$n 2>&1 \| grep "test result"; done` | 14/12/14/10, all 0 failed |
| 7 | **authority is wildcard-free** (✅✅✅ guard) | inspect `declared_type_name`: no bare `_ =>` / `other =>` arm catching type-bearing variants | exhaustive; primitives explicit |
| 8 | both consumers route through the authority | grep: `eval_type` + conforms? both call `declared_type_name`; `concrete_type_name_matches` wildcard gone | one authority, two callers |
| 9 | drift closed | probe_06 (conforms? struct) AND probe_12 (`type` struct) both green — same value, same answer | agree by construction |

**Clippy NOT a ceiling concern** per user direction.

## Prediction

**Target: 20–40 min Mode A. STOP: 75 min.**

Surface estimate: `src/runtime.rs` ~60–120 lines (the `declared_type_name` exhaustive match + Enum FQDN extraction + two call-site routings + delete `concrete_type_name_matches` wildcard). Single file.

Confidence: HIGH. `eval_type` already has the Struct/Record/HolonAST arms (factor them); the only NEW extraction is the Enum FQDN field; the only NEW discipline is enumerating primitives instead of wildcarding. The drift is mechanical to close once the authority exists.

## Risks

1. **Exhaustive match verbosity.** `Value` has many variants; listing primitives explicitly is verbose but is the compiler-guard. If a genuinely-kind-only sub-group is large, it may be grouped — but NEVER a bare catch-all that a type-bearing variant could slip into (Risk = re-introducing the exact bug). Trap-door 1.
2. **Enum FQDN field.** `Value::Enum(ev)` — the declared enum FQDN lives on `EnumValue`; find the right field (NOT `type_name()`). probe_10 (`type` enum) is the verifier. Trap-door 2.
3. **newtype representation.** probe_11 shows `type` already gets newtype right via the Struct arm (newtype is a `Value::Struct`). Confirm conforms? inherits that by routing through the authority — don't add a separate newtype arm unless the probe demands it. Trap-door 3.
4. **`String` vs `&'static str`.** `declared_type_name` returns owned `String` (per-instance FQDNs aren't static); `type_name()` stays `&'static str`. Don't conflate.

## Out-of-scope (REJECTED)

- is-<Name>? auto-mint (Stone 237.6 — rides conforms? → rides the authority).
- ✅✅✅✅ encapsulation / lint forbidding `type_name()` re-derivation (surfaces from this attempt if reachable; its own stone).
- holon-rs (STOP-5).

## SCORE

`SCORE-STONE-237.5.fix-nominal-identity.md` (NEW). 12-row scorecard verbatim + per-form FQDN-source table + wildcard-free confirmation + line count + honest deltas. Mirror 234.3c.fix SCORE shape.
