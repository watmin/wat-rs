# DESIGN-STONE — class-scan query harvest

> **Origin (2026-08-22).** `fanout_three_leftover_split`
> ranked the three leftovers. With-query wall **65.89**,
> without **26.67**, delta **39.22**. Named A:
> harvest:query **14.73** + out:query **3.02**. Unmarked
> query Alpha→RootJoin ~21. B compiled-rhs_net 3.07
> (2l). C out:production 3.61 (2u-dead). This stone
> internes A.

## The enemy

`compile-query` of `(?fact <- :Type)` mints Alpha +
RootJoin + QueryNode. Derived facts of that type
activate the query alpha and seed 40k RootJoin
tokens so harvest can `pmap_from_span` them.
Production already holds the Pairs. The query chain
is a second write of the same occupancy.

Stratified fire already keeps QueryNodes off the
production slice. Unstratified does not.

## The algorithm

A query-only alpha: every child is a RootJoin whose
every child is a QueryNode; compiled cond is
`fact_bind` with empty ops (no field constraints).

```
alpha_activate: skip query-only alphas
harvest: for each class-scan query
    class = alpha_pattern type_head
    var   = fact_bind
    for fact in input ∪ derived where class matches
        emit {var: fact}
    else existing beta harvest
```

Query-memory stays name → vector of binding maps.
Constrained / join queries keep the chain.

## ★ THE ONE CONTRACT DECISION

**Skip the query-only chain. Harvest the closed fact
bag by class.** Do not skip an alpha that still feeds
production. Dual-impl WHAT is unchanged.

## The gate

1. `fanout_three_leftover_split` with-query maps =
   40,000. with-query wall drops ≥ 1 ms vs 65.9.
2. 7strat including three-stratum. rete lib.
3. clippy `--lib -D warnings`.

## Predicted win

Independent guess (written first): with-query wall
**65.9 → ~40**. The ~21 ms RootJoin walk dies.
Harvest 14.73 remains (40k one-entry PMaps) and is
the next leftover if still ≥ 1 ms.

## Blast radius

`src/rete/kernel/fire/{mod,delta}.rs`. No `.wat`.
No Session field. No PMap representation change.

## Out of scope = REJECTED

- Skip constrained query alphas.
- Session-Vec. Skip freeze. intern `names`. 297.
- PMap Array1 (runtime-wide).

## Sequencing

1. Classify query-only alphas from the arm.
2. Skip in activate. Harvest by class.
3. Weigh. Stop.

## Weigh (2026-08-22) — LANDED

`fanout_three_leftover_split` with-query wall
**65.89 → 49.59** (−16.3). 40k maps held.
7strat 3/3 including three-stratum. 7b / 7exists / 8b
green. Clippy `--lib -D warnings` silent.
Grid fanout `[40000]` wat-ns **58.1 → 42.8**
(ratio 3.17 → 4.55). `:match`.

The unmarked Alpha→RootJoin walk is dead. Leftover
A is harvest:query **16.91** + out:query **3.04**
(40k one-entry PMaps). Next if ≥ 1 ms. Do not
intern `names`. Do not skip freeze.

Import/Export AlphaNode has no tests AST. Class-scan
without a type name would skip activate and harvest
empty — Hit 1→0. Classify only when
`alpha_pattern` resolves. Floor RED
`.floor/2026-08-22T21-44-17Z/` (8 Import fires);
fix: AST gate. Do not re-run that red.
