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

**Expansion** (mirrors `deftest` — a zero-arg `defn` whose RETURN TYPE marks it for 5b's reflection):
```
(:wat::core::defn :weather::cold-and-windy [] -> :wat::rete::Rule
  (:wat::rete::Rule
    "weather::cold-and-windy"
    (:wat::core::PersistentVector
      (:wat::core::quote (:weather::Temperature (?loc <- :location) (?c <- :celsius) (:wat::core::< ?c 20)))
      (:wat::core::quote (:weather::WindSpeed (?loc <- :location) (?k <- :kph) (:wat::core::> ?k 30))))
    (:wat::core::PersistentVector
      (:wat::core::quote (:wat::rete::insert (:weather::ColdAndWindy ?loc))))))
```

**Macro shape:** `[name <- :wat::WatAST  & rest <- :wat::core::Vector<wat::WatAST>]  -> :wat::WatAST`.
`rest` = `(:when  <conds-vector-node>  :then  <insert-form>…)`. The macro must:
1. **name → string**: `(:wat::core::keyword/to-string name)` then strip a leading `:` if present (the `Rule`'s
   `name` field is the fqdn WITHOUT colon — match `:wat::core::type`'s output convention).
2. **parse the keyword sections** of `rest`: locate `:when` (its value is the next element — a `[...]` vector
   node whose `ast->children` are the conditions) and `:then` (its value is the remaining elements — the N
   insert forms). For v1 assume the canonical order `:when` then `:then` (STOP if a robust general parse is
   needed — name it).
3. **quote-and-assemble each side**: build `(:wat::core::PersistentVector ~@(map (fn [c] `(:wat::core::quote ~c)) conds))`
   for lhs, and the same over the `:then` forms for rhs (uses the macro-eval engine's `map` + `~@` splice +
   nested quasiquote — arc 249).
4. **emit** the `defn` above.

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
- `wat/rete.wat` — `defrule` macro + `query` fn (+ a tiny `strip-leading-colon` helper if needed, or inline).
- `tests/probe_arc278_5a_defrule_query.rs` — the probe.

## Out of scope = REJECTED
- **`collect-rules`** (the reflection that gathers rules by namespace) — **5b** (Rust primitive); 5a collects
  the one rule manually by calling its generated fn.
- **`defquery` / `QueryNode` / parameterized queries** — later (the north-star uses `query`, not `defquery`).
- **`Snapshot`** (4d), the Rust kernel (perf arc).
- No Rust change. No record/signature change.
