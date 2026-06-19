# DESIGN — Stone 5b: `collect-rules` (the reflection that greens the north star)

The last stone of the semantic slice. `(:wat::rete::collect-rules :ns) -> PersistentVector<Rule>` gathers every
`defrule`-defined rule in a namespace, so `(compile (collect-rules :weather))` works and the north-star
acceptance test goes **green**. A Rust primitive (wat orchestrates Rust), mirroring the test runner's
`discover_tests`.

## Why a Rust primitive
`defrule` (5a) expands to a zero-arg `defn` returning `:wat::rete::Rule` — the return type is the reflection
marker, exactly as `deftest` marks tests by returning `:wat::test::TestResult`. There is no wat-level
"enumerate the defns in a namespace" primitive, and there shouldn't be a mutable global registry (wat is pure).
The test runner solves the identical problem by **reflecting the frozen symbol table** (`discover_tests`,
`src/test_runner.rs:584` → `is_test_function:611`: zero `param_types` + `ret_type` matches a marker type).
`collect-rules` does the same at eval time — the eval-time `sym: &SymbolTable` exposes
`functions: HashMap<String, Arc<Function>>` (`symbol_table.rs:35`), and `Function.ret_type` is reflectable
(`environment.rs:54`). This is the "wat orchestrates Rust" pattern: `defrule` plants the discoverable defns,
`collect-rules` reflects + invokes them.

## What 5b delivers (Rust)
`(:wat::rete::collect-rules <ns: :wat::core::keyword>) -> :wat::core::PersistentVector<wat::rete::Rule>`:
1. Eval arg[0] → a keyword value (e.g. `:weather`, stored as `":weather"`).
2. `prefix = "{ns}::"` (e.g. `":weather::"`).
3. Reflect `sym.functions`: select entries where `name.starts_with(prefix)` AND `f.param_types.is_empty()` AND
   `f.ret_type == TypeExpr::Path(":wat::rete::Rule")` (the `defrule` marker — mirrors `is_test_function`).
4. **Sort the names** (deterministic rule order — `discover_tests` sorts too).
5. For each name, **invoke the zero-arg fn**: `eval_inner` of `WatAST::List([Keyword(name)])` → the `Rule`
   value (the 5a probe's manual-collect proved `(:weather::cold-and-windy)` yields the Rule).
6. Collect into an `rpds::VectorSync<Value>` → `Value::wat__core__PersistentVector`.

Home: `src/rete/collect.rs` (new — reflection is a distinct concern from `matcher.rs`'s matching/RHS).
Registration: dispatch arm in `runtime.rs` (beside the other `:wat::rete::*` arms); TypeScheme in `check.rs`
(`params: [:wat::core::keyword]`, `ret: PersistentVector<:wat::rete::Rule>`) — **no `infer_list` bypass needed**
(the `:ns` arg is an undefined keyword = a plain keyword value, unifies normally; unlike `return-type-of` whose
arg was a fn).

## Namespace scoping
Prefix match on `"{ns}::"` — `:weather` collects `:weather::cold-and-windy` (and any `:weather::sub::*`), not
`:weatherfoo::x` (the `::` guards the boundary) and not other namespaces. Subtree inclusion is intended.

## The one contract decision (pinned)
`collect-rules` returns rules in **sorted-by-name order** (deterministic — the compiled network + the
differential oracle must be reproducible). Non-rule defns in the namespace are excluded by the
zero-arg + `ret_type == Rule` filter. Empty PV if the namespace has no rules (never raises).

## Files touched
- `src/rete/collect.rs` (new) — `eval_collect_rules`.
- `src/rete/mod.rs` — `mod collect;` + a note.
- `src/runtime.rs` — one dispatch arm.
- `src/check.rs` — one TypeScheme.
- `tests/probe_arc278_5b_collect_rules.rs` (new) — unit probe (scoping, multi-rule, non-rule exclusion).
- `tests/probe_arc278_northstar_cold_and_windy.rs` — **un-ignore** (the milestone goes green).

## NORTH STAR
`(:wat::rete::collect-rules :weather)` → `[cold-and-windy]` → `compile` → `insert` ×2 → `fire-rules` →
`(query fired :weather::ColdAndWindy)` → length **1**. The full DSL + engine, end to end, on the oracle.
**Labeled spec-speed; the Rust fire kernel (perf arc) is validated against this oracle next.**

## Out of scope = REJECTED
- **`defquery` / `QueryNode` / params** — deferred + likely annihilatable (a query = a result-deriving rule).
- **`Snapshot`** (4d) — small orthogonal sibling, later.
- **The Rust fire kernel** — the committed perf arc, AFTER the north star greens.
