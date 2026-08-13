# BRIEF — `defrule` lifts its `where` bodies

> **Design + rulings:** `DESIGN-STONE-defrule-splits-at-expansion-time.md`. Read its § THE SHAPE,
> § PROVEN, and § REFUTED first. **Do not re-derive them** — four shapes were killed by run getting
> here, and § REFUTED says which and why.
>
> **Names are CAST and ratified** (`wat-scripts/intueri/defrule-lifted-where-naming.wat.intueri`):
> the lifted body is `:<rule-fqdn>$where<N>`, the mention's binder is `$where<N>`.

## The work, one paragraph

`defrule` currently quotes its `:when` vector whole, so a `where` body — which is code — sits inside
a runtime `quote` and its dependencies are never shipped to a spawned child. Change the macro so
that, for each `(:wat::rete::where <body>)` condition, it emits **a lifted top-level defn holding
that body**, rewrites the quoted condition to **call** that defn, and binds the lifted name once in
a `let` around the `make-rule` call so the closure walker collects it. `Rule` does not change; the
user-facing `defrule` surface does not change; no Rust changes.

## The rooms, in order

| room | why you are here |
|---|---|
| **`wat/rete.wat:2406-2427`** | `defrule`'s macro — **the subject**. Today it takes `when-vec`/`then-vec` and quotes both verbatim (`:2425`). This is the only thing that changes. |
| **`wat/core.wat:1147-1167`** | **The worked reference.** The kwargs `defn` macro already emits `(do <decl> <decl> <decl>)` with a computed name minted by `keyword-node` (`:1149`) and a local binder spliced by `~(:wat::core::symbol-node "…")` (`:1163`). Copy this shape. |
| **`wat/core.wat:870-873`** | How the `$`-suffixed fqdn string is built — `string::interpolate ":{b}$impl{p}"`. Same technique for `":{rule}$where{n}"`. |
| **`wat/rete.wat:2364-2382`** | `make-rule` — reads `:when`/`:then` as quoted vector nodes via `ast->children`. It keeps working unchanged; the quoted condition simply now holds a call form. |
| **`src/rete/collect.rs:56-60`** | The discovery contract you must preserve: a rule is *a zero-arg fn whose return type is `:wat::rete::Rule`*. The lifted `$where<N>` defns return `:bool`, so they are excluded for free. |
| **`wat/query.wat:189`** | The **second** producer — `sift-rules-defsvc` emits the same `make-rule` shape from a generated service. It moves in this strike (STOP-2). |

**Worked references to copy, all green and committed:**

- `wat-scripts/scratch-pad/probe-arc278-macro-mints-a-lifted-toplevel-defn.wat` — a macro minting a
  computed-name top-level defn **plus** its consumer, in one expansion. This is the exact mechanism.
- `wat-scripts/scratch-pad/probe-arc278-lift-mention-ships-transitively.wat` — `PC 7 · BASE 5 ·
  MENTION-1 7`: one mention collects the lifted fn and everything it transitively calls.
- `wat-scripts/scratch-pad/probe-arc278-where-body-dep-not-shipped.wat` — `PC 6 · BASE 5 · SUBJECT
  5`: the defect being fixed. **Its SUBJECT must rise to meet its POSITIVE-CONTROL when you are
  done**, and its closing verdict line must be rewritten to say what it then proves (its header
  says so).
- A prior strike of comparable shape, for the artifact rhythm: `BRIEF-compiled-conditions.md`.

## Implementation sketch

For a rule with one `where`, the macro emits:

```clojure
;; 1. the LIFTED body — an ordinary top-level defn, computed name via keyword-node
(:wat::rete::core::defn :usr::ok-rule$where0 [?c <- :wat::core::i64] -> :wat::core::bool
  (:usr::big? ?c))

;; 2. the rule — the quoted condition CALLS the lifted defn; the let-binder MENTIONS it
(:wat::core::defn :usr::ok-rule [] -> :wat::rete::Rule
  (:wat::core::let [~(:wat::core::symbol-node "$where0") :usr::ok-rule$where0]
    (:wat::rete::make-rule "usr::ok-rule"
      (:wat::core::quote [(:usr::Temp (?c <- :c))
                          (:wat::rete::where (:usr::ok-rule$where0 ?c))])
      (:wat::core::quote [(:usr::Hot :c ?c)]))))
```

Both forms come back from the macro inside one `(:wat::core::do …)`, exactly as `core.wat:1147`
does. `N` counts `where` conditions **by position in `:when`**, starting at 0 — order is load-bearing
because a `where` sits at a position in the rete network.

The lifted defn's parameters are the `?var`s the body references; the macro can read them from the
`<-` binders in the preceding conditions of the same `:when` vector.

## Blast radius

`wat/rete.wat` (the `defrule` macro) and `wat/query.wat` (the sift generator). **No `src/` changes and
no `Rule` shape change in this strike.** The Rust hole (`Boundary::MakeRule`, `is_where_form`, and
the three `make_rule` descents) is expected to become dead once this lands, and deleting it is a
**separate follow-on** whose acceptance test is its own floor run — leave it alone here.

## STOP triggers

1. **STOP-1** — if the expansion cannot keep `defrule` producing a zero-arg defn returning
   `:wat::rete::Rule`, halt and report. `collect-rules` discovers rules by that signature and nothing
   else (`src/rete/collect.rs:56-60`).
2. **STOP-2** — `wat/query.wat:189` emits the same shape from a generated service and moves with
   this change. If its generator cannot carry the lift, halt and report rather than migrating one
   producer and leaving the other — that splits the corpus into two rule dialects.
3. **STOP-3** — if a `where` body references a `?var` the macro cannot resolve to a binder in the
   same `:when` vector, halt and report the rule. Do not guess a parameter list.
4. **STOP-4** — if a rule carrying **two or more** `where` conditions does not produce two distinct
   lifted defns with two coexisting binders, halt and report. **This case is UNMEASURED** — see the
   deliverable below.

## Deliverables

1. The `defrule` change in `wat/rete.wat`, and the matching change in `wat/query.wat`.
2. **A new probe for the multi-`where` case** in `wat-scripts/scratch-pad/`, because nothing has ever
   exercised it: a rule with **two** `where` conditions, asserting two distinct lifted defns exist,
   both deps ship, and the conditions keep their order. Give it a non-vacuity control — a
   single-`where` rule in the same file — so a passing count cannot come from measuring nothing.
3. `probe-arc278-where-body-dep-not-shipped.wat` updated: its SUBJECT count now matches its
   POSITIVE-CONTROL, and its closing verdict rewritten to state the regression it now guards.
