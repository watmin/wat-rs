# DESIGN — Stone 5a: `defrule` (the rule macro) + `query` (read derived facts)

The first slice of stone 5 — the homoiconic surface, the wat half. `defrule` turns the readable rule form into
a `Rule`-producing definition; `query` reads derived facts of a type out of a fired session. Both PURE WAT.
The reflection that auto-gathers rules (`collect-rules`) is **5b** (a Rust primitive); the north-star greens
there. 5a is tested by collecting the one rule manually.

## Why
`compile` takes `PersistentVector<Rule>` and the probes have been hand-building `Rule` records with quoted
condition/insert forms (`probe_arc278_4a:35-38`). `defrule` is the ergonomic surface over that construction;
`query` is the read-out the north-star's last line needs (`(:wat::rete::query fired :weather::ColdAndWindy)`).

## `defrule` — the macro

**Surface (from the north-star, `probe_arc278_northstar_cold_and_windy.rs:32-43`):**
```
(:wat::rete::defrule :weather::cold-and-windy
  :when [(:weather::Temperature (?loc <- :location) (?c <- :celsius) (:wat::core::< ?c 20))
         (:weather::WindSpeed    (?loc <- :location) (?k <- :kph)     (:wat::core::> ?k 30))]
  :then (:wat::rete::insert (:weather::ColdAndWindy ?loc)))
```

**Expansion** (mirrors `deftest` — a zero-arg `defn` whose RETURN TYPE marks it for 5b's reflection). To keep
the MACRO trivial — NO per-element quoting, NO nested quasiquote (the macro-eval engine's sharpest edge, where
the first attempt looped) — it QUOTES the whole `:when` vector and the `:then` forms, and a plain RUNTIME helper
`make-rule` does the per-element split:
```
(:wat::core::defn :weather::cold-and-windy [] -> :wat::rete::Rule
  (:wat::rete::make-rule
    "weather::cold-and-windy"
    (:wat::core::quote [(:weather::Temperature (?loc <- :location) (?c <- :celsius) (:wat::core::< ?c 20))
                        (:weather::WindSpeed    (?loc <- :location) (?k <- :kph)     (:wat::core::> ?k 30))])
    (:wat::core::quote [(:wat::rete::insert (:weather::ColdAndWindy ?loc))])))
```

**`make-rule` (RUNTIME fn — the per-element split, trivial):**
`(:wat::rete::make-rule [name <- :wat::core::String  when-ast <- :wat::WatAST  then-ast <- :wat::WatAST] -> :wat::rete::Rule)`
— `when-ast`/`then-ast` are quoted VECTOR nodes; `(:wat::core::ast->children when-ast)` yields the per-element
condition/insert WatASTs → convert to `PersistentVector<wat::WatAST>` (foldl `conj`; `ast->children` may return a
std `Vector`) → `(:wat::rete::Rule name lhs-pv rhs-pv)`. (`make-rule` is also the single seam for FUTURE rule
metadata — content-hash, docstrings — which ride `defn`'s own metadata-decoration arc; **NOT built here**.)

**Macro shape:** `[name <- :wat::WatAST  & rest <- :wat::core::Vector<wat::WatAST>]  -> :wat::WatAST`.
`rest` = `(:when  <conds-vector-node>  :then  <insert-form>…)`. Trivial now:
1. **name → string**: `(:wat::core::ast-name name)` — handles Symbol AND Keyword nodes (precedent
   `service.wat:356`, `deporder.wat:116`); then strip a leading `:` iff present (the `Rule.name` is the fqdn
   WITHOUT colon, matching `:wat::core::type`). **NOT `keyword/to-string`** — that's for a keyword VALUE; on a
   node it's the wrong tool (this is the loop the first attempt hit).
2. **grab the sections** (assume canonical `:when` then `:then` order; STOP if a general parse is needed):
   `when-vec = (:wat::core::get rest 1)` (the `[...]` node); `then-forms = (:wat::core::drop rest 3)` (inserts).
3. **emit** via quasiquote — quote the whole `when-vec`; splice `then-forms` into a vector literal (the
   `Record.wat:114` `[~@…]` idiom):
   `` `(:wat::core::defn ~name [] -> :wat::rete::Rule (:wat::rete::make-rule ~name-str (:wat::core::quote ~when-vec) (:wat::core::quote [~@then-forms]))) ``

⚠ **SCOPE GUARD:** `defrule` needs ONLY the name string from its own `name` node (compile-time). It does NOT
enumerate or reflect over functions — that's `collect-rules` = **5b** (a Rust primitive). If you reach for a
"list/extract fn names" primitive, you've drifted into 5b; STOP. (That drift is what made the first attempt loop.)

## `query` — read derived facts (pure wat)

`(:wat::rete::query [session <- :wat::rete::Session  ty <- :wat::core::keyword] -> :wat::core::PersistentVector)`:
- normalize the type keyword to the `(:wat::core::type fact)` string form: `(:wat::core::keyword/to-string ty)`
  then strip a leading `:` if present → e.g. `"weather::ColdAndWindy"`.
- flatten `production-memory`'s values into one `PV<:wat::Record>` (the 4c idiom:
  `foldl` over `(:wat::core::PersistentMap/values (:wat::rete::Session/production-memory session))`, inner
  `foldl` `conj`).
- `filter` by `(:wat::core::= (:wat::core::type f) <ty-string>)`; return the matching `PV`.
- The north-star wraps this in `length` → expects 1.

## The one contract decision (pinned)
`defrule` expands to a zero-arg `defn` returning `:wat::rete::Rule` (the reflection marker for 5b);
`Rule.name` = the fqdn without leading colon. `query` returns a `PersistentVector` of the derived facts whose
`(:wat::core::type f)` equals the normalized type string (empty PV if none — never raises).

## Files touched
- `wat/rete.wat` — `defrule` macro + `make-rule` runtime fn + `query` fn (+ a tiny `strip-leading-colon` and a
  `children->pv` helper if needed, or inline).
- `tests/probe_arc278_5a_defrule_query.rs` — the probe.

## Out of scope = REJECTED
- **`collect-rules`** (the reflection that gathers rules by namespace) — **5b** (Rust primitive); 5a collects
  the one rule manually by calling its generated fn.
- **`defquery` / `QueryNode` / parameterized queries** — later (the north-star uses `query`, not `defquery`).
- **`Snapshot`** (4d), the Rust kernel (perf arc).
- No Rust change. No record/signature change.
