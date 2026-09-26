# WEIGH — STONE 255.49: the bound's last unknowns — ACCEPTED (measurement)

**Executor: grok via pulsare, commit `657ab650a`** (the SCORE only; the tree was clean, and grok removed its
worktree). Weighed by the orchestrator on 2026-09-26. The instrument ran over 2286 `.wat` and 376 `.wat.bad`.

## Re-verified by the orchestrator

| claim | verified at |
|---|---|
| a generic edge is stored only for a `Path` or `Parametric` child; a tuple returns early | `src/types.rs:1473-1480` |
| `:wat::holon::Record <: :wat::core::Record` is a built-in seed | `src/types.rs:3282-3289` |
| a variant registers `<:` its enum, and nothing above the enum | `src/types.rs:1686` |
| the p11 stdlib site: `(+ acc (Option/expect (map/get (Element/bindings e) var)))`, where the map's value type is unpinned | `wat/rete/acc.wat:40-47` |
| the rigid `:T` hit: `assert-eq :- [T]` calls `=` on two `:T` | `wat/test.wat:61-64` |
| the lone both-records site asserts `(= Pt HPt)` is **false**, a comparison that can never be true (`Record/same-data?` is the cross-type tool, one line above) | `tests/types/probe_arc237_sC3_macro_split.wat:38-41` |

The instrumented counts (2016 equality sites, 16 p11 sites, 15 end-of-definition hits) are weighed on method and
the sites above; not re-run.

## The reading

1. **Tuples cannot be members today.** `extend-type` with a tuple child loads and admits nothing: generic edges
   drop tuples, and `assignable` reads edges only path-to-path. Six corpus sites order tuples, all
   `tests/types/ord_tuple_*`. No `=`/`not=` site has a tuple operand. **No live code orders a tuple.**
2. **Equatable by root edge:** one edge on `:wat::core::Record` covers records and holon records; structs need
   `:wat::core::Struct`. **Enums have no root** and **newtypes register no edge**; both are equatable today by a
   kind test and an inner recursion.
3. **Equality's compatibility rules carry almost nothing.** Of 2016 sites: 2000 are `unify`; **2** are
   subtype-only, both an enum against its own variant, which variant widening (ordering already widens,
   `src/check.rs` 13636-13648) would admit; **1** is both-records, the always-false `Pt`/`HPt` test; **10** are
   both-numeric `i64`/`f64`, the Numeric cross clause.
4. **p11: all 16 sites are ambiguous.** None is unique. The rule refuses every one, including one stdlib site
   (`wat/rete/acc.wat:45`), whose cause is an unpinned map value type upstream.
5. **Refuse at the end of the definition sees all 15 hits.** The three rigid `:T` hits (`assert-eq`,
   `dedupe-walk`, `eq-generic`) are C's intended case: the definition declares `(T <- Equatable)`.
6. The binder syntax the rulings write (`(T <- X)`, `(B <- A)`) does not parse yet. Building that is the work,
   not a finding against the ruling.

## Decisions for the builder

- Tuples: how "each element" is declared.
- Equality: whether the compatibility rules survive, or equality is one bounded clause plus variant widening
  plus the Numeric cross clause.
- Enums and newtypes: how they reach `Equatable`.
