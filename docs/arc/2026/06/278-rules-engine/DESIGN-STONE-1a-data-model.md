# DESIGN — Stone 1a: the rete data model (`wat/rete.wat` born)

> Arc 278 stone 1, part a — the engine's DATA MODEL in wat: the records + the node sum + the `Session`
> record, EDN-round-trippable, with a DAG render. NO compile, NO fire (those are 1b+). This is the first
> WAT engine stone; it pins the shapes everything builds on. Names by the 3rd intueri cast (2026-06-17).

## What it delivers

`wat/rete.wat` exists, loaded into the stdlib, holding: the data-flow records (`Token`/`Element`/
`Activation`), the `Rule` record, the MVP node records + the `Node` defenum sum, and the `Session` record
(the whole caller-facing engine state). All on `:wat::core::PersistentMap`/`PersistentVector` (stone 0).
EDN round-trips for free (it's all EDN data). A `render-dag` produces an inspectable graph string. A probe
hand-builds a tiny network, round-trips it, renders it, asserts structure.

## The vocabulary (intueri-blessed, 3rd cast)

```
;; ── data flow ──────────────────────────────────────────────────────────────
(:wat::Record::def :wat::rete::Token
  [matches  <- :wat::core::PersistentVector      ; provenance/support chain: [[fact node-id] …]
   bindings <- :wat::core::PersistentMap])       ; {?var → value}; flows LEFT through joins
(:wat::Record::def :wat::rete::Element
  [fact     <- :wat::core::PersistentMap          ; a fact (record-as-map for v1; see note)
   bindings <- :wat::core::PersistentMap])        ; alpha-bindings; flows RIGHT into a join
(:wat::Record::def :wat::rete::Activation
  [production-id <- :wat::core::i64               ; the ProductionNode queued to fire (intueri: not node-id)
   token        <- :wat::rete::Token])

;; ── rules as data ──────────────────────────────────────────────────────────
(:wat::Record::def :wat::rete::Rule
  [name <- :wat::core::String                     ; the namespaced rule name
   lhs  <- :wat::core::PersistentVector            ; conditions (form::matches?-shaped clauses)
   rhs  <- :wat::core::PersistentVector])          ; consequence forms (data; pure — applied by a consumer)

;; ── the network nodes (MVP set; negation/test/accumulate/expr-join arrive at stones 6–8) ──
(:wat::Record::def :wat::rete::AlphaNode
  [id <- :wat::core::i64   tests <- :wat::core::PersistentVector   children <- :wat::core::PersistentVector])
(:wat::Record::def :wat::rete::RootJoinNode
  [id <- :wat::core::i64   children <- :wat::core::PersistentVector   binding-keys <- :wat::core::PersistentVector])
(:wat::Record::def :wat::rete::HashJoinNode
  [id <- :wat::core::i64   children <- :wat::core::PersistentVector   binding-keys <- :wat::core::PersistentVector])
(:wat::Record::def :wat::rete::ProductionNode
  [id <- :wat::core::i64   rule-name <- :wat::core::String])
(:wat::Record::def :wat::rete::QueryNode
  [id <- :wat::core::i64   query-name <- :wat::core::String   param-keys <- :wat::core::PersistentVector])
(:wat::core::defenum :wat::rete::Node
  AlphaNode | RootJoinNode | HashJoinNode | ProductionNode | QueryNode)   ; exact defenum syntax per wat/service.wat

;; ── the session (the whole engine state — intueri: NOT WorkingMemory) ────────
(:wat::Record::def :wat::rete::Session
  [network           <- :wat::core::PersistentMap   ; id → Node  (the compiled DAG, id-indexed)
   rules             <- :wat::core::PersistentVector
   alpha-memory      <- :wat::core::PersistentMap    ; node-id → {join-bindings → [Element …]}
   beta-memory       <- :wat::core::PersistentMap    ; node-id → {join-bindings → [Token …]}
   production-memory <- :wat::core::PersistentMap    ; node-id → {token → [[facts] …]}  (the TM support store)
   facts             <- :wat::core::PersistentVector
   next-id           <- :wat::core::i64])
```

> **EXACT SYNTAX:** mirror `wat/service.wat`'s `defenum` (`:wat::service::Outcome<S,R>`) for the `Node` sum
> and `wat/lint.wat`'s `Record::def` (`:wat::lint::FixEdit`) for the records. Confirm the precise defenum
> variant-arm form on disk before writing (don't guess the `|` shape — read service.wat).

## The ONE contract decision

**`Element.fact` is a `PersistentMap` (record-as-map) for v1.** Facts are typed records; a Record IS
representable as a field→value map. v1 stores the fact's field-map in `Element.fact` (keeps the node records
homogeneous + EDN-trivial). When stone 2 (alpha activation) wires `form::matches?` over real fact records,
revisit whether `Element.fact` should hold the live record value vs the map — but for the 1a data model, the
map keeps it simple and round-trippable. (Flag, not a deferral: 1a only needs the SHAPE to exist + round-trip.)

## render-dag

`(:wat::rete::render-dag session) -> :wat::core::String` — walk `network` (id→Node), emit a readable graph
(node id · kind · children ids). Mirror a simple text/mermaid edge list. This is the "network as data is
real + inspectable" proof; the full inspection/`fact→explanation` lands with fire (stone 4).

## Proof (FM-2-bis — RED at HEAD)

`tests/probe_arc278_1a_data_model.rs` (RED, un-ignore on green): build a tiny `Session` by hand — a
`RootJoinNode`(id 0)→`ProductionNode`(id 1), put them in `network` (PersistentMap), construct the `Session`,
then (a) `render-dag` returns a string naming both nodes + the edge; (b) EDN round-trip the `Session`
(`value_to_edn_string`→`edn_string_to_value`) → equal. RED at HEAD: `:wat::rete::Session` etc. are unknown.

## Out of scope (affirmative cuts)

- **compile** (rule-set → network with sharing) — stone 1b.
- **the non-MVP nodes** (ExpressionJoin/FilterJoin, Negation, Test/Predicate, Accumulate) — stones 6–8 add
  the records + their `Node` variants then.
- **fire / insert / query / defrule / collect-rules** — stones 2–5.
- the `Rulebase`-vs-`Session` split (durability nudge) — on need, not now.

## Four questions

- **Obvious?** YES — the records name their domain nouns; `Session` is the whole-engine handle.
- **Simple?** YES — pure data records + a defenum + one render fn; no behavior.
- **Honest?** YES — `Session` (not WorkingMemory) names the whole; EDN round-trip is real, not faked.
- **Good UX?** YES — the data model reads as the RETE architecture; everything is inspectable EDN.

## Blast radius

NEW `wat/rete.wat` (+ register in `src/stdlib.rs` deporder, AFTER `wat/Record.wat` — it uses `Record::def`/
`defenum`/PersistentMap). + a deftest in `wat-tests/` and/or the Rust probe. NO Rust changes (pure wat on
the stone-0 collections). No git in the worker.
