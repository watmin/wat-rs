# Clara RETE — translated reference architecture for building `:wat::rete` (arc 278)

> **What this is.** A distilled, `file:line`-anchored map of Clara's RETE engine, extracted from source
> (`~/work/holon/clara-rules/src/main/clojure/clara/`, a read-only reference clone), to be the build
> reference each stone's BRIEF points at. **It is NOT a port spec.** Clojure → wat/Rust: reference the
> ALGORITHM, never transliterate the syntax. **Our cuts (arc 278 DESIGN) OVERRIDE Clara** — where they
> diverge, the DESIGN wins. Anchors verified against the source by the orchestrator's own grep
> (node line numbers, Token/Element, protocols, accumulator fields all matched).
>
> **Our cuts vs Clara (apply these as you read):** facts are TYPED RECORDS only (no `:fact-type-fn`, no
> map-facts); RHS is PURE (no side effects — returns data / logical-inserts derived facts); NO
> `insert-unconditional!`; NO user salience (ordering is structural — see §9); the matcher reuses
> `:wat::form::matches?` (arc 098). Persistent collections + transient-during-fire are OURS (stone 0).

## 1. Node taxonomy (engine.cljc) — richer than "alpha/beta/production"

Our DESIGN's `AlphaNode`/`BetaNode`/`ProductionNode` sketch is really this family:

| node | engine.cljc | role | activates |
|---|---|---|---|
| `AlphaNode` | :529 `[id env children activation fact-type]` | type-dispatch + intra-fact constraint test; `activation` = `(fn [fact env] → bindings|nil)`; emits `Element`s | alpha-activate/retract only (never from beta side) |
| `RootJoinNode` | :554 | the FIRST condition (no left side); wraps each Element into a fresh `Token`; `binding-keys=[]` | right only (left = no-op) |
| `HashJoinNode` | :608 | the standard two-input EQUALITY join; `binding-keys` = the hash-join key | left + right |
| `ExpressionJoinNode` | :683 `[… join-filter-fn …]` | hash-join THEN a `join-filter-fn` for non-equality constraints referencing ancestor bindings (`(= ?x (:f fact))` where `?x` came from the left) | left + right |
| `NegationNode` | :764 | `:not [Type …]` with no ancestor cross-refs; token passes downstream iff alpha-memory empty | left + right |
| `NegationWithJoinFilterNode` | :819 `[… join-filter-fn …]` | `:not` whose constraints reference parent bindings; two-sided delta logic (HARDEST node) | left + right |
| `TestNode` | :943 `[id env constraints test children]` | left-only; `test = (fn [token env] → bool)`; filters tokens | left only |
| `AccumulateNode` | :1014 | aggregation; accum-memory `[facts reduced-value]`; uses `:retract-fn` or re-accumulates | left + right |
| `AccumulateWithJoinFilterNode` | :1388 | accumulate whose condition cross-refs ancestors; caches CANDIDATE lists (no cached reduce), filters per-token | left + right |
| `ProductionNode` | :336 `[id production rhs]` | terminal; queues `Activation`; on left-retract drives the TM cascade | left only |
| `QueryNode` | :432 `[id query param-keys]` | terminal; stores tokens keyed by param values; `query` reads them back | left only |

Data records: `Token [matches bindings]` (:24), `Element [fact bindings]` (:27), `Activation [node token]` (:30), `Accumulator [initial-value retract-fn reduce-fn combine-fn convert-return-fn]` (:17).
Protocols: `ILeftActivate` (:99 left-activate/left-retract), `IRightActivate` (:107 right-activate/right-retract), `IAlphaActivate` (:189), `IAccumRightActivate` (:115).

## 2. Propagation model — the heart

- **`Token [matches bindings]` flows LEFT→RIGHT** through beta nodes. `matches` = vector of `[fact node-id]`
  (the full provenance/support chain — load-bearing for TM); `bindings` = accumulated `{:?var value}`.
  Born at `RootJoinNode/right-activate` as `(->Token [[fact node-id]] alpha-bindings)` (:584); extended at
  each join `(->Token (conj matches [fact id]) (conj bindings fact-bindings))` (:623/653).
- **`Element [fact bindings]` flows into the RIGHT side** of a join (one fact + its alpha-constraint bindings).
- **Join mechanics:** each beta node has TWO memories keyed by the same `join-bindings` sub-map:
  alpha-memory (Elements, right) and beta-memory (Tokens, left). left-activate crosses the new token against
  stored elements at that key; right-activate crosses the new element against stored tokens. **Binding
  unification at a HashJoinNode is a plain map merge** `(conj fact-binding (:bindings token))` (:623);
  monotonic, no unbinding. ExpressionJoinNode adds the `join-filter-fn` result map (:700).

## 3. Truth maintenance (logical insertion + cascade)

- `insert!` (rules.cljc:62 → engine.cljc:279) does NOT propagate immediately — it batches into
  `*rule-context*`; after the RHS returns, `flush-insertions!` (:311) calls `mem/add-insertions!`
  (memory.cljc:719) storing `{token → [[facts-batch]…]}` in `production-memory`, then queues the facts for
  alpha propagation.
- **Cascade:** when an upstream retract invalidates a token, `ProductionNode/left-retract` (:359-421) →
  `mem/remove-insertions!` returns the facts that token logically inserted → `retract-facts!` → alpha
  retraction → may recursively trigger further left-retracts = transitive TM.
- **The justification IS the token's `matches` chain.** No separate justification graph; the support is the
  provenance. (OUR engine: everything is logical insertion — we cut `insert-unconditional!`.)

## 4. Node sharing — the non-redundancy / DAG (compiler.clj)

`add-conjunctions` (compiler.clj:1113) builds the beta graph left-to-right. For each new condition it computes
the node data-structure, then **reuses an existing child node iff its data-structure equals the new node AND
its parent-set matches** (:1191-1194); else mints a new integer id (`create-id-fn`). Result: rules with an
identical condition PREFIX (same conditions, order, bindings) share all beta nodes to the divergence point.
Alpha sharing: `get-alphas-fn` groups facts by type → the set of alpha nodes per type. **This sharing IS the
"a test runs once across rules" non-redundancy** — structure and speed are one mechanism.

## 5. Working memory — transient-during-fire / persistent-at-rest (memory.cljc)

Four memory maps (`TransientLocalMemory` :425 / persistent :938):
- **alpha-memory** `{node-id → {join-bindings → [Element…]}}` (:440)
- **beta-memory** `{node-id → {join-bindings → [Token…]}}` (:451)
- **accum-memory** `{node-id → {join-bindings → {fact-bindings → [facts reduced-value]}}}` (:461) — 3-level;
  sentinels `::not-reduced` (no reduction yet) vs `::no-accum-reduced` (no entry).
- **production-memory** `{node-id → {token → [[facts-batch]…]}}` — the TM support store (:719/726).

**The boundary (validates our stone 0):** `fire-rules` calls `to-transient` at start, mutates a transient
(Clojure transients + mutable Java `LinkedList`/`TreeMap`/`PriorityQueue` for O(n) amortized removal), then
`to-persistent!` at the end → the new immutable session. External insert/retract do NOT propagate immediately
— they batch in `pending-operations` and apply at the next `fire-rules`, so one transient→persistent
round-trip per fire. **For us:** persistent-at-rest (`:wat::core::PersistentMap`) + a transient (mutable)
form for the fire loop's `assoc!` hot path — the `to-transient`/`to-persistent!` pair belongs in stone 0.

## 6. Condition grammar (dsl.clj)

- **Fact condition** (:48-103): `{:type FactType :constraints [...] :fact-binding :?x :args [...]}`.
- **Accumulator condition** (:119-126): `{:accumulator <expr> :from {:type … :constraints …} :result-binding :?r}`.
- **Boolean** (:134-148): `:and`/`:or`/`:not`/`:exists`, recursive. `:not` exactly one child. **`:or` is
  distributed to DNF** (`to-dnf`, compiler.clj:558) → multiple rules. **`:exists` is sugar — Clara expands it
  to an accumulator + test** (`extract-exists`, compiler.clj:677). (We'd decided `:exists ≡ (:not (:not X))`;
  both are valid — pick one at build, note the choice.)
- **`:test`** (compiler.clj:141): `{:constraints [...]}`, no type — predicates over bound vars. (Maps to our
  `form::matches?` `(where …)`.)
- **Constraint compile** (compiler.clj:242-315): equality `(= ?x v)` → a `let` + `assoc` into `?__bindings__`;
  non-equality → an `if` test; returns the bindings map or nil. (We REUSE `form::matches?` for this at the
  alpha level — arc 098 already does exactly this.)
- **`sort-conditions`** (compiler.clj:825): topologically orders conditions so every referenced var is bound
  by a prior condition; accumulators deferred last. (This is part of our "structural ordering" — §9.)

## 7. Accumulator protocol (accumulators.cljc + engine.cljc)

`accum` (accumulators.cljc:8) takes `{:initial-value :reduce-fn :combine-fn :retract-fn :convert-return-fn}`
(default convert = identity). **`:retract-fn` is the incremental-TM-over-aggregates hook:** on right-retract
(engine.cljc:1278), if present `(reduce retract-fn previous-reduced removed-facts)` else full re-accumulate.
`all` uses a linear drop-one retract; `sum`/`average` are O(1). **Nil-result suppression** (engine.cljc:1003):
if `convert-return-fn` returns nil the token does NOT propagate (this is how `:exists`/count=0 blocks).
Built-ins to ship (our names, DESIGN §accumulators): count/sum/min/max(+:returns-fact)/average/distinct/all/
group-by + `acc/accumulator` custom constructor.

## 8. defquery (dsl.clj:271-338, engine.cljc:432/2008)

A query = `{:name :lhs [conditions] :params #{:?p…}}`. `:params` become the `QueryNode`'s `param-keys` =
its join-keys, so the QueryNode stores tokens keyed by param VALUES. `(query session q :?p v)` looks up the
query node, validates params match, `mem/get-tokens` under those values → a seq of binding maps (synthetic
`__gen` vars filtered). A query is just a terminal node read instead of fired.

## 9. fire-rules + the agenda (engine.cljc:1751)

- **Activation** queued as `RuleOrderedActivation [node-id token activation rule-load-order …]` (memory.cljc:326).
- **activation-map** = a sorted map keyed by `[user-salience internal-salience]`, value a priority queue
  ordered by `rule-load-order` (definition order). **We CUT user-salience** → our grouping collapses to the
  structural order. **NOTE the `internal-salience`:** extracted-negation/exists sub-rules get internal-salience
  1 (fire before their parent) — this is a STRUCTURAL correctness ordering, NOT user priority. Our "no
  salience / structural order" (DESIGN §conflict-resolution) must still honor internal structural ordering for
  extracted sub-rules + the `sort-conditions` dependency order. So "structural" = condition-dependency order +
  internal sub-rule ordering; user salience is what we drop.
- **Loop → fixpoint:** pop highest group → bind `*rule-context*` → run rhs → flush
  unconditional/logical/retraction updates through alpha → recur; when no activations AND nothing pending,
  done. `:no-loop` (engine.cljc:348) guards a production from self-retriggering. Forward-chaining is exactly
  this: a fired rule inserts facts → propagate → new activations → loop.

## The hardest parts to clone (per-stone hazard map)

1. **Join left/right memory split** (stone: core join). Two memories per beta node, same `join-bindings` key;
   left-activate crosses stored elements, right-activate crosses stored tokens. Combine/miskey them → joins
   silently drop or duplicate. Ref engine.cljc:608.
2. **`NegationWithJoinFilterNode` delta logic** (stone: negation). right-activate must check BOTH "new element
   matches this token?" AND "did a previous element already match?" — retract downstream only on (yes, no).
   Snapshot previous elements BEFORE adding (mutable-list hazard, :888). Ref :819.
3. **Accumulate retract path** (stone: accumulate). Two variants, different strategies: `AccumulateNode`
   caches `[facts reduced]` + `:retract-fn`-or-reaccumulate; `AccumulateWithJoinFilterNode` caches raw
   candidates, filters per-token, never caches the reduce. Ref :1014/:1388/:1278.
4. **Transient/persistent boundary** (stone 0 + fire). All fire-time mutation on the transient; `to-persistent!`
   drains it. A pure-persistent-only clone is correct but loses the O(n) amortized removal. Ref memory.cljc:425/899.
5. **`::not-reduced` sentinel** (stone: accumulate). Absence-of-token ≠ accumulated-nil/zero; four cases in
   `right-activate-reduced` (:1153-1181). Wrong → missed propagations or double-fires.
6. **Logical TM insertion batching + cascade** (stone: fire+TM). `production-memory` holds `{token → [[batch]…]}`;
   a token firing twice → two batches, removed independently; left-retract → remove-insertions! → retract →
   recursive left-retracts. Ref engine.cljc:359-421, memory.cljc:719/726.
