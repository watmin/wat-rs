# DESIGN — Stone 2a: `alpha-match` (the rete single-fact matcher)

> Arc 278 stone 2, part a. The per-fact condition matcher, **purpose-built for rete** (NOT `form::matches?` —
> scrutinized + rejected: it's a compile-time special form returning `bool` + binding into an `Environment`
> over a *literal* pattern; the engine needs a pure data-in/data-out matcher). Given a condition-form (DATA)
> and a fact (record), return the binding map iff the fact matches. This is the genuinely-new per-fact layer;
> stone 2b wires it into `insert` + alpha-memory, stone 3 adds the cross-fact join.

## Contract (the ONE decision)

```
(:wat::rete::alpha-match [cond <- :wat::WatAST  fact <- :wat::Record]
  -> :wat::core::Option<wat::core::PersistentMap>)
```
- `cond` — a condition form `(:FactType clause …)`, arriving as `Value::wat__WatAST(Arc<WatAST>)` (what `quote`
  produces; runtime.rs:8708). It's the `Rule.lhs` element.
- `fact` — a record (`Value::Struct{type_name, fields}`).
- Returns `Some(bindings)` iff the fact's type == the condition's head keyword AND every clause holds;
  `None` otherwise. **Clara no-error semantics**: wrong type / missing field / failed constraint → `None`,
  never raise.
- `bindings` — a `PersistentMap` keyed by the logic-var **name string** (incl the leading `?`, e.g. `"?t"`)
  → the bound value. Heterogeneous values → bare `PersistentMap` (0d.1 makes it fold-able).

## The matcher — PURE, its OWN classifier (NOT `classify_clause`)

`classify_clause` is `form::matches?`'s grammar (`:keyword` field access, bare `=`/`<` heads, `where`). Our DSL
is different — `<-` bind, **FQDN** value ops, rete-namespaced combinators — so the matcher has its own small
classifier. Extract the `WatAST` from `cond`; it must be a `List` whose head is the type `Keyword`. If
`fact.type_name` ≠ that head (leading `:` stripped on both) → `None`. Then fold the clauses left→right,
threading the bindings map (start empty); classify each clause `List` by shape:

| clause shape | kind | semantics (v1) |
|---|---|---|
| `[Symbol(?v), Symbol(<-), Keyword(:field)]` | **bind** | `bindings["?v"] := fact.<field>` (field name = `:field` minus `:`, indexed via the type registry); field absent → `None` |
| `[Keyword(:wat::core::<op>), a, b]`, op ∈ `= not= < > <= >=` | **constraint** | resolve `a`,`b` → compare per op; false → `None` |
| `[Keyword(:wat::rete::and), clause…]` | **and** | every sub-clause holds (thread bindings) |
| `[Keyword(:wat::rete::or), clause…]` | **or** | ≥1 sub-clause holds |
| `[Keyword(:wat::rete::not), clause]` | **not** | the sub-clause must NOT hold (binds nothing) |
| `[Keyword(:wat::rete::where), expr]` | **where** | **OUT OF SCOPE v1 → STOP** (arbitrary-expr eval = stone 6 `TestNode`) |

**Operand resolution (the purity crux):** an operand is resolved ONLY from `{bindings, field, literal}` —
- `Symbol(?v)` → `bindings["?v"]` (error/`None` if unbound — within one condition a `?v` is bound by a prior
  `<-` clause; cross-condition `?v` reuse is the JOIN, stone 3).
- `Keyword(:field)` → `fact.<field>` (direct field read; lets a constraint reference a field without binding).
- literal (`IntLit`/`StringLit`/`BoolLit`/…) → its `Value`.
- **NEVER `eval_inner`, NEVER an `Environment`.** v1 operands are var/field/literal only (nested exprs are
  `where`/stone 6), so pure resolution is total. This is the structural difference from `form::matches?`.

Compare with `values_equal` (equality) + the numeric ordering match (mirror `eval_form_matches`'s `Compare`
arm, runtime.rs ~10615) — but as a **pure value-level** comparison, no env.

## Out of scope (affirmative cuts)
- `(where …)` / any nested-expression operand → stone 6 (STOP if hit).
- **Cross-fact `?var` unification (the JOIN)** → stone 3. alpha-match is SINGLE-fact: every `?var` is bound by
  THIS condition's own `<-` clauses; a `?var` reused across conditions is the join key, resolved by the beta
  network, not here.
- fact-binding `(?f <- :FactType …)` (whole fact) → later.
- `insert` / alpha-memory / running facts through the network → stone 2b (consumes this matcher).
- `record->map` for `Element.fact` → stone 2b (`eval_record_to_map`, runtime.rs:13075). The matcher reads the
  `Value::Struct` directly.

## Home
New `src/rete/` home — the engine's Rust-primitive home (first rete Rust primitive; collections → `src/collection/`,
rete → `src/rete/`): `src/rete/matcher.rs` + `src/rete/mod.rs` + a `mod rete;` decl in `src/lib.rs`. New
intrinsic head `:wat::rete::alpha-match` dispatched in `runtime.rs`; a check scheme in `check.rs`
(`WatAST × Record → Option<PersistentMap>`). NO new `Value` variant. NO change to `form::matches?`/`form_match.rs`.

## Proof (FM-2-bis — RED at HEAD)
`tests/probe_arc278_2a_alpha_match.rs` (RED, un-ignore on green): `Record::def :user::Temp [value <- i64]`.
- **match + bind** → `(alpha-match (quote (:user::Temp (?t <- :value) (:wat::core::> ?t 20))) (:user::Temp 25))`
  → `Some({"?t": 25})`; the probe digs out `?t` == `25`.
- **constraint fails** → same cond, `(:user::Temp 15)` → `None` (15 ≯ 20).
- **wrong type** → `(quote (:user::Other (?t <- :value)))` + `(:user::Temp 25)` → `None`.
RED at HEAD: `:wat::rete::alpha-match` unknown (Temp/quote/Option/PersistentMap all exist).

## Four questions
- **Obvious?** YES — conditions-as-data → a matcher taking the data + a fact, returning the bindings.
- **Simple?** YES — one pure fold with a tiny shape-classifier; no Environment, no eval; additive (nothing existing changes).
- **Honest?** YES — bindings extraction + no-error; reuses only value-level helpers; no bool-shoehorn.
- **Good UX?** YES — a clean, inspectable matcher primitive the WAT engine calls; `form::matches?` untouched.

## Blast radius
`src/rete/matcher.rs` + `src/rete/mod.rs` (new) · `src/lib.rs` (`mod rete;`) · `runtime.rs` (one dispatch arm) ·
`check.rs` (one scheme) · the probe. NO new Value variant. NO `form::matches?` change. No git in the worker.
