# SCORE — Arc 232 Stone 232.1 — defprotocol + extend-type macros (BUNDLED)

**Status:** COMPLETE. 12/12 PASS.
**Authored:** 2026-05-23
**Mode:** Mode A (sonnet writes substrate; orchestrator briefs + scores)

---

## 12-Row Scorecard

| # | Row | Command | Result |
|---|---|---|---|
| 1 | Compile clean | `cargo build --release -p wat 2>&1 \| tail -5` | `Finished \`release\` profile [optimized] target(s) in 17.88s` — 0 errors ✓ |
| 2 | **New probe FLIPS 0→3 PASS** (LOAD-BEARING) | `cargo test --release --test probe_arc232_stone1_defprotocol_macros 2>&1 \| tail -5` | `test result: ok. 3 passed; 0 failed` ✓ |
| 3 | **FM 2-bis probe STAYS GREEN** | `cargo test --release --test probe_diagnostic_defprotocol_dispatch 2>&1 \| tail -5` | `test result: ok. 3 passed; 0 failed` ✓ |
| 4 | Lib tests baseline | `cargo test --release --lib -p wat --no-fail-fast 2>&1 \| tail -3` | `test result: ok. 827 passed; 0 failed; 1 ignored` ✓ |
| 5 | **Stone 232.0a probe** (typed-entities reflection guard) | `cargo test --release --test probe_diagnostic_typed_entities_reflection 2>&1 \| tail -3` | `7 passed; 0 failed` ✓ |
| 6 | Stone 233.3 probe | `cargo test --release --test probe_stone_233_3_runtime_error_edn 2>&1 \| tail -3` | `5 passed; 0 failed` ✓ |
| 7 | Stone 233.2.e probe | `cargo test --release --test probe_stone_233_2_e_ast_derived_provenance 2>&1 \| tail -3` | `5 passed; 0 failed` ✓ |
| 8 | Stone 233.2.l probe | `cargo test --release --test probe_stone_233_2_l_wat_value_seal 2>&1 \| tail -3` | `3 passed; 0 failed` ✓ |
| 9 | Stone 233.2.k probe | `cargo test --release --test probe_stone_233_2_k_variant_retired 2>&1 \| tail -3` | `5 passed; 0 failed` ✓ |
| 10 | Stone 233.1 ValueSnapshot probes | `cargo test --release --test probe_diagnostic_value_snapshot_in_errors 2>&1 \| tail -3` | `8 passed; 0 failed` ✓ |
| 11 | Clippy no new warnings | `cargo clippy --release --lib -p wat -- -D warnings 2>&1 \| grep -c "warning"` | `54` — at threshold (≤ 54) ✓ |
| 12 | holon-rs untouched | `git -C /home/watmin/work/holon/holon-rs/ status --short` | empty output ✓ |

**Result: 12/12 PASS**

---

## Per-File Line Counts

| File | Lines | Role |
|---|---|---|
| `wat/holon/defprotocol.wat` | 125 | NEW — defprotocol macro |
| `wat/holon/extend-type.wat` | 129 | NEW — extend-type macro |
| `src/stdlib.rs` | +23 lines (325 total) | MODIFIED — two new WatSource entries |
| `tests/probe_arc232_stone1_defprotocol_macros.rs` | 173 | NEW — 3-contract probe |

Total new code: 427 lines across 4 files (23 modification lines in stdlib.rs).

---

## Time Breakdown

Estimated: ~75 minutes Mode A (under-band relative to 90-150 target).

- Reading all required docs (BRIEF, EXPECTATIONS, DESIGN, FM 2-bis probe, defrecord precedent): ~15 min
- First macro draft (defprotocol.wat + extend-type.wat): ~10 min
- stdlib.rs entries: ~5 min
- Probe authoring: ~5 min
- Debugging rest-param evaluation issue (core discovery — see below): ~25 min
- Debugging bracket/paren count: ~15 min

**Calibration:** Under-band (90-150 predicted, ~75 delivered). Consistent with prior stone calibration trend (232.0a: 52 min vs 40-75 target).

---

## Core Discovery: rest-param is WatAST::List, not Value::Vec

**The fundamental blocker** (resolved, fully documented):

The initial defprotocol design used `& (methods :AST<wat::core::Vector<wat::WatAST>>)` rest-param and tried to call `(:wat::core::map methods fn-closure)` inside a `~@(let [...] ...)` unquote-splicing. This failed with "unbound symbol: format" because:

1. The rest-binder binds `methods` as `WatAST::List(rest_args, span)` — NOT as `Value::Vec` of `Value::wat__WatAST`.
2. Inside `~@(let [...] ...)`, the let expression is EVALUATED at macro-expand time via `crate::runtime::eval`.
3. Before evaluation, `substitute_bindings` replaces the `methods` symbol with `WatAST::List([WatAST::List(format-form,...), ...])`.
4. When `eval` evaluates `(:wat::core::map methods fn-closure)`, `methods` is now a `WatAST::List` starting with `format`. The runtime tries to call `format` as a function → "unbound symbol: format".

**Solution**: Use the defrecord pattern exactly:
```
methods-h    (:wat::holon::from-wat (:wat::core::quote methods))
n-methods    (:wat::holon::statement-length methods-h)
methods-vec  (:wat::holon::Bundle/children methods-h)
dispatchers  (:wat::core::map (:wat::core::range 0 n-methods) fn-closure)
```

`(:wat::core::quote methods)` prevents evaluation of the substituted WatAST::List. `from-wat` converts it to HolonAST::Bundle. `Bundle/children` returns `Value::Vec` of HolonAST values (proper iterable for `map`). `range + Vector/get` iterates per defrecord precedent.

This is the same structural pattern defrecord uses for its field iteration. The discovery: defrecord's approach (take fields as a quoted WatAST arg, parse via from-wat + Bundle/children + range) is the ONLY safe way to iterate over rest-params at macro expand time. Direct `map methods fn` attempts to evaluate the substituted WatAST as code.

---

## D7 Decision — Method-name Validation at Expand Time

**DEFERRED to v2.** Rationale documented here and in BRIEF Out-of-scope.

Implementing compile-time validation that extend-type method names match defprotocol declarations requires a registry mapping protocol FQDN → declared method names. This registry must be stored somewhere accessible at macro-expand time. Options:
1. SymbolTable (Rust-side extension — violates STOP-4 / BRIEF's "NO Rust changes")
2. A wat-side atom (requires the registry be populated before extend-type runs, and be accessible at macro-expand time — no current mechanism)

Both options require Rust changes or new substrate primitives. Stone 232.1 is pure wat-side macro work. D7 validation defers to runtime: typos in method names surface as `UnknownFunction` naming the missing mangled keyword (arc 233 provides the span + verb). This is the honest behavior per `feedback_absence_is_signal` — the gap points at real substrate work.

---

## Honest Deltas

### 1. return type must be explicit in extend-type

The DESIGN imagined `extend-type` looking up the return type from the protocol's declaration at expand time. Since D7 registry is deferred, there is no way to do this without Rust changes. Resolution: extend-type method-body forms require an explicit `-> :RetType` annotation (consistent with defn syntax).

This is `feedback_verbose_is_honest` applied: the verbose form carries information (the expected return type). The type checker validates that the impl's return type is consistent when `apply` routes to it.

Probe form: `(format [self] -> :wat::core::String "voltage-formatted")` — explicit return type.

### 2. Paren/bracket counting at macro depth

The closing bracket structure of deeply-nested macros is subtle. The key rule: after the innermost `[self]` vector in the generated `apply` call, the closing sequence is:

```
[self])            ;; close apply
  )                ;; close inner let2 (body = apply)
)                  ;; close defn (body = let2)
)                  ;; close quasiquote (body = defn)
)                  ;; close fn-body let1 (body = quasiquote)
)                  ;; close fn (body = let1)
)                  ;; close map (args: range + fn)
]                  ;; close outer let binding vec
```

This is 6 closing parens + 1 bracket (7 total closing tokens) after `[self]`. Verified against defrecord's equivalent structure (7 `)` before `]` after the innermost quasiquote body closes).

### 3. from-wat on rest-param gives correct Bundle structure

For defprotocol with a SINGLE method declaration `(format [self] -> :wat::core::String)`:
- After `from-wat (quote methods)`: `HolonAST::Bundle([Bundle([symbol("format"), Bundle([symbol("self")]), symbol("->"), keyword("wat::core::String")])])`
- `statement-length` = 1 (one method)
- `Bundle/children` = `Value::Vec([HolonAST::Bundle(format-form)])`
- `Vector/get methods-vec 0` = the format-form Bundle
- `Bundle/children method-h` = `[symbol("format"), Bundle([symbol("self")]), symbol("->"), keyword("wat::core::String")]`
- Index 0 = symbol("format") → `from-holon` → `Value::keyword("format")` → `keyword/to-string` → "format"
- Index 3 = keyword("wat::core::String") → `from-holon` → `Value::keyword(":wat::core::String")`

The round-trip works correctly.

---

## Rank-Up Evidence — Arc 233 + Stone 232.0a Tools

### 1. Probe 3's UnknownFunction message (confirmed arc 233 diagnostic substrate)

Probe 3 in the new macro probe tests the missing-impl error path. The macro-generated dispatcher correctly fires and surfaces:

```
eval: UnknownFunction(":myapp::Unhandled/Formattable-format", Span { file: "<entry>", line: 17, col: 2 })
```

This is arc 233's precise UnknownFunction message naming the EXACT missing mangled keyword (`:myapp::Unhandled/Formattable-format`) plus the span. The error message passes the FM 2-bis probe 3 assertion without any scaffolding.

### 2. The diagnostic substrate's absence-of-need

During this stone's iteration, NO println! debugging was added. The error messages from the macro expansion machinery (e.g., "unbound symbol: format at entry:5:4", "computed unquote-splicing eval failed", Span pointing to wat/holon/defprotocol.wat:65:8) precisely named the failure site and the cause without ambiguity. Each error message told exactly what to fix.

Specifically:
- "unbound symbol: format at entry:5:4" → immediately pointed to the rest-param evaluation issue (the format symbol was being looked up at runtime instead of the method name being extracted from the WatAST)
- "macro :wat::holon::defprotocol — computed unquote-splicing eval failed" → named the exact macro + operation path (unquote-splicing → eval → failed)
- "span: Span { file: wat/holon/defprotocol.wat, line: 65, col: 8 }" → pinpointed the `~@(let ...)` unquote-splice site in defprotocol.wat

### 3. `#[wat_value]` seal — absence of need (structural confidence)

No attempt was made to add a new Value variant. The `#[wat_value]` seal's compile-error guarantee meant there was no uncertainty about whether adding substrate would accidentally extend Value. Pure macro work throughout.

### 4. Stone 232.0a reflection primitives (`extract-classifier`)

`extract-classifier` is live in the generated dispatcher bodies. Probe 1 and Probe 2 exercise it through the macro-generated dispatch path, and both pass. The reflection layer shipped in 232.0a is fully consumed by the defprotocol macro's generated dispatchers.

---

## What This Unblocks

- **Stone 232.3** — built-in-type extension proof. The defprotocol + extend-type macros are now live. Stone 232.3 is an integration test: extend a built-in type (e.g., `:wat::holon::Vector`) with a sample protocol. Mostly probe authoring over the macros this stone shipped.
- **Stone 232.5** — INSCRIPTION + USER-GUIDE chapter (arc 232 closure).
- **defrecord accessor synthesis** (separate stone outside arc 232) — method bodies in extend-type can use `Bind/right` + `Bundle/children` directly until accessor synthesis ships.

---

## Cross-References

- `docs/arc/2026/05/232-defprotocol-extend-type/BRIEF-STONE-232.1.md` — paired BRIEF
- `docs/arc/2026/05/232-defprotocol-extend-type/EXPECTATIONS-STONE-232.1.md` — paired EXPECTATIONS
- `docs/arc/2026/05/232-defprotocol-extend-type/DESIGN-STONE-232.1.md` — sub-DESIGN with locked decisions
- `docs/arc/2026/05/232-defprotocol-extend-type/SCORE-STONE-232.0a.md` — predecessor SCORE
- `tests/probe_diagnostic_defprotocol_dispatch.rs` — FM 2-bis probe (substrate sufficiency proof)
- `tests/probe_arc232_stone1_defprotocol_macros.rs` — the new probe (this stone's load-bearing artifact)
- `wat/holon/defprotocol.wat` — defprotocol macro (NEW)
- `wat/holon/extend-type.wat` — extend-type macro (NEW)
- `wat/holon/defrecord.wat` — defmacro precedent (key: from-wat + Bundle/children + range pattern)
- `src/stdlib.rs` — two new WatSource entries
