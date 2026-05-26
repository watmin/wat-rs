# BRIEF — Stone 237.6 — auto-mint `is-<Name>?` (named convenience over conforms?)

**Status:** READY TO SPAWN. `model: "sonnet"`.

## What to do

Every type-introducing declaration should hand you `:ns::is-<Name>?` ≡ `(conforms? x :ns::Name)`. Records already mint it (Record.wat). This stone adds the **four TypeEnv-registered forms** (struct/enum/newtype/union) via ONE new pass, and **unifies Record.wat's body onto conforms?**. Make the 10-contract probe go 10/10 (it's 1/10 — only the record predicate exists).

`is-<Name>?` is a **named convenience over the one mechanism (conforms?)** — not a second way. conforms? + declared_type_name (237.5/.fix) are the foundation and stay untouched.

## The fix

### 1. New pass: `register_type_predicates(types, sym)` (src/runtime.rs)

Mirror `register_struct_methods` (runtime.rs:2852+) — it iterates the TypeEnv and synthesizes `Function`s into `sym.functions`. The new pass iterates `types.iter()`; for every **non-Alias** `TypeDef` (Struct / Enum / Newtype / Union) synthesize:
- **name** `:<ns>::is-<LastSegment>?` — derive from the type's FQDN: split on `::`, take the last segment, prepend `is-`, append `?`, rejoin with the namespace. (`my::Shape` → `my::is-Shape?`; `myapp::Voltage` → `myapp::is-Voltage?`. Mirror Record.wat's naming.)
- **params** `[v]`; **param_types** — the predicate accepts ANY value (∀T); use a fresh type-param the way conforms?'s own value-arg is typed (check how conforms?'s scheme types arg 0).
- **body** `WatAST` = `(:wat::core::conforms? v :<FQDN>)` — a `WatAST::List` of `Keyword(":wat::core::conforms?")`, `Symbol(v)`, `Keyword(":<FQDN>")`.
- **ret** `:wat::core::bool`.
- `sym.functions.insert(predicate_path, Arc::new(func))` — landing in `sym.functions` is what makes `(:ns::is-Name? x)` dispatch as a **call** (not a record field-access — the pre-stone probe showed the field-access fallback; registering as a function fixes it, same as accessors).

Skip Alias. Call the pass from `src/freeze.rs` right after `register_types`, alongside `register_{struct,enum,newtype}_methods` (freeze.rs ~853-864).

### 2. Unify Record.wat's predicate body (wat/Record.wat)

Switch the emitted `is-<Name>?` body from `(:wat::core::= (:wat::core::type v) "fqdn")` to `(:wat::core::conforms? v :<class-fqdn>)`. One template line. Naming + signature unchanged.

## Read in order

1. `docs/arc/2026/05/237-polymorphism-consolidation/DESIGN-STONE-237.6.md` — sub-DESIGN (the convenience-not-mechanism doctrine, two emission sites, D-table).
2. `tests/probe_arc237_stone6_is_predicate.rs` — **LOAD-BEARING** 10 contracts. The union ones (probe_07/08/09) are the payload conforms? unwraps.
3. `src/runtime.rs:2852` (`register_struct_methods`) — the Function-synthesis template to mirror.
4. `src/runtime.rs:3057` area (enum constructor synthesis) — how a `WatAST` body is built from `WatAST::Keyword`/`Symbol`.
5. `src/freeze.rs:853-864` — where `register_{struct,enum,newtype}_methods` are called; add `register_type_predicates` here.
6. `wat/Record.wat` (the `is-<Name>?` expansion, ~line 50) — the body to switch to conforms?.
7. how conforms?'s value-arg is typed (its TypeScheme / infer_list arm) — to type the predicate's `v` param.

## Discipline

- Modify `src/runtime.rs` + `src/freeze.rs` + `wat/Record.wat` ONLY. No check.rs unless a predicate scheme demands it. No holon-rs (STOP-5). No new `Value` variant.
- conforms? + `declared_type_name` (237.5/.fix) are UNTOUCHED.
- typealias gets NO predicate. The pass skips `TypeDef::Alias`.
- Do NOT commit. Do NOT migrate Dispatch/arithmetic (237.7/237.8).

## STOP triggers (REJECTION — not permission to defer)

1. Lib baseline < 827.
2. 237.5 (`probe_arc237_stone5_conforms` 12/12), 237.5.fix (`probe_arc237_stone5fix_nominal` 12/12), 234.0 `type` (8/8), 237.1–237.4 — any regression.
3. holon-rs touched (STOP-5).
4. Files outside src/runtime.rs + src/freeze.rs + wat/Record.wat touched.
5. probe doesn't reach 10/10.
6. Record.wat predicate still computes via `(= (type v) …)` (must be `(conforms? …)` — the one-way unify is part of the stone).
7. 90 min elapsed.

## FM 2-bis evidence

`tests/probe_arc237_stone6_is_predicate.rs` (committed `e8cf382b`) — 10 contracts. Pre-stone 1/10: record `is-Circle?` green (Record.wat); the four TypeEnv-form predicates fail (don't exist). Post-stone 10/10.

## SCORE doc

`docs/arc/2026/05/237-polymorphism-consolidation/SCORE-STONE-237.6.md` (NEW). 10-row scorecard + the predicate-naming rule + confirmation that all `is-<Name>?` bodies (incl. Record.wat) compose conforms? + line counts + honest deltas. Mirror SCORE-STONE-237.2 shape.

## Calibration

One synthesis pass (mirror register_struct_methods) + freeze wiring + one Record.wat line + probe. **Target band: 30–55 min Mode A; 90 STOP.** Per `feedback_stone_briefs_cite_prior_score`: mirror Stone 237.2 SCORE shape.
