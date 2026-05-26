# SCORE — Stone 237.5.fix-nominal-identity — the one type-identity authority

**Date:** 2026-05-25
**Status:** COMPLETE — 12/12 fix-probe PASS. All scorecard rows green.

---

## Scorecard

| # | Row | Command | Result |
|---|-----|---------|--------|
| 1 | Compile clean | `cargo build --release -p wat 2>&1 \| tail -3` | 0 errors; 107 warnings (pre-existing ceiling) |
| 2 | **fix-nominal probe 12/12 PASS** (LOAD-BEARING) | `cargo test --release --test probe_arc237_stone5fix_nominal 2>&1 \| tail -3` | `12 passed; 0 failed` |
| 3 | Lib baseline | `cargo test --release --lib -p wat --no-fail-fast 2>&1 \| tail -3` | `827 passed; 0 failed` |
| 4 | Stone 237.5 regression | `cargo test --release --test probe_arc237_stone5_conforms 2>&1 \| tail -3` | `12 passed; 0 failed` |
| 5 | Arc 234 type probe regression | `cargo test --release --test probe_diagnostic_polymorphic_type 2>&1 \| tail -3` | `8 passed; 0 failed` |
| 6 | Stone 237.1 regression | `cargo test --release --test probe_arc237_stone1_typeunion_substrate 2>&1 \| grep "test result:"` | `14 passed; 0 failed` |
| 7 | Stone 237.2 regression | `cargo test --release --test probe_arc237_stone2_defclause_substrate 2>&1 \| grep "test result:"` | `12 passed; 0 failed` |
| 8 | Stone 237.3 regression | `cargo test --release --test probe_arc237_stone3_guard_ensure 2>&1 \| grep "test result:"` | `14 passed; 0 failed` |
| 9 | Stone 237.4 regression | `cargo test --release --test probe_arc237_stone4_rich_errors 2>&1 \| grep "test result:"` | `10 passed; 0 failed` |
| 10 | holon-rs untouched | STOP-5 | confirmed — zero holon-rs changes |
| 11 | No files outside src/runtime.rs | STOP-4 | confirmed — only src/runtime.rs modified |
| 12 | Authority is wildcard-free | structural | confirmed — see grep below |

All 12/12 PASS.

---

## Wildcard-free confirmation

```
grep -n "_ =>\|other =>" src/runtime.rs | grep -A2 -B2 declared_type_name
```

The `declared_type_name` method contains no bare `_ =>` or `other =>` arm. Every `Value` variant is listed explicitly. The Rust compiler enforces exhaustiveness — a future variant without a declared-type arm is a compile error, not a silent fallthrough.

---

## Per-form FQDN-source table

| Value kind | declared FQDN source | Example output |
|---|---|---|
| `Value::holon__HolonAST(h)` | `extract_classifier(h)` with fallback `"wat::holon::HolonAST"` | `"my::Foo"` |
| `Value::Struct(sv)` | `sv.type_name.trim_start_matches(':')` — also covers newtype (it's a Struct at runtime) | `"my::Point"`, `"my::Price"` |
| `Value::wat__Record { class_fqdn }` | `class_fqdn.to_string()` (already colon-free) | `"my::Circle"` |
| `Value::Enum(ev)` | `ev.type_path.trim_start_matches(':')` — the declared enum FQDN, NOT `type_name()` | `"my::Color"` |
| every primitive/kind variant | `self.type_name().to_string()` — explicit arm for each | `"wat::core::i64"`, `"wat::core::bool"`, … |

---

## Implementation

**File changed:** `src/runtime.rs` only.

### Three changes

**1. Minted `Value::declared_type_name(&self) -> String`** — added immediately after `Value::type_name()` in the `impl Value` block (~74 lines including doc comments). Exhaustive match: 4 nominal arms (HolonAST / Struct / Record / Enum) + 36 explicit primitive arms. No bare `_ =>` or `other =>`.

**2. Routed `eval_type`** — the inline 4-arm match (`HolonAST` / `Struct` / `Record` / `other`) is replaced by a single `arg_val.declared_type_name()` call. The dispatch doc comment updated to reference the one authority.

**3. Routed `concrete_type_name_matches`** — the old `(Record special-case) + (other.type_name() wildcard)` match is deleted. The function body is now one expression: `value.declared_type_name() == stripped`. Doc comment updated.

### Pre-fix drift (proven empirically)

| value | `(:wat::core::type v)` pre-fix | `(conforms? v :ItsType)` pre-fix | post-fix both |
|---|---|---|---|
| struct `:my::Point` | `"my::Point"` ✓ | `false` ✗ | `"my::Point"` / `true` ✓ |
| newtype `:my::Price` | `"my::Price"` ✓ | `false` ✗ | `"my::Price"` / `true` ✓ |
| enum `:my::Color::Red` | `"wat::core::Enum"` ✗ | `false` ✗ | `"my::Color"` / `true` ✓ |

---

## Line count

| File | Pre-stone lines | Post-stone lines | Net added |
|------|-----------------|------------------|-----------|
| `src/runtime.rs` | 33,096 | 33,170 | +74 (`declared_type_name` method + two call-site simplifications) |

---

## Honest deltas

None. The exhaustive match compiled on the first attempt. No cascade was required (no new Value variant, no check.rs change, no holon-rs change). `concrete_type_name_matches` is now a thin delegator to the authority rather than a second independent derivation — both consumers agree by construction.

---

## Working tree on return

```
 M src/runtime.rs
?? docs/arc/2026/05/237-polymorphism-consolidation/SCORE-STONE-237.5.fix-nominal-identity.md
```

holon-rs untouched. STOP-5 not triggered. DO NOT commit (orchestrator commits).
