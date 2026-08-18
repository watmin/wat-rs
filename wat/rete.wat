;; wat/rete.wat — arc 278 stone 1a: the rete engine DATA MODEL.
;;
;; Pure data records, the Rule record, the MVP node records + Node defenum sum,
;; the Session record, and a render-dag inspection fn. All on the stone-0
;; persistent collections (:wat::core::PersistentMap / :wat::core::PersistentVector).
;; EDN-round-trippable (it's all EDN data). NO compile, NO fire — just the
;; data model standing as data.
;;
;; Names by the 3rd intueri cast (2026-06-17):
;;   NOT WorkingMemory → Session
;;   production-id (NOT node-id) on Activation
;;
;; Namespace: :wat::rete::
;;
;; Load order: after :wat::core::Record::def (uses the Record macro). Record.wat's
;; macro is order-free (pre-expansion pass) but the eval-time record
;; constructors land at freeze time, so any eval-time dep on this file must
;; appear after it. :wat::core::PersistentMap / PersistentVector are Rust
;; intrinsics — always available.

;; ─── data flow ──────────────────────────────────────────────────────────────

;; Token — provenance/support chain + left-side bindings; flows LEFT through joins.
;; matches: [(fact, alpha-id) …] — the support chain; each entry is a typed tuple
;;   (the pair is heterogeneous: a Record + an i64, which a bare PV cannot honestly type).
;;   Load-bearing for TM: the chain grows one tuple per condition as the token flows through joins.
;; bindings: {?var → value} — variable bindings accumulated left-to-right.
(:wat::core::defrecord :wat::rete::Token
  [matches  <- :wat::core::PersistentVector<(wat::core::Record,wat::core::i64)>
   bindings <- :wat::core::PersistentMap])

;; Element — a fact presented to an alpha node; flows RIGHT into a join.
;; fact: the record fact itself (type-preserving; no conversion needed for provenance/TM/query-by-type).
;; bindings: alpha-bindings extracted by the alpha node's tests.
(:wat::core::defrecord :wat::rete::Element
  [fact     <- :wat::core::Record
   bindings <- :wat::core::PersistentMap])

;; Activation — a ProductionNode queued to fire.
;; production-id: the id of the ProductionNode to fire (intueri: NOT node-id).
;; token: the matching Token that triggered this activation.
(:wat::core::defrecord :wat::rete::Activation
  [production-id <- :wat::core::i64
   token         <- :wat::rete::Token])

;; ─── rules as data ──────────────────────────────────────────────────────────

;; Rule — a rule as pure data (not yet compiled into network nodes).
;; name: the namespaced rule name.
;; lhs:  conditions (form::matches?-shaped clauses) — PersistentVector<WatAST> so foldl works.
;; rhs:  consequence forms (data; pure — applied by a consumer).
(:wat::core::defrecord :wat::rete::Rule
  [name <- :wat::core::String
   lhs  <- :wat::core::PersistentVector<wat::WatAST>
   rhs  <- :wat::core::PersistentVector<wat::WatAST>])

;; Query — a named parametric query (Clara defquery). No :then; answers are
;; binding maps, filtered by param values at `query` time.
(:wat::core::defrecord :wat::rete::Query
  [name   <- :wat::core::String
   params <- :wat::core::PersistentVector<wat::core::String>
   lhs    <- :wat::core::PersistentVector<wat::WatAST>])

;; ─── the network nodes (MVP set) ────────────────────────────────────────────
;; Negation / Test / Accumulate / ExpressionJoin nodes arrive at stones 6–8.

;; AlphaNode — filters facts by structural tests; fans out to beta joins.
;; id:       unique node id (i64).
;; tests:    PersistentVector of test forms (form::matches? clauses) — typed for foldl.
;; children: PersistentVector of child node ids — typed as i64 for foldl.
(:wat::core::defrecord :wat::rete::AlphaNode
  [id       <- :wat::core::i64
   tests    <- :wat::core::PersistentVector<wat::WatAST>
   children <- :wat::core::PersistentVector<wat::core::i64>])

;; RootJoinNode — the leftmost beta join (no left memory needed; seeds the token).
;; id:           unique node id.
;; children:     PersistentVector of child node ids — typed as i64 for foldl.
;; binding-keys: PersistentVector of variable keys (Strings) bound at this join.
(:wat::core::defrecord :wat::rete::RootJoinNode
  [id           <- :wat::core::i64
   children     <- :wat::core::PersistentVector<wat::core::i64>
   binding-keys <- :wat::core::PersistentVector<wat::core::String>])

;; HashJoinNode — a standard two-input beta join node.
;; id:           unique node id.
;; children:     PersistentVector of child node ids — typed as i64 for foldl.
;; binding-keys: PersistentVector of join-key variable names (Strings).
(:wat::core::defrecord :wat::rete::HashJoinNode
  [id           <- :wat::core::i64
   children     <- :wat::core::PersistentVector<wat::core::i64>
   binding-keys <- :wat::core::PersistentVector<wat::core::String>])

;; ProductionNode — the terminal node; triggers an activation on a full token.
;; id:        unique node id.
;; rule-name: the namespaced rule name whose RHS this node fires.
(:wat::core::defrecord :wat::rete::ProductionNode
  [id        <- :wat::core::i64
   rule-name <- :wat::core::String])

;; TestNode — a left-only filter node (stone 6b-ii-a): keeps a token iff eval-test(expr, bindings) is true.
;; id:       unique node id.
;; expr:     the pure∧deterministic WatAST predicate (stored as a value; fence checked at compile).
;; children: PersistentVector of child node ids (ProductionNode or further TestNodes).
(:wat::core::defrecord :wat::rete::TestNode
  [id       <- :wat::core::i64
   expr     <- :wat::WatAST
   children <- :wat::core::PersistentVector<wat::core::i64>])

;; NegationNode — a left-only filter node (stone 7-a): passes a token iff ZERO elements in the
;; negated alpha-memory are compatible with the token's bindings. Hash-join inverted: pure replay
;; dissolves the two-sided delta (the negated alpha-memory is fixed within a fire).
;; id:              unique node id.
;; negated-alpha-id: the AlphaNode id whose alpha-memory holds the facts to check absence against.
;; children:        PersistentVector of child node ids (ProductionNode or further filter nodes).
(:wat::core::defrecord :wat::rete::NegationNode
  [id              <- :wat::core::i64
   negated-alpha-id <- :wat::core::i64
   children        <- :wat::core::PersistentVector<wat::core::i64>])

;; ExistsNode — a left-only filter node (stone 7-exists): the NegationNode sibling with its
;; filter predicate INVERTED. Passes a token iff ≥1 element in the inner alpha-memory is
;; compatible with the token's bindings (negation: ZERO; exists: ≥1). Binds NOTHING, fires the
;; token EXACTLY ONCE regardless of how many match (no multiplicity — the difference from a join).
;; Same shape as NegationNode; additive (NegationNode is untouched). Pure replay dissolves any
;; delta concern (the inner alpha-memory is fixed within a fire), exactly as :not.
;; id:           unique node id.
;; exists-alpha-id: the AlphaNode id whose alpha-memory holds the facts to check presence against.
;; children:     PersistentVector of child node ids (ProductionNode or further filter nodes).
(:wat::core::defrecord :wat::rete::ExistsNode
  [id              <- :wat::core::i64
   exists-alpha-id  <- :wat::core::i64
   children        <- :wat::core::PersistentVector<wat::core::i64>])

;; AccumulateNode — a left-input aggregate join node (stone 8-a): for each parent token,
;; gathers the token-compatible elements from from-alpha-id's alpha-memory, folds them with
;; apply-accumulator (over the 8-i acc::* folds), and extends the token with result-var → aggregate.
;; Pure replay: re-accumulates on every fire (no retract-fn needed).
;; id:            unique node id.
;; result-var:    the ?var name (String, WITHOUT the "?" prefix? — stored as full "?n") bound to the aggregate.
;; acc-form:      the accumulator form (WatAST), e.g. (:wat::rete::acc::count) or (:wat::rete::acc::sum ?v).
;; from-alpha-id: the AlphaNode id whose alpha-memory holds the :from facts.
;; children:      PersistentVector of child node ids (ProductionNode, TestNode, NegationNode, etc.).
(:wat::core::defrecord :wat::rete::AccumulateNode
  [id            <- :wat::core::i64
   result-var    <- :wat::core::String
   acc-form      <- :wat::WatAST
   from-alpha-id <- :wat::core::i64
   children      <- :wat::core::PersistentVector<wat::core::i64>])

;; QueryNode — a named query endpoint; like a production but returns answers.
;; id:         unique node id.
;; query-name: the namespaced query name.
;; param-keys: PersistentVector of query parameter variable names (Strings).
(:wat::core::defrecord :wat::rete::QueryNode
  [id         <- :wat::core::i64
   query-name <- :wat::core::String
   param-keys <- :wat::core::PersistentVector<wat::core::String>])

;; Node — the sum type over all MVP node records (exact defenum syntax per wat/service.wat).
;; Variants wrap their respective record. Used by compile + fire (stones 1b+);
;; the Session.network stores raw node records in v1 (the probe hand-builds with raw records).
(:wat::core::defenum :wat::rete::Node :wat::enum::Pure
  :AlphaNode       [node <- :wat::rete::AlphaNode]
  :RootJoinNode    [node <- :wat::rete::RootJoinNode]
  :HashJoinNode    [node <- :wat::rete::HashJoinNode]
  :ProductionNode  [node <- :wat::rete::ProductionNode]
  :TestNode        [node <- :wat::rete::TestNode]
  :NegationNode    [node <- :wat::rete::NegationNode]
  :ExistsNode      [node <- :wat::rete::ExistsNode]
  :AccumulateNode  [node <- :wat::rete::AccumulateNode]
  :QueryNode       [node <- :wat::rete::QueryNode])

;; ─── the session (the whole engine state) ───────────────────────────────────
;; intueri: NOT WorkingMemory — Session names the whole caller-facing engine state.

;; Session — the complete rete engine state; the caller-facing handle.
;;   network:           id → Node (raw node records) — the compiled DAG, id-indexed.
;;   rules:             PersistentVector of Rule (the rule-set as data).
;;   alpha-memory:      node-id → {join-bindings → [Element …]}
;;                      FIRE-SCOPED — write-only scratch, rebuilt from `facts` on every fire, never
;;                      read from a frozen Session. Population depends on which fire verb produced
;;                      this Session (`fire-once`/`fire-once'` populate it; `fire-rules`/
;;                      `fire-rules-spec` return it empty — arc 278 "alpha is fire-scoped").
;;   beta-memory:       node-id → {join-bindings → [Token …]}
;;                      FIRE-SCOPED — same treatment; `fire-once` populates it, `fire-rules`/
;;                      `fire-rules-spec`/`fire-once'` all return it empty.
;;   production-memory: node-id → PV<:wat::core::Record>  flat derived facts in 4a; grows to the {token → [facts]} support store in 4c (TM)
;;   facts:             PersistentVector of asserted facts.
;;   next-id:           the next free node id (i64).
;;   query-memory:      query-name → PV of binding maps (QueryNode answers; survives fire).
(:wat::core::defrecord :wat::rete::Session
  [network           <- :wat::core::PersistentMap
   rules             <- :wat::core::PersistentVector<wat::rete::Rule>
   alpha-memory      <- :wat::core::PersistentMap
   beta-memory       <- :wat::core::PersistentMap
   production-memory <- :wat::core::PersistentMap
   facts             <- :wat::core::PersistentVector
   next-id           <- :wat::core::i64
   query-memory      <- :wat::core::PersistentMap])

;; ─── P12a: explain substrate ────────────────────────────────────────────────

;; Support — the producing support record for one derived fact.
;;   rule:  the rule name that derived the fact (for Why.rule in P12b).
;;   token: the producing Token; token.matches = the support chain (for :via in P12b).
;; EPHEMERAL — carried only in Explained; never serialized / from-edn.
(:wat::core::defrecord :wat::rete::Support
  [rule  <- :wat::core::String
   token <- :wat::rete::Token])

;; Explained — the opt-in diagnostic result of fire-rules-explain.
;;   session: the same frozen Session the fast path produces (same closure, same derived facts).
;;   support: PersistentMap<derived-fact, Support> — the provenance index.
;; EPHEMERAL — re-derived per explain; never serialized.
(:wat::core::defrecord :wat::rete::Explained
  [session <- :wat::rete::Session
   support <- :wat::core::PersistentMap])

;; ─── P12b+P12c: derivation-tree records + explain walk ─────────────────────

;; DerivationNode — one node in the provenance tree. P12c: adds rule (Option<String>)
;; and changes via to PV<DerivationStep> (the edge payload from P12c).
;;   fact: the derived (or base) fact this node represents.
;;   rule: Some(rule-name) for a derived fact; None for a base/asserted leaf.
;;   via:  the supporting edges — one DerivationStep per supporting fact.
;;         Empty (length 0) ⟺ base/asserted fact (the leaf).
;;         Non-empty ⟺ derived fact (each step explains one supporting input).
;; EPHEMERAL — produced by explain; never serialized.
(:wat::core::defrecord :wat::rete::DerivationNode
  [fact <- :wat::core::Record
   rule <- :wat::core::Option<wat::core::String>
   via  <- :wat::core::PersistentVector<wat::rete::DerivationStep>])

;; DerivationStep — one edge in the provenance tree. Carries the payload that
;; makes the derivation readable without knowing the rule.
;;   supporting:  the supporting fact's own DerivationNode (recurse; leaf = empty via).
;;   pattern:     the matched condition's fact-type FQDN (e.g. "weather::Temperature").
;;   bindings:    per-step bound vars: only the variables this condition bound.
;;   constraints: the rule's satisfied predicates with bound values substituted.
;;                Rendered as WatAST, e.g. (:wat::core::< -5 0) from (:wat::core::< ?c 0) with ?c=-5.
;; EPHEMERAL — produced by explain; never serialized.
(:wat::core::defrecord :wat::rete::DerivationStep
  [supporting  <- :wat::rete::DerivationNode
   pattern     <- :wat::core::String
   bindings    <- :wat::core::PersistentMap<wat::core::String,wat::core::Value>
   constraints <- :wat::core::PersistentVector<wat::WatAST>])

;; step-payload — build a complete DerivationStep for one (sfact, alpha-id) match edge.
;; Calls the Rust step-payload' primitive which reuses resolve_operand + the clause classifier
;; from matcher.rs (faithful by construction: same resolver as what fired).
;; The supporting node (explain's recursion) is passed in; step-payload' builds the full record.
(:wat::core::defn :wat::rete::step-payload
  [session        <- :wat::rete::Session
   alpha-id       <- :wat::core::i64
   bindings       <- :wat::core::PersistentMap
   sfact          <- :wat::core::Record
   supporting     <- :wat::rete::DerivationNode]
  -> :wat::rete::DerivationStep
  (:wat::rete::step-payload' session alpha-id bindings sfact supporting))

;; explain — recursive derivation-tree walk over an Explained support index.
;; For a derived fact (present in Explained/support), returns DerivationNode{fact, rule, via} where
;; via is the list of DerivationStep edges — one per entry in the producing Token's matches chain.
;; For a base fact (absent from the index), returns DerivationNode{fact, rule=None, via=[]} (leaf).
;; Termination: the support DAG is acyclic (fixpoint round structure); base facts are not
;; in the support map → the None branch is the leaf, so recursion always terminates.
(:wat::core::defn :wat::rete::explain
  [ex   <- :wat::rete::Explained
   fact <- :wat::core::Record]
  -> :wat::rete::DerivationNode
  (:wat::core::let [support (:wat::rete::Explained/support ex)
                    sv-opt  (:wat::core::PersistentMap/get support fact)]
    (:wat::core::match sv-opt 
      ((:wat::core::Some sv)
       ;; derived fact — recurse on each supporting fact in the token's matches chain.
       ;; matches is PersistentVector<(wat::core::Record, wat::core::i64)>; each tuple is (sfact, alpha-id).
       (:wat::core::let [tok      (:wat::rete::Support/token sv)
                         matches  (:wat::rete::Token/matches tok)
                         bindings (:wat::rete::Token/bindings tok)
                         rule     (:wat::rete::Support/rule sv)
                         session  (:wat::rete::Explained/session ex)
                         ;; Arc 118.2a — `map` flipped LAZY; `DerivationNode`'s 3rd field is
                         ;; `PersistentVector<DerivationStep>`, so materialize via `into`.
                         via      (:wat::core::into (:wat::core::PersistentVector)
                                    (:wat::core::map
                                      (:wat::core::fn [m <- :(wat::core::Record,wat::core::i64)]
                                        -> :wat::rete::DerivationStep
                                        (:wat::core::let [sfact    (:wat::core::first m)
                                                          alpha-id (:wat::core::second m)]
                                          (:wat::rete::step-payload session alpha-id bindings sfact
                                            (:wat::rete::explain ex sfact))))
                                      matches))]
         (:wat::rete::DerivationNode :fact fact :rule (:wat::core::Some rule) :via via)))
      (:wat::core::None
       ;; base/asserted fact — leaf node, rule=None, via is empty.
       (:wat::rete::DerivationNode :fact fact :rule :wat::core::None :via (:wat::core::PersistentVector))))))

;; ─── render-dag ─────────────────────────────────────────────────────────────

;; node-kind-label — derive a short readable label from a raw node record's
;; declared type FQDN. Returns the last segment (e.g. "RootJoinNode").
;; (:wat::core::type node) returns the class FQDN without leading colon,
;; e.g. "wat::rete::RootJoinNode". We take the text after the last "::".
(:wat::core::defn :wat::rete::node-kind-label
  [node <- :wat::core::Record]
  -> :wat::core::String
  (:wat::core::let [fqdn   (:wat::core::type node)
                    parts  (:wat::core::string::split fqdn "::")
                    n      (:wat::core::length parts)]
    (:wat::core::if (:wat::core::i64::> n 0)
      (:wat::core::Option/expect  
        (:wat::core::get parts (:wat::core::i64::- n 1))
        "node-kind-label: last segment")
      fqdn)))

;; node-children-ids — read the children PersistentVector from a raw node record.
;; Dispatches on kind label: Alpha/RootJoin/HashJoin have children; leaves return empty.
;; WHY: record accessors are class-guarded at runtime; dispatch ensures we only call
;; AlphaNode/children when the node IS an AlphaNode, satisfying the guard.
(:wat::core::defn :wat::rete::node-children-ids
  [node <- :wat::core::Record]
  -> :wat::core::PersistentVector<wat::core::i64>
  (:wat::core::let [kind (:wat::rete::node-kind-label node)]
    (:wat::core::cond
      ((:wat::core::= kind "AlphaNode")
       (:wat::rete::AlphaNode/children node))
      ((:wat::core::= kind "RootJoinNode")
       (:wat::rete::RootJoinNode/children node))
      ((:wat::core::= kind "HashJoinNode")
       (:wat::rete::HashJoinNode/children node))
      ((:wat::core::= kind "TestNode")
       (:wat::rete::TestNode/children node))
      ((:wat::core::= kind "NegationNode")
       (:wat::rete::NegationNode/children node))
      ((:wat::core::= kind "ExistsNode")
       (:wat::rete::ExistsNode/children node))
      ((:wat::core::= kind "AccumulateNode")
       (:wat::rete::AccumulateNode/children node))
      (:else (:wat::core::PersistentVector)))))

;; children-ids-text — format a PersistentVector<i64> as "[id id ...]" for render-dag.
;; WHY: foldl builds space-separated ids so render-dag can emit the edge list inline.
(:wat::core::defn :wat::rete::children-ids-text
  [ids <- :wat::core::PersistentVector<wat::core::i64>]
  -> :wat::core::String
  (:wat::core::let [inner (:wat::core::foldl
                             (:wat::core::fn [acc <- :wat::core::String
                                              id  <- :wat::core::i64]
                               -> :wat::core::String
                               (:wat::core::let [id-s (:wat::core::i64::to-string id)]
                                 (:wat::core::if (:wat::core::= acc "")
                                   id-s
                                   (:wat::core::string::interpolate "{acc} {id-s}" :acc acc :id-s id-s))))
                             ""
                             ids)]
    (:wat::core::string::interpolate "[{inner}]" :inner inner)))

;; render-dag — walk Session.network (id→Node records), emit one readable line
;; per node: "  <id>  <kind> -> [<child-ids>]\n". Returns the whole graph as a String.
;;
;; Strategy: get keys from the PersistentMap as a Vec<i64>, foldl over them,
;; for each key fetch the node (Option/expect), derive the kind label, emit edges.
;; Uses PersistentMap/keys (returns Vec<K>) + foldl + PersistentMap/get.
(:wat::core::defn :wat::rete::render-dag
  [session <- :wat::rete::Session]
  -> :wat::core::String
  (:wat::core::let [network (:wat::rete::Session/network session)
                    keys    (:wat::core::PersistentMap/keys network)]
    (:wat::core::foldl
      (:wat::core::fn [acc <- :wat::core::String
                       k   <- :wat::core::i64]
        -> :wat::core::String
        (:wat::core::let [node  (:wat::core::Option/expect  
                                    (:wat::core::PersistentMap/get network k)
                                    "render-dag: node not found")
                          kind  (:wat::rete::node-kind-label node)
                          id-s  (:wat::core::i64::to-string k)
                          edge  (:wat::rete::children-ids-text
                                   (:wat::rete::node-children-ids node))
                          ;; DELIBERATE proof-by-diff FIXTURE (arc 278): this nested string::concat is
                          ;; below-bar (it should be one `format`), but it is left intentionally — the
                          ;; arc-277 auto-fix is bare-symbol-only and CANNOT reach this COMPOUND/nested
                          ;; case (deferred to RETE). The wat-rete engine's own `compound-concat-collapse`
                          ;; rule will clean it; that diff is the proof the rule works. Do NOT hand-fix.
                          line  (:wat::core::string::concat
                                   "  "
                                   (:wat::core::string::concat
                                     id-s
                                     (:wat::core::string::concat
                                       "  "
                                       (:wat::core::string::concat
                                         kind
                                         (:wat::core::string::concat
                                           " -> "
                                           (:wat::core::string::concat edge "\n"))))))]
          (:wat::core::string::concat acc line)))
      ""
      keys)))

;; ─── compile — rule-set → shared connected network ──────────────────────────

;; CompileState — internal state threaded through compile's rule + condition folds.
;; network: the id→Node PersistentMap built so far.
;; next-id: the next free node id.
;; dedup:   HashMap<String,i64> — maps a structural key to the existing node id;
;;          avoids rescanning the network to detect shareable nodes.
;; WHY a record: cleaner than a Tuple at call sites; fields are domain nouns.
(:wat::core::defrecord :wat::rete::CompileState
  [network <- :wat::core::PersistentMap
   next-id <- :wat::core::i64
   dedup   <- :wat::core::HashMap<wat::core::String,wat::core::i64>])

;; MintResult — result of find-or-mint: the resolved node id + updated state.
;; WHY a record: named fields communicate intent at call sites better than positional.
(:wat::core::defrecord :wat::rete::MintResult
  [id    <- :wat::core::i64
   state <- :wat::rete::CompileState])

;; network-add-child — add child-id to the children of the node at node-id in network.
;; Returns the updated PersistentMap.
;; SET SEMANTICS: children is a set of out-edges. Adding a child-id already present is a
;; no-op — a rete edge means "propagate to this child", and a second identical edge would mean
;; "propagate twice", which no caller wants. When two rules share a compiled node (routine under
;; the find-or-mint-* dedup), this keeps the node's out-degree from growing once per rule.
;; WHY: wiring edges = conj child-id onto the existing children PersistentVector and
;; re-assoc the node; :wat::core::Record/assoc does name-based field update on any Record.
(:wat::core::defn :wat::rete::network-add-child
  [network  <- :wat::core::PersistentMap
   node-id  <- :wat::core::i64
   child-id <- :wat::core::i64]
  -> :wat::core::PersistentMap
  (:wat::core::let [node   (:wat::core::Option/expect
                                  (:wat::core::PersistentMap/get network node-id)
                                  "network-add-child: node not found")
                    old-ch (:wat::rete::node-children-ids node)]
    (:wat::core::if (:wat::core::PersistentVector/contains? old-ch child-id)
      network
      (:wat::core::let [new-ch   (:wat::core::PersistentVector/conj old-ch child-id)
                        new-node (:wat::core::Record/assoc node :children new-ch)]
        (:wat::core::PersistentMap/assoc network node-id new-node)))))

;; find-or-mint-alpha — find an existing AlphaNode whose tests == cond, or mint a new one.
;; Dedup key: "alpha:<write-forms cond>".
;; Returns a MintResult(id, updated-state).
;; WHY write-forms for key: gives a canonical string from the WatAST form; structural
;; equality on the form is span-agnostic so identical conditions always produce the same key.
(:wat::core::defn :wat::rete::find-or-mint-alpha
  [cond  <- :wat::WatAST
   state <- :wat::rete::CompileState]
  -> :wat::rete::MintResult
  (:wat::core::let [cond-text (:wat::core::write-forms cond)
                    dkey      (:wat::core::string::interpolate "alpha:{cond-text}" :cond-text cond-text)
                    network   (:wat::rete::CompileState/network state)
                    next-id   (:wat::rete::CompileState/next-id state)
                    dedup     (:wat::rete::CompileState/dedup   state)
                    found-opt (:wat::core::HashMap/get dedup dkey)]
    (:wat::core::match found-opt 
      ((:wat::core::Some existing-id)
       (:wat::rete::MintResult :id existing-id :state state))
      (:wat::core::None
       (:wat::core::let [alpha     (:wat::rete::AlphaNode
                                      :id next-id
                                      :tests (:wat::core::PersistentVector cond)
                                      :children (:wat::core::PersistentVector))
                         new-net   (:wat::core::PersistentMap/assoc network next-id alpha)
                         new-dedup (:wat::core::HashMap/assoc dedup dkey next-id)
                         new-state (:wat::rete::CompileState
                                      :network new-net
                                      :next-id (:wat::core::i64::+ next-id 1)
                                      :dedup new-dedup)]
         (:wat::rete::MintResult :id next-id :state new-state))))))

;; find-or-mint-root-join — find or mint a RootJoinNode for the first condition.
;; Dedup key: "rootjoin:<cond-text>".
;; WHY split from hash-join: if-branching between different record types (RootJoinNode vs
;; HashJoinNode) cannot be unified by the type checker; two typed fns avoid the mismatch.
(:wat::core::defn :wat::rete::find-or-mint-root-join
  [cond  <- :wat::WatAST
   state <- :wat::rete::CompileState]
  -> :wat::rete::MintResult
  (:wat::core::let [cond-text (:wat::core::write-forms cond)
                    dkey      (:wat::core::string::interpolate "rootjoin:{cond-text}" :cond-text cond-text)
                    network   (:wat::rete::CompileState/network state)
                    next-id   (:wat::rete::CompileState/next-id state)
                    dedup     (:wat::rete::CompileState/dedup   state)
                    found-opt (:wat::core::HashMap/get dedup dkey)]
    (:wat::core::match found-opt 
      ((:wat::core::Some existing-id)
       (:wat::rete::MintResult :id existing-id :state state))
      (:wat::core::None
       (:wat::core::let [join-node (:wat::rete::RootJoinNode
                                      :id next-id
                                      :children (:wat::core::PersistentVector)
                                      :binding-keys (:wat::core::PersistentVector))
                         new-net   (:wat::core::PersistentMap/assoc network next-id join-node)
                         new-dedup (:wat::core::HashMap/assoc dedup dkey next-id)
                         new-state (:wat::rete::CompileState
                                      :network new-net
                                      :next-id (:wat::core::i64::+ next-id 1)
                                      :dedup new-dedup)]
         (:wat::rete::MintResult :id next-id :state new-state))))))

;; find-or-mint-hash-join — find or mint a HashJoinNode for a non-first condition.
;; Dedup key: "hashjoin:<parent-id>:<cond-text>" — both condition AND left parent must match.
(:wat::core::defn :wat::rete::find-or-mint-hash-join
  [cond      <- :wat::WatAST
   parent-id <- :wat::core::i64
   state     <- :wat::rete::CompileState]
  -> :wat::rete::MintResult
  (:wat::core::let [cond-text (:wat::core::write-forms cond)
                    pid-s     (:wat::core::i64::to-string parent-id)
                    dkey      (:wat::core::string::interpolate "hashjoin:{pid-s}:{cond-text}" :pid-s pid-s :cond-text cond-text)
                    network   (:wat::rete::CompileState/network state)
                    next-id   (:wat::rete::CompileState/next-id state)
                    dedup     (:wat::rete::CompileState/dedup   state)
                    found-opt (:wat::core::HashMap/get dedup dkey)]
    (:wat::core::match found-opt 
      ((:wat::core::Some existing-id)
       (:wat::rete::MintResult :id existing-id :state state))
      (:wat::core::None
       (:wat::core::let [join-node (:wat::rete::HashJoinNode
                                      :id next-id
                                      :children (:wat::core::PersistentVector)
                                      :binding-keys (:wat::core::PersistentVector))
                         new-net   (:wat::core::PersistentMap/assoc network next-id join-node)
                         new-dedup (:wat::core::HashMap/assoc dedup dkey next-id)
                         new-state (:wat::rete::CompileState
                                      :network new-net
                                      :next-id (:wat::core::i64::+ next-id 1)
                                      :dedup new-dedup)]
         (:wat::rete::MintResult :id next-id :state new-state))))))

;; compile-condition — fold step: process one condition form in a rule.
;; acc = (CompileState, PV of parent node-ids). Empty PV = no parent yet (first
;; condition → RootJoin). Condition `:or` leaves N arm terminals in the PV;
;; the next condition fans out (one HashJoin / one Test / one :not per parent)
;; and Clara does not require `:or` to be last.
(:wat::core::defrecord :wat::rete::CondFoldAcc
  [state      <- :wat::rete::CompileState
   parent-ids <- :wat::core::PersistentVector<wat::core::i64>])

;; wire-parents — hang `child` off every parent (condition `:or` leaves N terminals).
(:wat::core::defn :wat::rete::wire-parents
  [network <- :wat::core::PersistentMap
   pids    <- :wat::core::PersistentVector<wat::core::i64>
   child   <- :wat::core::i64]
  -> :wat::core::PersistentMap
  (:wat::core::foldl
    (:wat::core::fn [net <- :wat::core::PersistentMap
                     pid <- :wat::core::i64]
      -> :wat::core::PersistentMap
      (:wat::rete::network-add-child net pid child))
    network
    pids))

;; Axis — BRIEF-the-fence-names-the-head, builder-ruled: the CLOSED-SET RULE
;; (REALIZATIONS.md:2676 — "a closed set is an enum, name holds value; open identifiers stay
;; Keyword/String"). WHICH axis `:wat::rete::axis-violation` is being asked about. `:Total` landed
;; BRIEF-total-t1-the-axis-unarmed — and, as designed, minting it broke `axis-violation-message`'s
;; exhaustive match below until its arm was added: the checker enumerated its own consumers instead
;; of anyone remembering by hand. `:Total` is UNARMED — `compile-condition` does not consult
;; `:wat::rete::total?` yet (see `purity.rs`'s module doc, "A third axis, `Total`").
;;
;; ★ NAME COLLISION — read before editing this line: the nature marker `:wat::enum::Pure` right
;; after the type name is UNRELATED to the `:Pure` VARIANT two lines under it. The marker declares
;; THIS ENUM's own runtime representation holds pure data (trivially true — every variant here is a
;; bare unit tag, no payload, so it can never carry a live resource, same as
;; `:wat::runtime::Purity`/`:wat::runtime::Determinism` in runtime-meta.wat). The `:Pure` VARIANT is
;; the axis constant meaning "check effect-freedom." Same word, two unrelated things — do not
;; conflate them when you read or extend this form.
(:wat::core::defenum :wat::rete::Axis :wat::enum::Pure
  :Pure
  :Deterministic
  :Total
  ;; #57 LAW A — the head is a rete primitive. The builder's law: "the entire rete query language
  ;; may only be composed from rete primitives." Its own variant because reusing :Pure would make
  ;; the refusal LIE — `:wat::core::>` IS pure, deterministic and total, and is refused for one
  ;; reason only: it is not from rete. The name is a WORD IN THE SENTENCE `axis-violation-message`
  ;; builds ("is not a rete primitive"), not a label — an earlier spelling, `:Vocabulary`, was cast
  ;; to intueri and failed exactly there: "is not vocabulary" does not parse, and it named the
  ;; table we check rather than the law we hold.
  :RetePrimitive)

;; AxisViolation — BRIEF-the-fence-names-the-head. The result of `:wat::rete::axis-violation`:
;; the offending head when a `where`/accumulator expr falsifies :pure or :deterministic.
;;   head: the violating verb's fqdn (e.g. ":wat::io::IOReader/open-file", ":wat::core::Uuid/v4").
;;   axis: which axis was asked about (:wat::rete::Axis::Pure, ::Deterministic or ::Total) — echoed
;;         back for self-description.
;;   span: the failing call's source Location when the walk was still inside an inspectable AST at
;;         the moment of failure; None for the one case it wasn't (a native fn stub with no body).
;; PROVISIONAL NAME + fields — cast owed (see the brief).
(:wat::core::defrecord :wat::rete::AxisViolation
  [head <- :wat::core::String
   axis <- :wat::rete::Axis
   span <- :wat::core::Option<wat::kernel::Location>])

;; first-failing-axis — given the SAME booleans a fence's `and` already computed, names WHICH axis
;; to explain, mirroring `and`'s left-to-right short-circuit: report the FIRST conjunct that failed,
;; because that is the one the caller must fix first. Never called when all hold — every fence call
;; site reaches this only inside the branch where the accept check already failed.
;;
;; ★ THE CHAIN OF RESTRICTIONS, and the ORDER IS THE MESSAGE (builder-ruled 2026-08-05):
;;
;;     is-pure  →  is-deterministic  →  is-total  →  is-rete
;;
;; Each is measured STRICTLY and separately — *"verbosity is our shield"* (R63/R65: the same
;; exhaustiveness we pay for in keystrokes is what lets us change meaning later). They are NOT
;; collapsed even where one arguably implies another: `is-total` is not folded into `is-rete` on
;; the theory that every rete primitive is total by minting discipline, because
;;   (a) that discipline is a CONVENTION until a gate enforces it — 5 RETE_OPS rows carry
;;       `total: false` today (task #80), so the conjunct can genuinely fire; and
;;   (b) `total?` is a GENERAL language capability, not a rete detail. It shipped callable and
;;       UNARMED with T1, and an unarmed mechanism is R59's dead protocol — a green floor
;;       certifying something that has never once run. The `where` fence is its FIRST REAL
;;       CONSUMER, and proving it here is what earns the right to lean on it elsewhere
;;       ([[300 ALIVS ARGVIT]] — the consumer is the crucible).
;; Builder: *"the is-total will have utility beyond rete — prove it works here so we have our
;; reliable toolkit for further language usage — this is not a one off."*
(:wat::core::defn :wat::rete::first-failing-axis
  [is-pure  <- :wat::core::bool
   is-det   <- :wat::core::bool
   is-total <- :wat::core::bool]
  -> :wat::rete::Axis
  ;; `cond`, not a nested-`if` ladder — the chain reads top-to-bottom in the SAME order the
  ;; conjunction short-circuits, so the code and the law have one shape. (A nested `if` here would
  ;; also trip our own `lint_finds_the_nested_if_ladder`.) Builder: *"use cond over if chaining."*
  (:wat::core::cond
    ((:wat::core::not is-pure) :wat::rete::Axis::Pure)
    ((:wat::core::not is-det)  :wat::rete::Axis::Deterministic)
    (:else                     :wat::rete::Axis::Total)))

;; axis-violation-message — build a human-actionable fence message from an ALREADY-DECIDED
;; rejection. `context` names the fenced site ("where" / "accumulator"); `failing-axis` is the axis
;; `first-failing-axis` picked. This function NEVER changes accept/reject (STOP-3) — both call sites
;; reach it only after `(and is-pure is-det)` was already false — it only names the failure that was
;; already found. `:wat::core::Option/expect`'s message argument is evaluated lazily (only on the
;; None/rejected branch — `expect_panic` in runtime.rs), so this walk never runs on an accepted expr
;; even though it is wired as a plain argument expression.
;;
;; The `match` below is EXHAUSTIVE over `:wat::rete::Axis` — that is the payoff of the enum over a
;; free keyword: BRIEF-total-t1-the-axis-unarmed minted `:Total` and this match went non-exhaustive
;; until the arm below was added — the checker enumerated its own consumer instead of anyone
;; remembering by hand. NOTE: `total?` is UNARMED — `compile-condition` never computes a `:Total`
;; `failing-axis` (that would require consulting `total?` at the fence, which this stone does NOT
;; do), so the `:Total` arm below is currently unreachable from `compile-condition`, but the match
;; must still name it for the function to compile at all — same discipline as the two live arms.
(:wat::core::defn :wat::rete::axis-violation-message
  [context      <- :wat::core::String
   expr         <- :wat::WatAST
   failing-axis <- :wat::rete::Axis]
  -> :wat::core::String
  (:wat::core::match failing-axis
    (:wat::rete::Axis::Pure
     (:wat::core::match (:wat::rete::axis-violation expr :wat::rete::Axis::Pure)
       ((:wat::core::Some v)
        (:wat::core::string::concat "compile-condition: " context " expr is not pure — '"
                                     (:wat::rete::AxisViolation/head v) "' is not pure"))
       (:wat::core::None
        (:wat::core::format "compile-condition: {context} expr is not pure (offending head could not be attributed)" :context context))))
    (:wat::rete::Axis::Deterministic
     (:wat::core::match (:wat::rete::axis-violation expr :wat::rete::Axis::Deterministic)
       ((:wat::core::Some v)
        (:wat::core::string::concat "compile-condition: " context " expr is not deterministic — '"
                                     (:wat::rete::AxisViolation/head v) "' is not deterministic"))
       (:wat::core::None
        (:wat::core::format "compile-condition: {context} expr is not deterministic (offending head could not be attributed)" :context context))))
    (:wat::rete::Axis::Total
     (:wat::core::match (:wat::rete::axis-violation expr :wat::rete::Axis::Total)
       ((:wat::core::Some v)
        (:wat::core::string::concat "compile-condition: " context " expr is not total — '"
                                     (:wat::rete::AxisViolation/head v) "' is not total"))
       (:wat::core::None
        (:wat::core::format "compile-condition: {context} expr is not total (offending head could not be attributed)" :context context))))
    ;; #57 LAW A — the sentence the name was CHOSEN for. The three arms above read "is not pure" /
    ;; "is not deterministic" / "is not total"; this one reads "is not a rete primitive", which IS
    ;; the law ("the entire rete query language may only be composed from rete primitives") and
    ;; tells the author what to do without a lookup. The remedy is named explicitly because a
    ;; refusal that withholds the cure makes the reader hunt (R29 RVINA ERVDIT — the checker
    ;; educates); the rete twin of a core op is its name with `rete::` inserted after `wat::`.
    (:wat::rete::Axis::RetePrimitive
     (:wat::core::match (:wat::rete::axis-violation expr :wat::rete::Axis::RetePrimitive)
       ((:wat::core::Some v)
        (:wat::core::string::concat "compile-condition: " context " expr is not a rete primitive — '"
                                     (:wat::rete::AxisViolation/head v)
                                     "' is not a rete primitive; a where admits only :wat::rete:: ops"))
       (:wat::core::None
        (:wat::core::format "compile-condition: {context} expr is not a rete primitive (offending head could not be attributed)" :context context))))))

(:wat::core::defn :wat::rete::compile-condition
  [acc  <- :wat::rete::CondFoldAcc
   cond <- :wat::WatAST]
  -> :wat::rete::CondFoldAcc
  (:wat::core::let [state0    (:wat::rete::CondFoldAcc/state     acc)
                    parent-ids (:wat::rete::CondFoldAcc/parent-ids acc)
                    ;; TOP: detect (:wat::rete::where <expr>) form
                    ;; Keyword-headed wrappers (`:where`/`:not`/`:exists`/`:or`/`:and`)
                    ;; or a symbol-headed fact-bind / accumulate. Non-empty list.
                    cond-ch   (:wat::core::ast->children cond)
                    head      (:wat::core::first cond-ch)
                    head-nm        (:wat::core::ast-name head)
                    is-where       (:wat::core::= head-nm ":wat::rete::where")
                    is-not         (:wat::core::= head-nm ":wat::rete::not")
                    is-exists      (:wat::core::= head-nm ":wat::rete::exists")
                    is-or          (:wat::core::= head-nm ":wat::rete::or")
                    is-and         (:wat::core::= head-nm ":wat::rete::and")
                    ;; Accumulate: `?` head AND not fact-bind `(?p <- :ns::Type …)`.
                    ;; Fact-bind shares the `?` head; `:from` / a list after `<-` is accumulate.
                    is-accumulate  (:wat::rete::cond-is-accumulate cond)]
    (:wat::core::if is-or
      ;; Each arm is its own left chain from the SAME incoming parents.
      ;; Terminals of every arm become the outgoing parent-ids (Clara :or
      ;; of activations). Nested `:or` recurses through compile-condition.
      (:wat::core::let [or-ch (:wat::core::ast->children cond)
                        arms  (:wat::core::foldl
                                 (:wat::core::fn [acc <- :wat::core::PersistentVector<wat::WatAST>
                                                  i   <- :wat::core::i64]
                                   -> :wat::core::PersistentVector<wat::WatAST>
                                   (:wat::core::PersistentVector/conj acc
                                     (:wat::core::Option/expect
                                       (:wat::core::get or-ch i)
                                       "compile-condition: or arm")))
                                 (:wat::core::PersistentVector)
                                 (:wat::core::range 1 (:wat::core::length or-ch)))
                        _or-n (:wat::core::Option/expect
                                 (:wat::core::if (:wat::core::i64::> (:wat::core::length arms) 0)
                                   (:wat::core::Some nil)
                                   :wat::core::None)
                                 "compile-condition: or of conditions has no arms")
                        incoming parent-ids]
        (:wat::core::foldl
          (:wat::core::fn [fold-acc <- :wat::rete::CondFoldAcc
                           arm      <- :wat::WatAST]
            -> :wat::rete::CondFoldAcc
            (:wat::core::let [arm-acc (:wat::rete::compile-condition
                                        (:wat::rete::CondFoldAcc
                                          :state (:wat::rete::CondFoldAcc/state fold-acc)
                                          :parent-ids incoming)
                                        arm)]
              (:wat::rete::CondFoldAcc
                :state (:wat::rete::CondFoldAcc/state arm-acc)
                :parent-ids (:wat::core::PersistentVector/concat
                              (:wat::rete::CondFoldAcc/parent-ids fold-acc)
                              (:wat::rete::CondFoldAcc/parent-ids arm-acc)))))
          (:wat::rete::CondFoldAcc :state state0 :parent-ids (:wat::core::PersistentVector))
          arms))
    (:wat::core::if is-and
      ;; Sequential group (Clara `:and` inside `:or` / `:not`). Each child
      ;; sees the previous child's terminals — same as listing them in :when.
      (:wat::core::let [and-ch (:wat::core::ast->children cond)
                        kids   (:wat::core::foldl
                                 (:wat::core::fn [acc <- :wat::core::PersistentVector<wat::WatAST>
                                                  i   <- :wat::core::i64]
                                   -> :wat::core::PersistentVector<wat::WatAST>
                                   (:wat::core::PersistentVector/conj acc
                                     (:wat::core::Option/expect
                                       (:wat::core::get and-ch i)
                                       "compile-condition: and child")))
                                 (:wat::core::PersistentVector)
                                 (:wat::core::range 1 (:wat::core::length and-ch)))
                        _and-n (:wat::core::Option/expect
                                 (:wat::core::if (:wat::core::i64::> (:wat::core::length kids) 0)
                                   (:wat::core::Some nil)
                                   :wat::core::None)
                                 "compile-condition: and of conditions has no children")]
        (:wat::core::foldl :wat::rete::compile-condition acc kids))
    (:wat::core::if is-where
      ;; ── where branch (6b-ii-a) ──────────────────────────────────────────────
      (:wat::core::let [expr      (:wat::core::second cond-ch)
                        ;; ★ THE FULL CHAIN OF RESTRICTIONS — pure ∧ deterministic ∧ total ∧ rete.
                        ;; Each measured STRICTLY and separately; see `first-failing-axis` for why
                        ;; none is folded into another. Raise at compile if any fails. The message
                        ;; names the offending head + axis (BRIEF-the-fence-names-the-head); it is
                        ;; computed lazily by `Option/expect` (only on the None/reject branch), so
                        ;; the diagnostic walk never runs on an accepted expr.
                        is-pure   (:wat::rete::pure? expr)
                        is-det    (:wat::rete::deterministic? expr)
                        ;; TOTAL, ARMED — the first real consumer of `total?`, which shipped with
                        ;; T1 callable and unarmed. A partial op inside a `where` is the hazard the
                        ;; whole endeavour exists to remove: there is no jump-table opcode for
                        ;; "raises", so #49a's compiled executor cannot dispatch one.
                        is-total  (:wat::rete::total? expr)
                        ;; #57 LAW A, ARMED. "The entire rete query language may only be composed
                        ;; from rete primitives." A core-spelled op is refused even when it is pure,
                        ;; deterministic AND total — which is exactly why RetePrimitive is its own
                        ;; axis and not a fourth reading of :Pure.
                        is-rete   (:wat::rete::primitive? expr)
                        _fence    (:wat::core::Option/expect
                                      (:wat::core::if (:wat::core::and is-pure is-det is-total is-rete)
                                        (:wat::core::Some nil)
                                        :wat::core::None)
                                      (:wat::rete::axis-violation-message "where" expr
                                        ;; the axis is EXACT, never a default: if the first three
                                        ;; conjuncts held, the only one left to have failed is law A.
                                        (:wat::core::if (:wat::core::and is-pure is-det is-total)
                                          :wat::rete::Axis::RetePrimitive
                                          (:wat::rete::first-failing-axis is-pure is-det is-total))))
                        ;; #49 — lower at rule-compile. A form that cannot lower never
                        ;; enters the network. Fire only executes the circuit.
                        _lowered  (:wat::rete::lower expr)
                        ;; mint the TestNode
                        network0  (:wat::rete::CompileState/network state0)
                        next-id0  (:wat::rete::CompileState/next-id state0)
                        dedup0    (:wat::rete::CompileState/dedup   state0)
                        test-node (:wat::rete::TestNode :id next-id0 :expr expr :children (:wat::core::PersistentVector))
                        net1      (:wat::core::PersistentMap/assoc network0 next-id0 test-node)
                        state1    (:wat::rete::CompileState
                                     :network net1
                                     :next-id (:wat::core::i64::+ next-id0 1)
                                     :dedup dedup0)
                        ;; wire every parent → test (`:or` may leave N terminals)
                        net2      (:wat::rete::wire-parents
                                     (:wat::rete::CompileState/network state1)
                                     parent-ids
                                     next-id0)
                        state2    (:wat::rete::CompileState
                                     :network net2
                                     :next-id (:wat::rete::CompileState/next-id state1)
                                     :dedup (:wat::rete::CompileState/dedup   state1))]
        (:wat::rete::CondFoldAcc :state state2
          :parent-ids (:wat::core::PersistentVector/conj (:wat::core::PersistentVector) next-id0)))
      (:wat::core::if is-not
        ;; ── :not branch (7-a) ───────────────────────────────────────────────────
        ;; Leading :not is legal (Clara negated conjunction matches the empty world).
        ;; Empty parent-ids: filter seeds one empty-binding token.
        (:wat::core::let [;; Extract <inner> — the 2nd child of (:wat::rete::not <inner>)
                          inner       (:wat::core::second cond-ch)
                          ;; find-or-mint an AlphaNode for <inner> (so alpha pass populates it)
                          alpha-res   (:wat::rete::find-or-mint-alpha inner state0)
                          neg-alpha-id (:wat::rete::MintResult/id    alpha-res)
                          state1      (:wat::rete::MintResult/state alpha-res)
                          ;; mint the NegationNode
                          network1    (:wat::rete::CompileState/network state1)
                          next-id1    (:wat::rete::CompileState/next-id state1)
                          dedup1      (:wat::rete::CompileState/dedup   state1)
                          neg-node    (:wat::rete::NegationNode :id next-id1 :negated-alpha-id neg-alpha-id :children (:wat::core::PersistentVector))
                          net2        (:wat::core::PersistentMap/assoc network1 next-id1 neg-node)
                          state2      (:wat::rete::CompileState
                                         :network net2
                                         :next-id (:wat::core::i64::+ next-id1 1)
                                         :dedup dedup1)
                          net3        (:wat::rete::wire-parents
                                          (:wat::rete::CompileState/network state2)
                                          parent-ids
                                          next-id1)
                          state3      (:wat::rete::CompileState
                                         :network net3
                                         :next-id (:wat::rete::CompileState/next-id state2)
                                         :dedup (:wat::rete::CompileState/dedup   state2))]
          (:wat::rete::CondFoldAcc :state state3
            :parent-ids (:wat::core::PersistentVector/conj (:wat::core::PersistentVector) next-id1)))
        (:wat::core::if is-exists
          ;; ── :exists branch (7-exists) ────────────────────────────────────────────
          ;; Leading :exists is legal (Clara test-simple-exists). Empty parent-ids:
          ;; filter emits one token per distinct inner binding, not a dummy seed.
          (:wat::core::let [;; Extract <inner> — the 2nd child of (:wat::rete::exists <inner>)
                            inner        (:wat::core::second cond-ch)
                            ;; find-or-mint an AlphaNode for <inner> (so alpha pass populates it)
                            alpha-res    (:wat::rete::find-or-mint-alpha inner state0)
                            ex-alpha-id  (:wat::rete::MintResult/id    alpha-res)
                            state1       (:wat::rete::MintResult/state alpha-res)
                            ;; mint the ExistsNode
                            network1     (:wat::rete::CompileState/network state1)
                            next-id1     (:wat::rete::CompileState/next-id state1)
                            dedup1       (:wat::rete::CompileState/dedup   state1)
                            ex-node      (:wat::rete::ExistsNode :id next-id1 :exists-alpha-id ex-alpha-id :children (:wat::core::PersistentVector))
                            net2         (:wat::core::PersistentMap/assoc network1 next-id1 ex-node)
                            state2       (:wat::rete::CompileState
                                           :network net2
                                           :next-id (:wat::core::i64::+ next-id1 1)
                                           :dedup dedup1)
                            net3         (:wat::rete::wire-parents
                                            (:wat::rete::CompileState/network state2)
                                            parent-ids
                                            next-id1)
                            state3       (:wat::rete::CompileState
                                           :network net3
                                           :next-id (:wat::rete::CompileState/next-id state2)
                                           :dedup (:wat::rete::CompileState/dedup   state2))]
            (:wat::rete::CondFoldAcc :state state3
              :parent-ids (:wat::core::PersistentVector/conj (:wat::core::PersistentVector) next-id1)))
        (:wat::core::if is-accumulate
          ;; ── accumulate branch (8-a) ─────────────────────────────────────────────
          ;; Form: (?result-var <- (<acc-form>) :from (<inner>))
          ;; children: [?result-var, <-, acc-form, :from, inner]
          ;; Leading accumulate is legal (Clara test-count: empty world → count 0).
          ;; Empty parent-ids: accumulate-pass seeds one empty-binding token.
          (:wat::core::let [;; result-var: strip the "?" prefix from head-nm to get the var name string
                            result-var   head-nm
                            ;; acc-form: items[2]
                            acc-form     (:wat::core::Option/expect  
                                             (:wat::core::get cond-ch 2)
                                             "compile-condition: accumulate missing acc-form")
                            ;; 8-custom FENCE: the acc-form head selects the fold. A built-in
                            ;; head (:wat::rete::acc::*) is trusted (skip the fence). Any other
                            ;; head is a USER fold fn → assert it is pure∧det (the same 6a fence
                            ;; `where` uses), else raise at compile. Build a synthetic call
                            ;; `(<acc-hd> __acc__)` and run pure?/deterministic? on it — head_ok
                            ;; classifies the user fn transitively (purity.rs:classify_fn).
                            acc-ch       (:wat::core::ast->children acc-form)
                            acc-hd       (:wat::core::first acc-ch)
                            acc-hd-nm    (:wat::core::ast-name acc-hd)
                            is-builtin   (:wat::core::string::starts-with? acc-hd-nm ":wat::rete::acc::")
                            fence-call   (:wat::core::quasiquote
                                            ((:wat::core::unquote acc-hd) __acc__))
                            ;; fence: pure ∧ deterministic (skipped for :wat::rete::acc::* builtins,
                            ;; which are trusted). is-pure/is-det are computed unconditionally — same
                            ;; shape as the `where` fence above — which is safe even for a builtin
                            ;; acc-hd: pure?/deterministic? default-deny gracefully on an unrecognized
                            ;; head, never panic. The message names the offending head + axis
                            ;; (BRIEF-the-fence-names-the-head, "accumulator" not "where"); it is
                            ;; computed lazily by `Option/expect` (only on the None/reject branch).
                            is-pure      (:wat::rete::pure? fence-call)
                            is-det       (:wat::rete::deterministic? fence-call)
                            ;; TOTAL, ARMED — the accumulator fence is the `where` fence's sibling
                            ;; (the stone scopes the vocabulary to "a `where` (and the accumulator
                            ;; fence)"), so it measures the same chain.
                            is-total     (:wat::rete::total? fence-call)
                            ;; #83 LAW A, ARMED HERE TOO — the fourth conjunct, closing the gap that
                            ;; left this fence at three where `where` and `:then` had four. The prior
                            ;; revision of this comment said "`is-rete` is NOT added here … widening it
                            ;; to this surface is its own strike"; that was a deferral written AGAINST
                            ;; the stone, which scopes the vocabulary to "a `where` (AND THE ACCUMULATOR
                            ;; FENCE)". This IS that strike.
                            ;;
                            ;; Note the `is-builtin` short-circuit below exempts a `:wat::rete::acc::*`
                            ;; head WHOLESALE (all four conjuncts), so the population law A newly reaches
                            ;; here is exactly the USER fold fn — which stays admissible transitively
                            ;; whenever its body bottoms out in rete primitives (the composition door,
                            ;; purity.rs:classify_fn). What it now refuses is a core-spelled fold.
                            is-rete      (:wat::rete::primitive? fence-call)
                            _acc-fence   (:wat::core::Option/expect
                                             (:wat::core::if is-builtin
                                               (:wat::core::Some nil)
                                               (:wat::core::if (:wat::core::and is-pure is-det is-total is-rete)
                                                 (:wat::core::Some nil)
                                                 :wat::core::None))
                                             (:wat::rete::axis-violation-message "accumulator" fence-call
                                               ;; the axis is EXACT, never a default — the same rule the
                                               ;; `where` fence uses: if the first three conjuncts held,
                                               ;; the only one left to have failed is law A.
                                               (:wat::core::if (:wat::core::and is-pure is-det is-total)
                                                 :wat::rete::Axis::RetePrimitive
                                                 (:wat::rete::first-failing-axis is-pure is-det is-total))))
                            ;; assert items[3] is :from (structural validation)
                            from-kw      (:wat::core::Option/expect  
                                             (:wat::core::get cond-ch 3)
                                             "compile-condition: accumulate missing :from")
                            _from-check  (:wat::core::Option/expect  
                                             (:wat::core::if (:wat::core::= (:wat::core::ast-name from-kw) ":from")
                                               (:wat::core::Some nil)
                                               :wat::core::None)
                                             "compile-condition: accumulate expected :from at position 3")
                            ;; inner: items[4] — the :from fact-pattern condition
                            inner        (:wat::core::Option/expect  
                                             (:wat::core::get cond-ch 4)
                                             "compile-condition: accumulate missing :from inner condition")
                            ;; find-or-mint an AlphaNode for the :from inner condition
                            alpha-res    (:wat::rete::find-or-mint-alpha inner state0)
                            from-alpha-id (:wat::rete::MintResult/id    alpha-res)
                            state1       (:wat::rete::MintResult/state alpha-res)
                            ;; mint the AccumulateNode
                            network1     (:wat::rete::CompileState/network state1)
                            next-id1     (:wat::rete::CompileState/next-id state1)
                            dedup1       (:wat::rete::CompileState/dedup   state1)
                            acc-node     (:wat::rete::AccumulateNode
                                             :id next-id1
                                             :result-var result-var
                                             :acc-form acc-form
                                             :from-alpha-id from-alpha-id
                                             :children (:wat::core::PersistentVector))
                            net2         (:wat::core::PersistentMap/assoc network1 next-id1 acc-node)
                            state2       (:wat::rete::CompileState
                                            :network net2
                                            :next-id (:wat::core::i64::+ next-id1 1)
                                            :dedup dedup1)
                            net3         (:wat::rete::wire-parents
                                             (:wat::rete::CompileState/network state2)
                                             parent-ids
                                             next-id1)
                            state3       (:wat::rete::CompileState
                                            :network net3
                                            :next-id (:wat::rete::CompileState/next-id state2)
                                            :dedup (:wat::rete::CompileState/dedup   state2))]
            (:wat::rete::CondFoldAcc :state state3
              :parent-ids (:wat::core::PersistentVector/conj (:wat::core::PersistentVector) next-id1)))
          ;; ── alpha+join: first condition → RootJoin; later → one HashJoin per parent
          (:wat::core::let [alpha-res  (:wat::rete::find-or-mint-alpha cond state0)
                        alpha-id   (:wat::rete::MintResult/id    alpha-res)
                        state1     (:wat::rete::MintResult/state alpha-res)
                        is-first   (:wat::core::= (:wat::core::length parent-ids) 0)]
            (:wat::core::if is-first
              (:wat::core::let [join-res (:wat::rete::find-or-mint-root-join cond state1)
                                join-id  (:wat::rete::MintResult/id    join-res)
                                state2   (:wat::rete::MintResult/state join-res)
                                net3     (:wat::rete::network-add-child
                                           (:wat::rete::CompileState/network state2)
                                           alpha-id
                                           join-id)
                                state3   (:wat::rete::CompileState
                                           :network net3
                                           :next-id (:wat::rete::CompileState/next-id state2)
                                           :dedup (:wat::rete::CompileState/dedup state2))]
                (:wat::rete::CondFoldAcc :state state3
                  :parent-ids (:wat::core::PersistentVector/conj (:wat::core::PersistentVector) join-id)))
              (:wat::core::let [fan (:wat::core::foldl
                                      (:wat::core::fn [acc <- :wat::rete::CondFoldAcc
                                                       pid <- :wat::core::i64]
                                        -> :wat::rete::CondFoldAcc
                                        (:wat::core::let [st0 (:wat::rete::CondFoldAcc/state acc)
                                                          jr  (:wat::rete::find-or-mint-hash-join cond pid st0)
                                                          jid (:wat::rete::MintResult/id jr)
                                                          st1 (:wat::rete::MintResult/state jr)
                                                          n1  (:wat::rete::network-add-child
                                                                (:wat::rete::CompileState/network st1)
                                                                alpha-id
                                                                jid)
                                                          n2  (:wat::rete::network-add-child n1 pid jid)
                                                          st2 (:wat::rete::CompileState
                                                                :network n2
                                                                :next-id (:wat::rete::CompileState/next-id st1)
                                                                :dedup (:wat::rete::CompileState/dedup st1))]
                                          (:wat::rete::CondFoldAcc :state st2
                                            :parent-ids (:wat::core::PersistentVector/conj
                                                          (:wat::rete::CondFoldAcc/parent-ids acc)
                                                          jid))))
                                      (:wat::rete::CondFoldAcc :state state1
                                        :parent-ids (:wat::core::PersistentVector))
                                      parent-ids)]
                fan)))))))))))

;; then-item-fence — Stone B (arc 278 DESIGN-STONE-then-is-a-vector-of-singular-facts.md §
;; "Stone B"): the RHS fence, mirroring `compile-condition`'s `where` fence (above,
;; :wat::rete::compile-condition's `is-where` branch) and the accumulator fence's synthetic-call
;; trick (the `is-accumulate` branch) — except a `:then` item is ALREADY a call form
;; `(<head> arg…)`, so no synthesis is needed: `pure?`/`deterministic?` on the item itself walks
;; the head (constructor_meta / sym.functions, `purity.rs::head_ok`) AND recurses into every
;; operand. ONE check covers BOTH widenings named in `BRIEF-then-user-forms.md`: (a) the item
;; head may be a fn (not only a fact-type constructor — `head_ok` dispatches on either via the
;; SAME declaration-derived / sym.functions doors already used for `where`) and (b) an operand
;; may be a composed expression, not only `?var`/`:field`/literal (classify_expr's generic
;; list-call arm already recurses into every argument on the same axis).
;;
;; SECOND check, new to `:then` (not shared with `where`, which never claims to produce
;; anything): the item must RETURN A FACT. Evaluate the head to its fn value — `eval-ast!`, since
;; `item-ch`'s head is already a `:wat::WatAST` value — then read its declared return type
;; (`return-type-of`) and confirm that type is a registered record/struct
;; (`field-names-of` raises otherwise — the SAME registry `validate_and_reorder_then`'s
;; `lookup_fields` reads on the Rust side, reached here in wat because that Rust validator
;; carries `types` but not `sym`, per `BRIEF-then-user-forms.md`'s "the fence goes where the
;; where fence already is").
;;
;; A bare record-type keyword (e.g. `:usr::Rate`) evaluates to a KEYWORD, not its constructor fn
;; (arc 294 item 9a's construction flip — `runtime.rs::eval_return_type_of`'s own doc) — resolve
;; through the PRIME `:T'` in that case, the exact `:wat::rete::query` macro idiom
;; ("types-as-forms"). A plain `:wat::core::defn` has no such indirection and already resolved to
;; a fn on the bare read.
;;
;; foldl-compatible: `(acc, item) -> acc`, so `compile-rule` folds this straight over `rhs`
;; without a lambda wrapper — the accumulator is a throwaway `i64`, unused except to satisfy
;; foldl's shape; every check here is a side-effecting raise (an axis violation panics via
;; `Option/expect`, exactly like `where`'s fence; "does not return a fact" raises normally,
;; via `field-names-of`'s own diagnostic — both are freeze-time-only, never per derived fact).
(:wat::core::defn :wat::rete::then-item-fence
  [acc  <- :wat::core::i64
   item <- :wat::WatAST]
  -> :wat::core::i64
  (:wat::core::let [is-pure   (:wat::rete::pure? item)
                    is-det    (:wat::rete::deterministic? item)
                    ;; TOTAL, ARMED. A `:then` item that can raise aborts the fire mid-derivation,
                    ;; which is the same hazard by a different door.
                    is-total  (:wat::rete::total? item)
                    ;; ★ LAW A, ARMED ON THE RHS TOO (2026-08-05, the builder's call after the
                    ;; measurement below). The `where` fence got `is-rete` first and this one was
                    ;; deliberately left at three conjuncts — which MEASURED as a real gap:
                    ;;
                    ;;     :then item using (:wat::core::i64::+ …)                      -> refused (not total)
                    ;;     :then item using (:wat::core::if (:wat::core::i64::> n 5) …) -> COMPILED
                    ;;
                    ;; A core-spelled TOTAL op sailed straight through. The ruling is general —
                    ;; *"rete forms must only be composed of rete forms and primitives"* — and the
                    ;; RHS is not exempt: `compiled_rhs.rs` ALREADY EXISTS, so the RHS is already a
                    ;; compiled surface and wants the same closed head-space `where` does. There is
                    ;; no opcode for a head the vocabulary does not list.
                    ;;
                    ;; This does NOT narrow what a `:then` may CONSTRUCT: a user record/fact
                    ;; constructor is admitted by the declaration-derived constructor door, which
                    ;; law A never consults a namespace for (`head_ok`'s first door). What it
                    ;; refuses is core-spelled COMPUTATION inside the item.
                    is-rete   (:wat::rete::primitive? item)
                    _fence    (:wat::core::Option/expect
                                  (:wat::core::if (:wat::core::and is-pure is-det is-total is-rete)
                                    (:wat::core::Some nil)
                                    :wat::core::None)
                                  (:wat::rete::axis-violation-message "then" item
                                    ;; exact, never a default: if the first three held, law A is
                                    ;; the only conjunct left to have failed.
                                    (:wat::core::if (:wat::core::and is-pure is-det is-total)
                                      :wat::rete::Axis::RetePrimitive
                                      (:wat::rete::first-failing-axis is-pure is-det is-total))))
                    item-ch   (:wat::core::ast->children item)
                    head      (:wat::core::first item-ch)
                    head-val0 (:wat::core::Result/expect
                                  (:wat::eval-ast! head)
                                  "compile-rule: :then item head failed to evaluate")
                    ;; `:wat::core::type` returns the COLON-FREE FQDN (`Value::declared_type_name`)
                    ;; — compare against "wat::core::fn", not ":wat::core::fn".
                    is-fn-val (:wat::core::= (:wat::core::type head-val0) "wat::core::fn")
                    ;; A bare record-type keyword evaluates to a KEYWORD — re-resolve through its
                    ;; PRIME `:T'` to reach the constructor fn (see this defn's doc). A plain
                    ;; `defn` already resolved to a fn above and takes the `is-fn-val` branch.
                    prime-kw  (:wat::core::keyword-node
                                  (:wat::core::string::concat (:wat::core::ast-name head) "'"))
                    head-fn   (:wat::core::if is-fn-val
                                  head-val0
                                  (:wat::core::Result/expect
                                     (:wat::eval-ast! prime-kw)
                                     "compile-rule: :then item head failed to resolve to a fn"))
                    ;; return-type-of raises "unknown type" itself if head-fn is STILL a bare
                    ;; keyword (a genuinely unrecognised head) — no separate check needed here.
                    ret-ty    (:wat::runtime::return-type-of head-fn)
                    ;; `keyword/from-string` wants COLON-FREE input (it adds the sigil itself);
                    ;; `return-type-of` already returns a colon-free FQDN — do not re-prepend one.
                    ret-kw    (:wat::core::keyword/from-string ret-ty)
                    ;; raises unless ret-kw names a registered record/struct — "produces a fact."
                    _fact-ty  (:wat::runtime::field-names-of ret-kw)]
    acc))

;; ast-qvars — every `?var` symbol under a condition AST (binds and uses).
(:wat::core::defn :wat::rete::ast-qvars
  [ast <- :wat::WatAST]
  -> :wat::core::PersistentVector<wat::core::String>
  (:wat::core::let [k (:wat::core::ast-kind ast)]
    (:wat::core::if (:wat::core::= k "symbol")
      (:wat::core::let [nm (:wat::core::ast-name ast)]
        (:wat::core::if (:wat::core::string::starts-with? nm "?")
          (:wat::core::PersistentVector/conj (:wat::core::PersistentVector) nm)
          (:wat::core::PersistentVector)))
      (:wat::core::if
        (:wat::core::if (:wat::core::= k "list")
          true
          (:wat::core::= k "vector"))
        (:wat::core::let [ch (:wat::core::ast->children ast)
                          n  (:wat::core::length ch)]
          (:wat::core::foldl
            (:wat::core::fn [acc <- :wat::core::PersistentVector<wat::core::String>
                             i   <- :wat::core::i64]
              -> :wat::core::PersistentVector<wat::core::String>
              (:wat::core::let [kid (:wat::core::Option/expect
                                      (:wat::core::get ch i)
                                      "ast-qvars")]
                (:wat::core::foldl
                  (:wat::core::fn [out <- :wat::core::PersistentVector<wat::core::String>
                                   nm  <- :wat::core::String]
                    -> :wat::core::PersistentVector<wat::core::String>
                    (:wat::core::if (:wat::core::PersistentVector/contains? out nm)
                      out
                      (:wat::core::PersistentVector/conj out nm)))
                  acc
                  (:wat::rete::ast-qvars kid))))
            (:wat::core::PersistentVector)
            (:wat::core::range 0 n)))
        (:wat::core::PersistentVector)))))

;; cond-bind-keys — `?var` names this condition BINDS (`(?v <- :field)`,
;; fact-bind `(?p <- :ns::Type …)`, accum result, `:from` inner, `:exists`
;; inner). `:not` / `:where` bind nothing.
(:wat::core::defn :wat::rete::cond-bind-keys
  [cond <- :wat::WatAST]
  -> :wat::core::PersistentVector<wat::core::String>
  (:wat::core::if (:wat::core::not (:wat::core::= (:wat::core::ast-kind cond) "list"))
    (:wat::core::PersistentVector)
    (:wat::core::let [ch (:wat::core::ast->children cond)
                      n  (:wat::core::length ch)]
      (:wat::core::if (:wat::core::= n 0)
        (:wat::core::PersistentVector)
        (:wat::core::let [head   (:wat::core::first ch)
                          head-k (:wat::core::ast-kind head)]
          (:wat::core::if (:wat::core::= head-k "symbol")
            (:wat::core::let [hnm (:wat::core::ast-name head)]
              (:wat::core::if (:wat::rete::cond-is-fact-bind cond)
                (:wat::core::foldl
                  (:wat::core::fn [acc <- :wat::core::PersistentVector<wat::core::String>
                                   i   <- :wat::core::i64]
                    -> :wat::core::PersistentVector<wat::core::String>
                    (:wat::core::let [kid (:wat::core::Option/expect
                                            (:wat::core::get ch i)
                                            "cond-bind-keys: fact-bind clause")]
                      (:wat::core::foldl
                        (:wat::core::fn [out <- :wat::core::PersistentVector<wat::core::String>
                                         nm  <- :wat::core::String]
                          -> :wat::core::PersistentVector<wat::core::String>
                          (:wat::core::if (:wat::core::PersistentVector/contains? out nm)
                            out
                            (:wat::core::PersistentVector/conj out nm)))
                        acc
                        (:wat::rete::cond-bind-keys kid))))
                  (:wat::core::PersistentVector/conj (:wat::core::PersistentVector) hnm)
                  (:wat::core::range 3 n))
                (:wat::core::if
                  (:wat::core::if (:wat::core::string::starts-with? hnm "?")
                    (:wat::core::if (:wat::core::= n 3)
                      (:wat::core::if (:wat::core::= (:wat::core::ast-kind
                                                      (:wat::core::Option/expect
                                                        (:wat::core::get ch 1)
                                                        "cond-bind-keys: bind arrow"))
                                                    "symbol")
                        (:wat::core::= (:wat::core::ast-name
                                        (:wat::core::Option/expect
                                          (:wat::core::get ch 1)
                                          "cond-bind-keys: bind arrow"))
                                      "<-")
                        false)
                      false)
                    false)
                  (:wat::core::PersistentVector/conj (:wat::core::PersistentVector) hnm)
                  (:wat::core::if
                    (:wat::core::if (:wat::core::string::starts-with? hnm "?")
                      (:wat::core::if (:wat::core::= n 5)
                        (:wat::core::= (:wat::core::ast-name
                                        (:wat::core::Option/expect
                                          (:wat::core::get ch 3)
                                          "cond-bind-keys: :from"))
                                      ":from")
                        false)
                      false)
                    (:wat::core::foldl
                      (:wat::core::fn [acc <- :wat::core::PersistentVector<wat::core::String>
                                       nm  <- :wat::core::String]
                        -> :wat::core::PersistentVector<wat::core::String>
                        (:wat::core::if (:wat::core::PersistentVector/contains? acc nm)
                          acc
                          (:wat::core::PersistentVector/conj acc nm)))
                      (:wat::core::PersistentVector/conj (:wat::core::PersistentVector) hnm)
                      (:wat::rete::cond-bind-keys
                        (:wat::core::Option/expect
                          (:wat::core::get ch 4)
                          "cond-bind-keys: :from inner")))
                    (:wat::core::PersistentVector)))))
            (:wat::core::if (:wat::core::= head-k "keyword")
              (:wat::core::let [hnm (:wat::core::ast-name head)]
                (:wat::core::cond
                  ((:wat::core::= hnm ":wat::rete::not")
                   (:wat::core::PersistentVector))
                  ((:wat::core::= hnm ":wat::rete::where")
                   (:wat::core::PersistentVector))
                  ((:wat::core::= hnm ":wat::rete::exists")
                   (:wat::rete::cond-bind-keys
                     (:wat::core::second ch)))
                  (:else
                   (:wat::core::foldl
                     (:wat::core::fn [acc <- :wat::core::PersistentVector<wat::core::String>
                                      i   <- :wat::core::i64]
                       -> :wat::core::PersistentVector<wat::core::String>
                       (:wat::core::let [kid (:wat::core::Option/expect
                                               (:wat::core::get ch i)
                                               "cond-bind-keys: child")]
                         (:wat::core::foldl
                           (:wat::core::fn [out <- :wat::core::PersistentVector<wat::core::String>
                                            nm  <- :wat::core::String]
                             -> :wat::core::PersistentVector<wat::core::String>
                             (:wat::core::if (:wat::core::PersistentVector/contains? out nm)
                               out
                               (:wat::core::PersistentVector/conj out nm)))
                           acc
                           (:wat::rete::cond-bind-keys kid))))
                     (:wat::core::PersistentVector)
                     (:wat::core::range 1 n)))))
              (:wat::core::PersistentVector))))))))

;; cond-is-fact-bind — `(?p <- :ns::Type …)` (Clara `[?p <- Type]`). Type keyword has `::`.
(:wat::core::defn :wat::rete::cond-is-fact-bind
  [cond <- :wat::WatAST]
  -> :wat::core::bool
  (:wat::core::if (:wat::core::not (:wat::core::= (:wat::core::ast-kind cond) "list"))
    false
    (:wat::core::let [ch (:wat::core::ast->children cond)]
      (:wat::core::if (:wat::core::< (:wat::core::length ch) 3)
        false
        (:wat::core::if
          (:wat::core::if (:wat::core::= (:wat::core::ast-kind (:wat::core::first ch)) "symbol")
            (:wat::core::string::starts-with? (:wat::core::ast-name (:wat::core::first ch)) "?")
            false)
          (:wat::core::if
            (:wat::core::if (:wat::core::= (:wat::core::ast-kind
                                            (:wat::core::Option/expect
                                              (:wat::core::get ch 1)
                                              "cond-is-fact-bind: arrow"))
                                          "symbol")
              (:wat::core::= (:wat::core::ast-name
                               (:wat::core::Option/expect
                                 (:wat::core::get ch 1)
                                 "cond-is-fact-bind: arrow"))
                             "<-")
              false)
            (:wat::core::if (:wat::core::= (:wat::core::ast-kind
                                            (:wat::core::Option/expect
                                              (:wat::core::get ch 2)
                                              "cond-is-fact-bind: type"))
                                          "keyword")
              (:wat::core::string::contains?
                (:wat::core::ast-name
                  (:wat::core::Option/expect
                    (:wat::core::get ch 2)
                    "cond-is-fact-bind: type"))
                "::")
              false)
            false)
          false)))))

;; cond-is-accumulate — `?result` head that is NOT a fact-bind.
(:wat::core::defn :wat::rete::cond-is-accumulate
  [cond <- :wat::WatAST]
  -> :wat::core::bool
  (:wat::core::if (:wat::rete::cond-is-fact-bind cond)
    false
    (:wat::core::if (:wat::core::not (:wat::core::= (:wat::core::ast-kind cond) "list"))
      false
      (:wat::core::let [ch (:wat::core::ast->children cond)]
        (:wat::core::if (:wat::core::= (:wat::core::length ch) 0)
          false
          (:wat::core::if (:wat::core::= (:wat::core::ast-kind (:wat::core::first ch)) "symbol")
            (:wat::core::string::starts-with? (:wat::core::ast-name (:wat::core::first ch)) "?")
            false))))))

;; sort-lhs — Clara defers accumulators so a later fact can bind the group
;; key (test-count-none-joined: Wind first, then count Temps at ?loc → 0).
;; Non-accums that mention an accum result-var stay after the accum (`:where`
;; on ?c). Relative order inside each partition is preserved.
(:wat::core::defn :wat::rete::sort-lhs
  [lhs <- :wat::core::PersistentVector<wat::WatAST>]
  -> :wat::core::PersistentVector<wat::WatAST>
  (:wat::core::let [result-vars
                    (:wat::core::foldl
                      (:wat::core::fn [acc  <- :wat::core::PersistentVector<wat::core::String>
                                       cond <- :wat::WatAST]
                        -> :wat::core::PersistentVector<wat::core::String>
                        (:wat::core::if (:wat::rete::cond-is-accumulate cond)
                          (:wat::core::let [ch (:wat::core::ast->children cond)]
                            (:wat::core::PersistentVector/conj
                              acc
                              (:wat::core::ast-name (:wat::core::first ch))))
                          acc))
                      (:wat::core::PersistentVector)
                      lhs)
                    uses-result?
                    (:wat::core::fn [cond <- :wat::WatAST] -> :wat::core::bool
                      (:wat::core::let [qs (:wat::rete::ast-qvars cond)]
                        (:wat::core::foldl
                          (:wat::core::fn [hit <- :wat::core::bool
                                           rv  <- :wat::core::String]
                            -> :wat::core::bool
                            (:wat::core::if hit
                              true
                              (:wat::core::PersistentVector/contains? qs rv)))
                          false
                          result-vars)))
                    independent
                    (:wat::core::foldl
                      (:wat::core::fn [acc  <- :wat::core::PersistentVector<wat::WatAST>
                                       cond <- :wat::WatAST]
                        -> :wat::core::PersistentVector<wat::WatAST>
                        (:wat::core::if
                          (:wat::core::if (:wat::rete::cond-is-accumulate cond)
                            false
                            (:wat::core::not (uses-result? cond)))
                          (:wat::core::PersistentVector/conj acc cond)
                          acc))
                      (:wat::core::PersistentVector)
                      lhs)
                    accums
                    (:wat::core::foldl
                      (:wat::core::fn [acc  <- :wat::core::PersistentVector<wat::WatAST>
                                       cond <- :wat::WatAST]
                        -> :wat::core::PersistentVector<wat::WatAST>
                        (:wat::core::if (:wat::rete::cond-is-accumulate cond)
                          (:wat::core::PersistentVector/conj acc cond)
                          acc))
                      (:wat::core::PersistentVector)
                      lhs)
                    rest
                    (:wat::core::foldl
                      (:wat::core::fn [acc  <- :wat::core::PersistentVector<wat::WatAST>
                                       cond <- :wat::WatAST]
                        -> :wat::core::PersistentVector<wat::WatAST>
                        (:wat::core::if
                          (:wat::core::if (:wat::rete::cond-is-accumulate cond)
                            false
                            (uses-result? cond))
                          (:wat::core::PersistentVector/conj acc cond)
                          acc))
                      (:wat::core::PersistentVector)
                      lhs)]
    (:wat::core::PersistentVector/concat
      independent
      (:wat::core::PersistentVector/concat accums rest))))

;; compile-rule — fold step: process one Rule into the network.
;; WHY: folds over the rule's lhs conditions with compile-condition, then mints
;; the ProductionNode as a child of every remaining parent (one join after a
;; linear :when; N arm terminals after a condition `:or`).
;;
;; Arc 278 Stone B — fences `rhs` (the rule's `:then` items) via `then-item-fence` BEFORE folding
;; `lhs`, so a malformed RHS is caught before any network nodes are minted for this rule. Mirrors
;; how `where`/accumulate are fenced inline during the LHS fold — this is the RHS's own
;; freeze-time-only pass, over the SAME `rule` this fn already receives.
(:wat::core::defn :wat::rete::compile-rule
  [state <- :wat::rete::CompileState
   rule  <- :wat::rete::Rule]
  -> :wat::rete::CompileState
  (:wat::core::let [lhs        (:wat::rete::Rule/lhs rule)
                    rhs        (:wat::rete::Rule/rhs rule)
                    rname      (:wat::rete::Rule/name rule)
                    _rhs-fence (:wat::core::foldl :wat::rete::then-item-fence 0 rhs)
                    lhs-sorted (:wat::rete::sort-lhs lhs)
                    init-acc   (:wat::rete::CondFoldAcc
                                 :state state
                                 :parent-ids (:wat::core::PersistentVector))
                    final-acc  (:wat::core::foldl :wat::rete::compile-condition init-acc lhs-sorted)
                    state2     (:wat::rete::CondFoldAcc/state      final-acc)
                    pids       (:wat::rete::CondFoldAcc/parent-ids final-acc)
                    network2   (:wat::rete::CompileState/network state2)
                    next-id2   (:wat::rete::CompileState/next-id state2)
                    prod       (:wat::rete::ProductionNode :id next-id2 :rule-name rname)
                    net3       (:wat::core::PersistentMap/assoc network2 next-id2 prod)
                    net4       (:wat::rete::wire-parents net3 pids next-id2)]
    (:wat::rete::CompileState :network net4 :next-id (:wat::core::i64::+ next-id2 1)
      :dedup (:wat::rete::CompileState/dedup state2))))

;; compile-query — same LHS fold as compile-rule; terminal is a QueryNode.
(:wat::core::defn :wat::rete::compile-query
  [state <- :wat::rete::CompileState
   q     <- :wat::rete::Query]
  -> :wat::rete::CompileState
  (:wat::core::let [lhs        (:wat::rete::sort-lhs (:wat::rete::Query/lhs q))
                    qname      (:wat::rete::Query/name q)
                    init-acc   (:wat::rete::CondFoldAcc
                                 :state state
                                 :parent-ids (:wat::core::PersistentVector))
                    final-acc  (:wat::core::foldl :wat::rete::compile-condition init-acc lhs)
                    state2     (:wat::rete::CondFoldAcc/state      final-acc)
                    pids       (:wat::rete::CondFoldAcc/parent-ids final-acc)
                    network2   (:wat::rete::CompileState/network state2)
                    next-id2   (:wat::rete::CompileState/next-id state2)
                    qnode      (:wat::rete::QueryNode
                                 :id next-id2
                                 :query-name qname
                                 :param-keys (:wat::rete::Query/params q))
                    net3       (:wat::core::PersistentMap/assoc network2 next-id2 qnode)
                    net4       (:wat::rete::wire-parents net3 pids next-id2)]
    (:wat::rete::CompileState :network net4 :next-id (:wat::core::i64::+ next-id2 1)
      :dedup (:wat::rete::CompileState/dedup state2))))

;; compile — rules only (existing callers). Use compile-all to add queries.
(:wat::core::defn :wat::rete::compile
  [rules <- :wat::core::PersistentVector<wat::rete::Rule>]
  -> :wat::rete::Session
  (:wat::rete::compile-all rules (:wat::core::PersistentVector)))

;; compile-all — rules + queries (Clara mk-session mixes both).
(:wat::core::defn :wat::rete::compile-all
  [rules   <- :wat::core::PersistentVector<wat::rete::Rule>
   queries <- :wat::core::PersistentVector<wat::rete::Query>]
  -> :wat::rete::Session
  (:wat::core::let [init-state (:wat::rete::CompileState
                                  :network (:wat::core::PersistentMap)
                                  :next-id 0
                                  :dedup (:wat::core::HashMap :wat::core::String :wat::core::i64))
                    after-rules (:wat::core::foldl :wat::rete::compile-rule init-state rules)
                    final-state (:wat::core::foldl :wat::rete::compile-query after-rules queries)
                    network  (:wat::rete::CompileState/network final-state)
                    next-id  (:wat::rete::CompileState/next-id final-state)
                    empty-pm (:wat::core::PersistentMap)
                    empty-pv (:wat::core::PersistentVector)]
    (:wat::rete::Session
       :network network
       :rules rules
       :alpha-memory empty-pm
       :beta-memory empty-pm
       :production-memory empty-pm
       :facts empty-pv
       :next-id next-id
       :query-memory empty-pm)))

;; ─── insert + fire-rules ────────────────────────────────────────────────────────

;; insert-spec — the wat reference engine (the SPEC / differential oracle). Stages a fact into
;; the session's working memory. Zero activation.
;; WHY zero activation: the WM stays open while the caller stages multiple facts;
;; fire-rules is the lock that runs them through the network all at once.
;; WHY reconstruct Session: Record/assoc returns the base :wat::core::Record type; the
;; typed Session constructor preserves the concrete return type for the checker.
(:wat::core::defn :wat::rete::insert-spec
  [session <- :wat::rete::Session
   fact    <- :wat::core::Record]
  -> :wat::rete::Session
  (:wat::rete::Session
    :network (:wat::rete::Session/network           session)
    :rules (:wat::rete::Session/rules             session)
    :alpha-memory (:wat::rete::Session/alpha-memory      session)
    :beta-memory (:wat::rete::Session/beta-memory       session)
    :production-memory (:wat::rete::Session/production-memory session)
    :facts (:wat::core::PersistentVector/conj (:wat::rete::Session/facts session) fact)
    :next-id (:wat::rete::Session/next-id           session)
    :query-memory (:wat::rete::Session/query-memory session)))

;; insert-all-spec — the wat reference engine (the SPEC / differential oracle) for BATCH insert.
;; Stages every fact in `facts` into the session's working memory: N chained insert-spec calls,
;; folded left→right so caller order is preserved. Zero activation — the exact insert-spec
;; contract, N times over (rete.wat:828-830 — WM stays open until fire-rules).
(:wat::core::defn :wat::rete::insert-all-spec
  [session <- :wat::rete::Session
   facts   <- :wat::core::PersistentVector<wat::core::Record>]
  -> :wat::rete::Session
  (:wat::core::foldl
    :wat::rete::insert-spec
    session
    facts))

;; insert-all — THE PUBLIC BATCH VERB. Delegates to the native Rust `insert-all'`: resolves
;; `facts` by name through the class's RecordDef (never a positional index) and extends the
;; Session's `facts` by every element of `facts` in ONE rebuild — not N rebuilds (`insert` × N).
;; Zero activation — same contract as insert-all-spec / insert-spec. Observationally equivalent
;; to insert-all-spec (proven by the insert-all differential); the native kernel is the fast
;; impl, the spec keeps it honest.
(:wat::core::defn :wat::rete::insert-all
  [session <- :wat::rete::Session
   facts   <- :wat::core::PersistentVector<wat::core::Record>]
  -> :wat::rete::Session
  (:wat::rete::insert-all' session facts))

;; insert — THE PUBLIC PRODUCTION VERB. A `defclause` of two arities:
;;
;;   2-ary   — UNCHANGED, byte for byte: delegates straight to the native `insert'`. This is the
;;             streaming hot path (the chaos engine takes facts ONE AT A TIME off a wire) and it
;;             MUST NOT be re-routed through insert-all — that would force a one-element
;;             PersistentVector allocation onto the case that matters most, buying nothing.
;;             (DESIGN-STONE-insert-all.md ★ THE ONE CONTRACT DECISION / BRIEF STOP-1.)
;;   3+-ary  — sugar: `(:wat::rete::insert session f1 f2 f3)` collects `f2 f3` into `rest` (the
;;             typed rest-param shape `:wat::core::+` proves works, wat/core.wat:58-99) and
;;             assembles `fact :: rest` into one PersistentVector before delegating to
;;             `insert-all` — the real primitive Clara ships (`rules.cljc:11,17`) and we did not,
;;             until this stone.
;;
;; ★ `fact`'s declared type is a bare type-var `T`, NOT `:wat::core::Record` — a runtime-dispatch
;; constraint, not a stylistic choice. `defclause`'s clause-selection matcher
;; (`value_matches_type_by_name`, runtime.rs:7112) special-cases `Value::Aggregate` by comparing
;; the declared Path string against the value's CONCRETE class (Arc 259 S2c-ii.0 — built for a
;; defclause keyed on one specific class, e.g. `:user::Tag`). No concrete record's class is ever
;; literally "wat::core::Record" (the dynamic/open supertype every user record inhabits), so a
;; clause declaring `fact <- :wat::core::Record` can NEVER match any real fact at runtime —
;; confirmed empirically (`NoMatchingClause`, every attempted clause failing arg-type match). A
;; bare type-var hits the `is_type_var` wildcard branch instead (unconditional match), so arity
;; alone discriminates the two clauses below, exactly as needed. `insert'` / `insert-all` (both
;; plain `defn`, not `defclause`) are unaffected — their `:wat::core::Record` params are checked
;; statically by the ordinary checker, which already understands Record as an open supertype.
(:wat::core::defclause :wat::rete::insert
  ([session <- :wat::rete::Session
    fact    <- :wat::core::Record] -> :wat::rete::Session
    (:wat::rete::insert' session fact))
  ([session <- :wat::rete::Session
    fact    <- :wat::core::Record
    & rest  <- :wat::core::Vector<wat::core::Record>] -> :wat::rete::Session
    (:wat::rete::insert-all session
      (:wat::core::foldl
        (:wat::core::fn [acc <- :wat::core::PersistentVector<wat::core::Record>
                         f   <- :T] -> :wat::core::PersistentVector<wat::core::Record>
          (:wat::core::PersistentVector/conj acc f))
        (:wat::core::PersistentVector/conj (:wat::core::PersistentVector) fact)
        rest))))

;; activate-fact — fold step: try one fact against a single AlphaNode's condition.
;; On a match, appends an Element(fact, bindings) to alpha-memory at alpha-id;
;; on no match, returns alpha-memory unchanged.
;; WHY two assoc branches: avoids a nested intermediate PV match (the empty PV
;; would be typed PersistentVector<?> and conflict with the Some-arm's PV type).
(:wat::core::defn :wat::rete::activate-fact
  [alpha-id  <- :wat::core::i64
   cond      <- :wat::WatAST
   alpha-mem <- :wat::core::PersistentMap
   fact      <- :wat::core::Record]
  -> :wat::core::PersistentMap
  (:wat::core::let [match-result (:wat::rete::alpha-match cond fact)]
    (:wat::core::match match-result 
      ((:wat::core::Some bindings)
       ;; WHY staged-fact = Element(record, bindings): stores the original typed record
       ;; (not a map) so downstream queries + TM provenance can use the fact type directly.
       (:wat::core::let [staged-fact (:wat::rete::Element :fact fact :bindings bindings)]
         (:wat::core::match (:wat::core::PersistentMap/get alpha-mem alpha-id) 
           ((:wat::core::Some pv)
            (:wat::core::PersistentMap/assoc alpha-mem alpha-id
              (:wat::core::PersistentVector/conj pv staged-fact)))
           (:wat::core::None
            (:wat::core::PersistentMap/assoc alpha-mem alpha-id
              (:wat::core::PersistentVector/conj (:wat::core::PersistentVector) staged-fact))))))
      (:wat::core::None alpha-mem))))

;; activate-alpha — fold step: run all staged facts through a single AlphaNode.
;; Skips non-AlphaNode entries (join nodes, production nodes, etc.).
;; acc = alpha-memory PersistentMap threaded through the network-keys fold.
(:wat::core::defn :wat::rete::activate-alpha
  [facts     <- :wat::core::PersistentVector
   network   <- :wat::core::PersistentMap
   alpha-mem <- :wat::core::PersistentMap
   node-id   <- :wat::core::i64]
  -> :wat::core::PersistentMap
  (:wat::core::let [node (:wat::core::Option/expect  
                             (:wat::core::PersistentMap/get network node-id)
                             "activate-alpha: node not found")
                    kind (:wat::rete::node-kind-label node)]
    (:wat::core::cond
      ((:wat::core::= kind "AlphaNode")
       ;; WHY get tests[0]: AlphaNode.tests is a PV; the first element is the single
       ;; condition form (WatAST) compiled from the rule's LHS clause.
       (:wat::core::let [cond (:wat::core::Option/expect  
                                  (:wat::core::get (:wat::rete::AlphaNode/tests node) 0)
                                  "activate-alpha: AlphaNode has no tests")]
         (:wat::core::foldl
           (:wat::core::fn [acc  <- :wat::core::PersistentMap
                            fact <- :wat::core::Record]
             -> :wat::core::PersistentMap
             (:wat::rete::activate-fact node-id cond acc fact))
           alpha-mem
           facts)))
      (:else alpha-mem))))

;; seed-token — build a single Token seeding the beta chain from one Element.
;; The support entry is a typed tuple (fact, alpha-id); the bindings are the
;; Element's alpha-bindings carried straight through (root-join adds no new bindings).
;; WHY Tuple: the pair is heterogeneous — a Record plus an i64 — which a bare PV
;; cannot honestly type.  The tuple form is what Token.matches is declared to hold.
(:wat::core::defn :wat::rete::seed-token
  [el       <- :wat::rete::Element
   alpha-id <- :wat::core::i64]
  -> :wat::rete::Token
  (:wat::rete::Token
    :matches (:wat::core::PersistentVector
      (:wat::core::Tuple (:wat::rete::Element/fact el) alpha-id))
    :bindings (:wat::rete::Element/bindings el)))

;; append-token — append a Token to beta-memory at root-join-id; create the PV if absent.
;; WHY two assoc branches: same rationale as activate-fact — avoids a nested intermediate
;; PV match where the empty branch would have an under-typed PersistentVector<?>.
(:wat::core::defn :wat::rete::append-token
  [beta-mem     <- :wat::core::PersistentMap
   root-join-id <- :wat::core::i64
   tok          <- :wat::rete::Token]
  -> :wat::core::PersistentMap
  (:wat::core::match (:wat::core::PersistentMap/get beta-mem root-join-id) 
    ((:wat::core::Some pv)
     (:wat::core::PersistentMap/assoc beta-mem root-join-id
       (:wat::core::PersistentVector/conj pv tok)))
    (:wat::core::None
     (:wat::core::PersistentMap/assoc beta-mem root-join-id
       (:wat::core::PersistentVector/conj (:wat::core::PersistentVector) tok)))))

;; seed-root-join-children — for one AlphaNode that has Elements, follow its children;
;; for each child that is a RootJoinNode, seed one Token per Element into beta-memory.
;; WHY fold over children then fold over elements: the outer fold fans out to all
;; RootJoinNode children; the inner fold seeds each Element as a Token.
(:wat::core::defn :wat::rete::seed-root-join-children
  [alpha-id  <- :wat::core::i64
   els       <- :wat::core::PersistentVector
   network   <- :wat::core::PersistentMap
   beta-mem  <- :wat::core::PersistentMap]
  -> :wat::core::PersistentMap
  (:wat::core::let [alpha-node (:wat::core::Option/expect  
                                   (:wat::core::PersistentMap/get network alpha-id)
                                   "seed-root-join-children: alpha node not found")
                    child-ids  (:wat::rete::AlphaNode/children alpha-node)]
    (:wat::core::foldl
      (:wat::core::fn [bm       <- :wat::core::PersistentMap
                       child-id <- :wat::core::i64]
        -> :wat::core::PersistentMap
        (:wat::core::let [child-node (:wat::core::Option/expect  
                                         (:wat::core::PersistentMap/get network child-id)
                                         "seed-root-join-children: child node not found")]
          (:wat::core::cond
            ((:wat::core::= (:wat::rete::node-kind-label child-node) "RootJoinNode")
             ;; WHY fold els here: seed one Token per Element into this RootJoinNode's slot.
             (:wat::core::foldl
               (:wat::core::fn [bm2 <- :wat::core::PersistentMap
                                el  <- :wat::rete::Element]
                 -> :wat::core::PersistentMap
                 (:wat::rete::append-token bm2 child-id (:wat::rete::seed-token el alpha-id)))
               bm
               els))
            (:else bm))))
      beta-mem
      child-ids)))

;; root-join-pass — fold step for the root-join pass: for each network node id,
;; if it is an AlphaNode with Elements in alpha-memory, seed its RootJoinNode children.
(:wat::core::defn :wat::rete::root-join-pass
  [alpha-mem <- :wat::core::PersistentMap
   network   <- :wat::core::PersistentMap
   beta-mem  <- :wat::core::PersistentMap
   node-id   <- :wat::core::i64]
  -> :wat::core::PersistentMap
  (:wat::core::let [node (:wat::core::Option/expect  
                             (:wat::core::PersistentMap/get network node-id)
                             "root-join-pass: node not found")]
    (:wat::core::cond
      ((:wat::core::= (:wat::rete::node-kind-label node) "AlphaNode")
       (:wat::core::match (:wat::core::PersistentMap/get alpha-mem node-id) 
         ((:wat::core::Some els)
          (:wat::rete::seed-root-join-children node-id els network beta-mem))
         (:wat::core::None beta-mem)))
      (:else beta-mem))))

;; ─── hash-join pass (stone 3b) ──────────────────────────────────────────────

;; alpha-feeding — reverse-lookup: find the AlphaNode id whose children contains hj-id.
;; WHY reverse-lookup: the network stores forward edges (alpha → join via children);
;; the join semantics require the RIGHT memory = alpha-memory of the feeding alpha.
;; Folds over all node-ids; carries the found alpha-id (>= 0) or -1 (not yet found).
(:wat::core::defn :wat::rete::alpha-feeding
  [hj-id   <- :wat::core::i64
   network  <- :wat::core::PersistentMap]
  -> :wat::core::i64
  (:wat::core::foldl
    (:wat::core::fn [found   <- :wat::core::i64
                     node-id <- :wat::core::i64]
      -> :wat::core::i64
      (:wat::core::if (:wat::core::i64::>= found 0)
        found
        (:wat::core::let [node (:wat::core::Option/expect  
                                   (:wat::core::PersistentMap/get network node-id)
                                   "alpha-feeding: node not found")]
          (:wat::core::if (:wat::core::= (:wat::rete::node-kind-label node) "AlphaNode")
            (:wat::core::if (:wat::core::PersistentVector/contains?
                               (:wat::rete::AlphaNode/children node)
                               hj-id)
              node-id
              -1)
            -1))))
    -1
    (:wat::core::PersistentMap/keys network)))

;; token-element-compatible? — check shared-variable agreement between a Token and an Element.
;; Fold over element.bindings keys: if a key is ALSO in token.bindings with a DIFFERENT value
;; → incompatible. A variable present only on one side never conflicts.
;; WHY contains-key? before get: avoids comparing Option(None) against Option(Some(v)),
;; which would incorrectly flag a token-missing key as a conflict.
(:wat::core::defn :wat::rete::token-element-compatible?
  [tok <- :wat::rete::Token
   el  <- :wat::rete::Element]
  -> :wat::core::bool
  (:wat::core::let [t-binds (:wat::rete::Token/bindings   tok)
                    e-binds (:wat::rete::Element/bindings  el)]
    (:wat::core::foldl
      (:wat::core::fn [compat <- :wat::core::bool
                       k      <- :wat::core::String]
        -> :wat::core::bool
        (:wat::core::if (:wat::core::not compat)
          false
          (:wat::core::if (:wat::core::PersistentMap/contains-key? t-binds k)
            ;; key in BOTH — the Options agree iff the underlying values are equal
            ;; (e-binds always has the key since we iterate its own keys → Some == Some)
            (:wat::core::= (:wat::core::PersistentMap/get t-binds k)
                           (:wat::core::PersistentMap/get e-binds k))
            true)))
      true
      (:wat::core::PersistentMap/keys e-binds))))

;; extend-token — produce a new Token that merges an Element's fact and bindings.
;; matches: append (Tuple element.fact alpha-id) — the provenance support entry.
;; bindings: fold element.bindings into token.bindings (assoc each entry; shared vars
;;           are idempotent — they agree by construction after compatible? passed).
(:wat::core::defn :wat::rete::extend-token
  [tok      <- :wat::rete::Token
   el       <- :wat::rete::Element
   alpha-id <- :wat::core::i64]
  -> :wat::rete::Token
  (:wat::core::let [e-binds     (:wat::rete::Element/bindings el)
                    new-matches (:wat::core::PersistentVector/conj
                                   (:wat::rete::Token/matches tok)
                                   (:wat::core::Tuple (:wat::rete::Element/fact el) alpha-id))
                    new-binds   (:wat::core::foldl
                                   (:wat::core::fn [bm <- :wat::core::PersistentMap
                                                    k  <- :wat::core::String]
                                     -> :wat::core::PersistentMap
                                     (:wat::core::match (:wat::core::PersistentMap/get e-binds k)
                                                        
                                       ((:wat::core::Some v)
                                        (:wat::core::PersistentMap/assoc bm k v))
                                       (:wat::core::None bm)))
                                   (:wat::rete::Token/bindings tok)
                                   (:wat::core::PersistentMap/keys e-binds))]
    (:wat::rete::Token :matches new-matches :bindings new-binds)))

;; cross-join-node — cross LEFT (tokens) × RIGHT (elements) for one HashJoinNode.
;; For each compatible (token, element) pair, extend the token and append to beta-mem at hj-id.
;; WHY nested foldl: outer fan-out over tokens, inner fan-out over elements; pure accumulator.
(:wat::core::defn :wat::rete::cross-join-node
  [tokens   <- :wat::core::PersistentVector
   elements <- :wat::core::PersistentVector
   hj-id    <- :wat::core::i64
   alpha-id <- :wat::core::i64
   beta-mem <- :wat::core::PersistentMap]
  -> :wat::core::PersistentMap
  (:wat::core::foldl
    (:wat::core::fn [bm  <- :wat::core::PersistentMap
                     tok <- :wat::rete::Token]
      -> :wat::core::PersistentMap
      (:wat::core::foldl
        (:wat::core::fn [bm2 <- :wat::core::PersistentMap
                         el  <- :wat::rete::Element]
          -> :wat::core::PersistentMap
          (:wat::core::if (:wat::rete::token-element-compatible? tok el)
            (:wat::rete::append-token bm2 hj-id (:wat::rete::extend-token tok el alpha-id))
            bm2))
        bm
        elements))
    beta-mem
    tokens))

;; hash-join-pass — fold step: propagate tokens from a beta node to its HashJoinNode children.
;; For each node-id: if it is a left-parent (RootJoin / HashJoin / Test / Negation / Exists /
;; Accumulate) with tokens in beta-memory, for each HashJoinNode child J, cross
;; beta-memory[here] × alpha-memory[alpha-feeding(J)].
;; WHY Test/Negation/Exists/Accumulate count: compile will parent a HashJoin on a mid-chain
;; :where / :not / :exists / accumulate (Clara does; so must we). A kind gate that only
;; accepted Root/Hash starved that child — A1, both impls.
;; WHY topological pass in node-id order: compile assigns IDs left-to-right, and fire-once
;; now populate-then-emits per node (filter/accum first, then this pass), so a TestNode's
;; beta is already filled when we emit to its HashJoin children. DAG; monotone insertions only.
(:wat::core::defn :wat::rete::hash-join-pass
  [alpha-mem <- :wat::core::PersistentMap
   network   <- :wat::core::PersistentMap
   beta-mem  <- :wat::core::PersistentMap
   node-id   <- :wat::core::i64]
  -> :wat::core::PersistentMap
  (:wat::core::let [node (:wat::core::Option/expect  
                             (:wat::core::PersistentMap/get network node-id)
                             "hash-join-pass: node not found")
                    kind (:wat::rete::node-kind-label node)]
    (:wat::core::if (:wat::core::or (:wat::core::= kind "RootJoinNode")
                                    (:wat::core::or (:wat::core::= kind "HashJoinNode")
                                      (:wat::core::or (:wat::core::= kind "TestNode")
                                        (:wat::core::or (:wat::core::= kind "NegationNode")
                                          (:wat::core::or (:wat::core::= kind "ExistsNode")
                                                          (:wat::core::= kind "AccumulateNode"))))))
      (:wat::core::match (:wat::core::PersistentMap/get beta-mem node-id) 
        ((:wat::core::Some tokens)
         (:wat::core::foldl
           (:wat::core::fn [bm       <- :wat::core::PersistentMap
                            child-id <- :wat::core::i64]
             -> :wat::core::PersistentMap
             (:wat::core::let [child (:wat::core::Option/expect  
                                         (:wat::core::PersistentMap/get network child-id)
                                         "hash-join-pass: child not found")]
               (:wat::core::if (:wat::core::= (:wat::rete::node-kind-label child) "HashJoinNode")
                 ;; WHY match on alpha-mem: no elements on the right → no matches possible;
                 ;; skip the cross to avoid building an empty PV (avoids the untyped-PV hazard).
                 (:wat::core::let [aid (:wat::rete::alpha-feeding child-id network)]
                   (:wat::core::match (:wat::core::PersistentMap/get alpha-mem aid)
                                      
                     ((:wat::core::Some els)
                      (:wat::rete::cross-join-node tokens els child-id aid bm))
                     (:wat::core::None bm)))
                 bm)))
           beta-mem
           (:wat::rete::node-children-ids node)))
        (:wat::core::None beta-mem))
      beta-mem)))

;; ─── production pass (stone 4a) ────────────────────────────────────────────

;; node-parent — reverse-lookup: find the id of the node whose node-children-ids contains child-id.
;; Returns -1 if no parent is found (e.g. the root of the network).
;; WHY kind-agnostic via node-children-ids: a ProductionNode's parent is a RootJoinNode (1-condition rule)
;; OR a HashJoinNode (multi-condition rule); dispatching on node-children-ids covers both without
;; hard-coding the parent kind. Mirrors alpha-feeding but uses the shared node-children-ids accessor.
;; node-parents — every node that names `child-id` as a child. Condition `:or`
;; wires one ProductionNode to N arm terminals; fire must read ALL of them.
(:wat::core::defn :wat::rete::node-parents
  [child-id <- :wat::core::i64
   network  <- :wat::core::PersistentMap]
  -> :wat::core::PersistentVector<wat::core::i64>
  (:wat::core::foldl
    (:wat::core::fn [acc     <- :wat::core::PersistentVector<wat::core::i64>
                     node-id <- :wat::core::i64]
      -> :wat::core::PersistentVector<wat::core::i64>
      (:wat::core::let [node (:wat::core::Option/expect
                                (:wat::core::PersistentMap/get network node-id)
                                "node-parents: node not found")]
        (:wat::core::if (:wat::core::PersistentVector/contains?
                           (:wat::rete::node-children-ids node)
                           child-id)
          (:wat::core::PersistentVector/conj acc node-id)
          acc)))
    (:wat::core::PersistentVector)
    (:wat::core::PersistentMap/keys network)))

(:wat::core::defn :wat::rete::node-parent
  [child-id <- :wat::core::i64
   network  <- :wat::core::PersistentMap]
  -> :wat::core::i64
  (:wat::core::foldl
    (:wat::core::fn [found   <- :wat::core::i64
                     node-id <- :wat::core::i64]
      -> :wat::core::i64
      (:wat::core::if (:wat::core::i64::>= found 0)
        found
        (:wat::core::let [node (:wat::core::Option/expect  
                                   (:wat::core::PersistentMap/get network node-id)
                                   "node-parent: node not found")]
          (:wat::core::if (:wat::core::PersistentVector/contains?
                             (:wat::rete::node-children-ids node)
                             child-id)
            node-id
            -1))))
    -1
    (:wat::core::PersistentMap/keys network)))

;; tokens-from-parents — concat beta-memory tokens from every parent.
;; Condition `:or` leaves N terminals; a later Test/:not/:exists/accum
;; (and production) must read ALL of them, not the first parent only.
(:wat::core::defn :wat::rete::tokens-from-parents
  [beta-mem   <- :wat::core::PersistentMap
   parent-ids <- :wat::core::PersistentVector<wat::core::i64>]
  -> :wat::core::PersistentVector<wat::rete::Token>
  (:wat::core::foldl
    (:wat::core::fn [acc <- :wat::core::PersistentVector<wat::rete::Token>
                     pid <- :wat::core::i64]
      -> :wat::core::PersistentVector<wat::rete::Token>
      (:wat::core::match (:wat::core::PersistentMap/get beta-mem pid)
        ((:wat::core::Some tokens)
         (:wat::core::foldl
           (:wat::core::fn [a <- :wat::core::PersistentVector<wat::rete::Token>
                            t <- :wat::rete::Token]
             -> :wat::core::PersistentVector<wat::rete::Token>
             (:wat::core::PersistentVector/conj a t))
           acc
           tokens))
        (:wat::core::None acc)))
    (:wat::core::PersistentVector)
    parent-ids))

;; rule-by-name — linear find: given a rule name String, return the matching Rule from rules PV.
;; WHY foldl carrying Option: PersistentVector has no early-exit find; foldl short-circuits by
;; passing found values through unchanged (match Some → pass; None → test name; conj on hit).
;; The caller panics on None (a missing rule = a compile bug).
(:wat::core::defn :wat::rete::rule-by-name
  [rules <- :wat::core::PersistentVector<wat::rete::Rule>
   rname <- :wat::core::String]
  -> :wat::rete::Rule
  (:wat::core::Option/expect  
    (:wat::core::foldl
      (:wat::core::fn [found <- :wat::core::Option<wat::rete::Rule>
                       rule  <- :wat::rete::Rule]
        -> :wat::core::Option<wat::rete::Rule>
        (:wat::core::match found 
          ((:wat::core::Some _) found)
          (:wat::core::None
           (:wat::core::if (:wat::core::= (:wat::rete::Rule/name rule) rname)
             (:wat::core::Some rule)
             :wat::core::None))))
      :wat::core::None
      rules)
    "rule-by-name: rule not found"))

;; fire-production — fire one ProductionNode: for each token in beta-memory[parent(P)], evaluate
;; each insert-form in the rule's RHS and conj the derived fact into production-memory[P-id].
;; WHY read beta-memory[parent]: the ProductionNode itself has no beta-memory slot; the tokens
;; that reached P are stored at the parent join node (the final join in the chain).
;; WHY conj-into-prod-mem mirrors append-token: same Some/None branch to avoid untyped-PV hazard.
(:wat::core::defn :wat::rete::fire-production
  [prod-id  <- :wat::core::i64
   network  <- :wat::core::PersistentMap
   beta-mem <- :wat::core::PersistentMap
   rules    <- :wat::core::PersistentVector<wat::rete::Rule>
   prod-mem <- :wat::core::PersistentMap]
  -> :wat::core::PersistentMap
  (:wat::core::let [prod-node  (:wat::core::Option/expect  
                                   (:wat::core::PersistentMap/get network prod-id)
                                   "fire-production: prod node not found")
                    rname      (:wat::rete::ProductionNode/rule-name prod-node)
                    parent-ids (:wat::rete::node-parents prod-id network)
                    rule       (:wat::rete::rule-by-name rules rname)
                    rhs        (:wat::rete::Rule/rhs rule)]
    (:wat::core::foldl
      (:wat::core::fn [pm0 <- :wat::core::PersistentMap
                       parent-id <- :wat::core::i64]
        -> :wat::core::PersistentMap
      (:wat::core::match (:wat::core::PersistentMap/get beta-mem parent-id) 
      ((:wat::core::Some tokens)
       ;; For each token: for each insert-form in rhs: eval-insert → conj into prod-mem[prod-id].
       (:wat::core::foldl
         (:wat::core::fn [pm  <- :wat::core::PersistentMap
                          tok <- :wat::rete::Token]
           -> :wat::core::PersistentMap
           (:wat::core::foldl
             (:wat::core::fn [pm2  <- :wat::core::PersistentMap
                              form <- :wat::WatAST]
               -> :wat::core::PersistentMap
               (:wat::core::let [derived (:wat::rete::eval-insert form (:wat::rete::Token/bindings tok))]
                 (:wat::core::match (:wat::core::PersistentMap/get pm2 prod-id) 
                   ((:wat::core::Some pv)
                    (:wat::core::PersistentMap/assoc pm2 prod-id
                      (:wat::core::PersistentVector/conj pv derived)))
                   (:wat::core::None
                    (:wat::core::PersistentMap/assoc pm2 prod-id
                      (:wat::core::PersistentVector/conj (:wat::core::PersistentVector) derived))))))
             pm
             rhs))
         pm0
         tokens))
      (:wat::core::None pm0)))
      prod-mem
      parent-ids)))

;; test-pass — fold step (6b-ii-a): for each TestNode, filter beta-memory[parent] by eval-test(expr, bindings)
;; into beta-memory[test-id]. Runs AFTER hash-join-pass and BEFORE production-pass.
;; WHY topological: compile assigns IDs in order (join < test), so ascending node-id order is safe.
;; WHY node-parent: mirrors fire-production's parent lookup; a TestNode's parent is a join node
;; (RootJoin or HashJoin) whose tokens are already in beta-memory from the join passes.
(:wat::core::defn :wat::rete::test-pass
  [network  <- :wat::core::PersistentMap
   beta-mem <- :wat::core::PersistentMap
   node-id  <- :wat::core::i64]
  -> :wat::core::PersistentMap
  (:wat::core::let [node (:wat::core::Option/expect  
                             (:wat::core::PersistentMap/get network node-id)
                             "test-pass: node not found")]
    (:wat::core::if (:wat::core::= (:wat::rete::node-kind-label node) "TestNode")
      (:wat::core::let [expr   (:wat::rete::TestNode/expr node)
                        tokens (:wat::rete::tokens-from-parents
                                 beta-mem
                                 (:wat::rete::node-parents node-id network))]
        (:wat::core::foldl
          (:wat::core::fn [bm  <- :wat::core::PersistentMap
                           tok <- :wat::rete::Token]
            -> :wat::core::PersistentMap
            (:wat::core::if (:wat::rete::eval-test expr (:wat::rete::Token/bindings tok))
              (:wat::rete::append-token bm node-id tok)
              bm))
          beta-mem
          tokens))
      beta-mem)))

;; cond-children — wrapper arms of `(:wat::rete::and …)` / `(:or …)`.
(:wat::core::defn :wat::rete::cond-children
  [form <- :wat::WatAST]
  -> :wat::core::PersistentVector<wat::WatAST>
  (:wat::core::let [ch (:wat::core::ast->children form)]
    (:wat::core::foldl
      (:wat::core::fn [acc <- :wat::core::PersistentVector<wat::WatAST>
                       i   <- :wat::core::i64]
        -> :wat::core::PersistentVector<wat::WatAST>
        (:wat::core::PersistentVector/conj acc
          (:wat::core::Option/expect
            (:wat::core::get ch i)
            "cond-children")))
      (:wat::core::PersistentVector)
      (:wat::core::range 1 (:wat::core::length ch)))))

;; binding-extensions — every binding map that satisfies `cond` under `bindings`.
;; Fact: each matching WM fact. `:and`: backtrack. `:or`: concat arms. `:where`: keep or drop.
(:wat::core::defn :wat::rete::binding-extensions
  [cond     <- :wat::WatAST
   facts    <- :wat::core::PersistentVector
   bindings <- :wat::core::PersistentMap]
  -> :wat::core::PersistentVector<wat::core::PersistentMap>
  (:wat::core::let [head-nm (:wat::core::ast-name
                              (:wat::core::first (:wat::core::ast->children cond)))]
    (:wat::core::cond
      ((:wat::core::= head-nm ":wat::rete::and")
       (:wat::core::foldl
         (:wat::core::fn [exts <- :wat::core::PersistentVector<wat::core::PersistentMap>
                          kid  <- :wat::WatAST]
           -> :wat::core::PersistentVector<wat::core::PersistentMap>
           (:wat::core::foldl
             (:wat::core::fn [out <- :wat::core::PersistentVector<wat::core::PersistentMap>
                              ext <- :wat::core::PersistentMap]
               -> :wat::core::PersistentVector<wat::core::PersistentMap>
               (:wat::core::PersistentVector/concat
                 out
                 (:wat::rete::binding-extensions kid facts ext)))
             (:wat::core::PersistentVector)
             exts))
         (:wat::core::PersistentVector/conj (:wat::core::PersistentVector) bindings)
         (:wat::rete::cond-children cond)))
      ((:wat::core::= head-nm ":wat::rete::or")
       (:wat::core::foldl
         (:wat::core::fn [out <- :wat::core::PersistentVector<wat::core::PersistentMap>
                          kid <- :wat::WatAST]
           -> :wat::core::PersistentVector<wat::core::PersistentMap>
           (:wat::core::PersistentVector/concat
             out
             (:wat::rete::binding-extensions kid facts bindings)))
         (:wat::core::PersistentVector)
         (:wat::rete::cond-children cond)))
      ((:wat::core::= head-nm ":wat::rete::where")
       (:wat::core::if (:wat::rete::eval-test
                         (:wat::core::second (:wat::core::ast->children cond))
                         bindings)
         (:wat::core::PersistentVector/conj (:wat::core::PersistentVector) bindings)
         (:wat::core::PersistentVector)))
      ((:wat::core::= head-nm ":wat::rete::not")
       (:wat::core::if (:wat::rete::exists-cond-under
                         (:wat::core::second (:wat::core::ast->children cond))
                         facts bindings)
         (:wat::core::PersistentVector)
         (:wat::core::PersistentVector/conj (:wat::core::PersistentVector) bindings)))
      (:else
       (:wat::core::foldl
         (:wat::core::fn [acc  <- :wat::core::PersistentVector<wat::core::PersistentMap>
                          fact <- :wat::core::Record]
           -> :wat::core::PersistentVector<wat::core::PersistentMap>
           (:wat::core::match (:wat::rete::alpha-match-under cond fact bindings)
             ((:wat::core::Some b)
              (:wat::core::PersistentVector/conj acc b))
             (:wat::core::None acc)))
         (:wat::core::PersistentVector)
         facts)))))

;; exists-cond-under — does the inner :not/:exists condition hold under bindings?
;; A fact: any-fact-matches-under. `:and` of facts: some join of the children exists
;; (Clara [:not [:and [Wind] [Temp]]]).
(:wat::core::defn :wat::rete::exists-cond-under
  [cond     <- :wat::WatAST
   facts    <- :wat::core::PersistentVector
   bindings <- :wat::core::PersistentMap]
  -> :wat::core::bool
  (:wat::core::let [head-nm (:wat::core::ast-name
                              (:wat::core::first (:wat::core::ast->children cond)))]
    (:wat::core::cond
      ((:wat::core::= head-nm ":wat::rete::and")
       (:wat::core::i64::> (:wat::core::length
                             (:wat::rete::binding-extensions cond facts bindings))
                           0))
      ((:wat::core::= head-nm ":wat::rete::or")
       (:wat::core::foldl
         (:wat::core::fn [found <- :wat::core::bool
                          kid   <- :wat::WatAST]
           -> :wat::core::bool
           (:wat::core::if found
             true
             (:wat::rete::exists-cond-under kid facts bindings)))
         false
         (:wat::rete::cond-children cond)))
      ((:wat::core::= head-nm ":wat::rete::where")
       (:wat::rete::eval-test
         (:wat::core::second (:wat::core::ast->children cond))
         bindings))
      ((:wat::core::= head-nm ":wat::rete::not")
       (:wat::core::not
         (:wat::rete::exists-cond-under
           (:wat::core::second (:wat::core::ast->children cond))
           facts bindings)))
      (:else
       (:wat::rete::any-fact-matches-under cond facts bindings)))))

;; distinct-maps — first-wins unique PersistentMaps (Clara exists: two Winds at
;; one loc → one binding).
(:wat::core::defn :wat::rete::distinct-maps
  [maps <- :wat::core::PersistentVector<wat::core::PersistentMap>]
  -> :wat::core::PersistentVector<wat::core::PersistentMap>
  (:wat::core::foldl
    (:wat::core::fn [acc <- :wat::core::PersistentVector<wat::core::PersistentMap>
                     m   <- :wat::core::PersistentMap]
      -> :wat::core::PersistentVector<wat::core::PersistentMap>
      (:wat::core::if (:wat::core::PersistentVector/contains? acc m)
        acc
        (:wat::core::PersistentVector/conj acc m)))
    (:wat::core::PersistentVector)
    maps))

;; tokens-or-empty-seed — parent tokens, or one empty-binding token when the
;; node is leading (`:not`, accumulate). Mid-chain with no parent tokens stays empty.
(:wat::core::defn :wat::rete::tokens-or-empty-seed
  [network  <- :wat::core::PersistentMap
   beta-mem <- :wat::core::PersistentMap
   node-id  <- :wat::core::i64]
  -> :wat::core::PersistentVector<wat::rete::Token>
  (:wat::core::let [pids (:wat::rete::node-parents node-id network)]
    (:wat::core::if (:wat::core::= (:wat::core::length pids) 0)
      (:wat::core::PersistentVector/conj
        (:wat::core::PersistentVector)
        (:wat::rete::Token
          :matches (:wat::core::PersistentVector)
          :bindings (:wat::core::PersistentMap)))
      (:wat::rete::tokens-from-parents beta-mem pids))))

;; any-fact-matches-under — oracle :not / :exists beta check.
;; Re-runs the inner condition against every staged fact WITH the token's
;; bindings seeded (Clara join-filter). Shared-var agreement is not enough:
;; `?v < ?m` after an accum names a left-bound var that empty-seed alpha never
;; sees, so the negated alpha stays empty and :not falsely passes.
(:wat::core::defn :wat::rete::any-fact-matches-under
  [cond     <- :wat::WatAST
   facts    <- :wat::core::PersistentVector
   bindings <- :wat::core::PersistentMap]
  -> :wat::core::bool
  (:wat::core::foldl
    (:wat::core::fn [found <- :wat::core::bool
                     fact  <- :wat::core::Record]
      -> :wat::core::bool
      (:wat::core::if found
        true
        (:wat::core::match (:wat::rete::alpha-match-under cond fact bindings)
          ((:wat::core::Some _) true)
          (:wat::core::None false))))
    false
    facts))

;; filter-pass — unified fold step (7-a): replaces the standalone test-pass.
;; Dispatches by node kind:
;;   TestNode     → eval-test filter (same as the old test-pass).
;;   NegationNode → negation filter: pass the un-extended token iff ZERO staged
;;                  facts match the inner cond under the token's bindings.
;; facts is threaded in (the :not / :exists filter needs the input bag);
;; TestNode ignores it.
;; WHY unified fold: any interleaving of :where/:not in a condition chain is correct because
;; fire-once walks node-ids in topological (ascending) order and, per node, populate
;; (this pass) THEN emit (hash-join-pass). A filter reads its parent's beta (already
;; written); a later HashJoin then reads THIS node's beta. The old "all joins, then all
;; filters" split made Join→Test→Join starve — the comment was true only for trailing filters.
(:wat::core::defn :wat::rete::filter-pass
  [network   <- :wat::core::PersistentMap
   alpha-mem <- :wat::core::PersistentMap
   facts     <- :wat::core::PersistentVector
   beta-mem  <- :wat::core::PersistentMap
   node-id   <- :wat::core::i64]
  -> :wat::core::PersistentMap
  (:wat::core::let [node (:wat::core::Option/expect  
                             (:wat::core::PersistentMap/get network node-id)
                             "filter-pass: node not found")
                    kind (:wat::rete::node-kind-label node)]
    (:wat::core::cond
      ((:wat::core::= kind "TestNode")
       ;; eval-test filter: keep token iff expr evaluates true under token's bindings.
       ;; Read EVERY parent — a Test after `:or` hangs off N arm terminals.
       (:wat::core::let [expr   (:wat::rete::TestNode/expr node)
                         tokens (:wat::rete::tokens-from-parents
                                  beta-mem
                                  (:wat::rete::node-parents node-id network))]
         (:wat::core::foldl
           (:wat::core::fn [bm  <- :wat::core::PersistentMap
                            tok <- :wat::rete::Token]
             -> :wat::core::PersistentMap
             (:wat::core::if (:wat::rete::eval-test expr (:wat::rete::Token/bindings tok))
               (:wat::rete::append-token bm node-id tok)
               bm))
           beta-mem
           tokens)))
      ((:wat::core::= kind "NegationNode")
       ;; pass the un-extended token iff the inner cond does NOT exist under
       ;; the token (fact, or `:and` of facts). Leading :not seeds one empty token.
       (:wat::core::let [neg-alpha-id (:wat::rete::NegationNode/negated-alpha-id node)
                         tokens       (:wat::rete::tokens-or-empty-seed
                                        network beta-mem node-id)
                         alpha-node   (:wat::core::Option/expect
                                         (:wat::core::PersistentMap/get network neg-alpha-id)
                                         "filter-pass: negated alpha missing")
                         cond         (:wat::core::Option/expect
                                         (:wat::core::get (:wat::rete::AlphaNode/tests alpha-node) 0)
                                         "filter-pass: negated alpha has no cond")]
         (:wat::core::foldl
           (:wat::core::fn [bm  <- :wat::core::PersistentMap
                            tok <- :wat::rete::Token]
             -> :wat::core::PersistentMap
             (:wat::core::if
               (:wat::rete::exists-cond-under
                 cond facts (:wat::rete::Token/bindings tok))
               bm
               (:wat::rete::append-token bm node-id tok)))
           beta-mem
           tokens)))
      ((:wat::core::= kind "ExistsNode")
       ;; Mid-chain: pass the un-extended parent token once if the inner holds.
       ;; Leading: one token per DISTINCT inner binding (Clara test-simple-exists).
       (:wat::core::let [ex-alpha-id (:wat::rete::ExistsNode/exists-alpha-id node)
                         pids        (:wat::rete::node-parents node-id network)
                         alpha-node  (:wat::core::Option/expect
                                        (:wat::core::PersistentMap/get network ex-alpha-id)
                                        "filter-pass: exists alpha missing")
                         cond        (:wat::core::Option/expect
                                        (:wat::core::get (:wat::rete::AlphaNode/tests alpha-node) 0)
                                        "filter-pass: exists alpha has no cond")]
         (:wat::core::if (:wat::core::= (:wat::core::length pids) 0)
           (:wat::core::foldl
             (:wat::core::fn [bm  <- :wat::core::PersistentMap
                              ext <- :wat::core::PersistentMap]
               -> :wat::core::PersistentMap
               (:wat::rete::append-token bm node-id
                 (:wat::rete::Token
                   :matches (:wat::core::PersistentVector)
                   :bindings ext)))
             beta-mem
             (:wat::rete::distinct-maps
               (:wat::rete::binding-extensions
                 cond facts (:wat::core::PersistentMap))))
           (:wat::core::foldl
             (:wat::core::fn [bm  <- :wat::core::PersistentMap
                              tok <- :wat::rete::Token]
               -> :wat::core::PersistentMap
               (:wat::core::if
                 (:wat::rete::exists-cond-under
                   cond facts (:wat::rete::Token/bindings tok))
                 (:wat::rete::append-token bm node-id tok)
                 bm))
             beta-mem
             (:wat::rete::tokens-from-parents beta-mem pids)))))
      (:else beta-mem))))

;; production-pass — fold step: if this node is a ProductionNode, fire it; else pass through.
;; Mirrors hash-join-pass as a fold step over node-ids; seeds with the existing production-memory.
(:wat::core::defn :wat::rete::production-pass
  [network  <- :wat::core::PersistentMap
   beta-mem <- :wat::core::PersistentMap
   rules    <- :wat::core::PersistentVector<wat::rete::Rule>
   prod-mem <- :wat::core::PersistentMap
   node-id  <- :wat::core::i64]
  -> :wat::core::PersistentMap
  (:wat::core::let [node (:wat::core::Option/expect  
                             (:wat::core::PersistentMap/get network node-id)
                             "production-pass: node not found")]
    (:wat::core::if (:wat::core::= (:wat::rete::node-kind-label node) "ProductionNode")
      (:wat::rete::fire-production node-id network beta-mem rules prod-mem)
      prod-mem)))

;; walk-sorted-ids — TCO over a Vector of node-ids. Vector-foldl instantiates
;; PersistentMap as PersistentMap<K,V> and then rejects the existing pass fns
;; (typed bare PersistentMap). Walking by index keeps Acc unparameterized.
;; phase: 0 alpha, 1 root-join, 2 populate-then-emit, 3 production.
(:wat::core::defn :wat::rete::walk-sorted-ids
  [phase   <- :wat::core::i64
   facts   <- :wat::core::PersistentVector
   network <- :wat::core::PersistentMap
   rules   <- :wat::core::PersistentVector<wat::rete::Rule>
   amem    <- :wat::core::PersistentMap
   bmem    <- :wat::core::PersistentMap
   ids     <- :wat::core::Vector<wat::core::i64>
   i       <- :wat::core::i64
   acc     <- :wat::core::PersistentMap]
  -> :wat::core::PersistentMap
  (:wat::core::if (:wat::core::i64::>= i (:wat::core::length ids))
    acc
    (:wat::core::let [node-id (:wat::core::Option/expect
                                 (:wat::core::get ids i)
                                 "walk-sorted-ids: id")
                      acc1    (:wat::core::cond
                                ((:wat::core::= phase 0)
                                 (:wat::rete::activate-alpha facts network acc node-id))
                                ((:wat::core::= phase 1)
                                 (:wat::rete::root-join-pass amem network acc node-id))
                                ((:wat::core::= phase 2)
                                 (:wat::rete::hash-join-pass amem network
                                   (:wat::rete::filter-pass network amem facts
                                     (:wat::rete::accumulate-pass network amem acc node-id)
                                     node-id)
                                   node-id))
                                (:else
                                 (:wat::rete::production-pass network bmem rules acc node-id)))]
      (:wat::rete::walk-sorted-ids phase facts network rules amem bmem ids
        (:wat::core::i64::+ i 1) acc1))))

;; collect-query-memory — QueryNode name → parent-token bindings (the fire's answers).
(:wat::core::defn :wat::rete::collect-query-memory
  [network  <- :wat::core::PersistentMap
   beta-mem <- :wat::core::PersistentMap]
  -> :wat::core::PersistentMap
  (:wat::core::foldl
    (:wat::core::fn [acc     <- :wat::core::PersistentMap
                     node-id <- :wat::core::i64]
      -> :wat::core::PersistentMap
      (:wat::core::let [node (:wat::core::Option/expect
                                (:wat::core::PersistentMap/get network node-id)
                                "collect-query-memory: node")]
        (:wat::core::if (:wat::core::= (:wat::rete::node-kind-label node) "QueryNode")
          (:wat::core::let [qname (:wat::rete::QueryNode/query-name node)
                            pids  (:wat::rete::node-parents node-id network)
                            toks  (:wat::rete::tokens-from-parents beta-mem pids)
                            maps  (:wat::core::foldl
                                     (:wat::core::fn [a   <- :wat::core::PersistentVector<wat::core::PersistentMap>
                                                      tok <- :wat::rete::Token]
                                       -> :wat::core::PersistentVector<wat::core::PersistentMap>
                                       (:wat::core::PersistentVector/conj a
                                         (:wat::rete::Token/bindings tok)))
                                     (:wat::core::PersistentVector)
                                     toks)]
            (:wat::core::PersistentMap/assoc acc qname maps))
          acc)))
    (:wat::core::PersistentMap)
    (:wat::core::PersistentMap/keys network)))

;; fire-once — single-pass fire cycle: alpha → root-join → hash-join → production.
;; Pure value-semantics: takes a Session, returns a new frozen Session with fresh memories.
;; Recomputes all memories from Session.facts each call (re-run-from-scratch); derived facts
;; go to production-memory only — they do not re-enter facts here (cascade is fire-rules' job).
;; WHY reconstruct Session: same reason as insert (Record/assoc returns :wat::core::Record).
(:wat::core::defn :wat::rete::fire-once
  [session <- :wat::rete::Session]
  -> :wat::rete::Session
  (:wat::core::let [network  (:wat::rete::Session/network session)
                    rules    (:wat::rete::Session/rules   session)
                    facts    (:wat::rete::Session/facts   session)
                    ;; WHY sort: compile mints ids left-to-right, so ascending id IS
                    ;; topological. PersistentMap/keys is HAMT order — not that. The old
                    ;; split (all joins, then all filters) was commute-tolerant. The
                    ;; unified populate-then-emit walk is not: a TestNode visited before
                    ;; its parent HashJoin sees an empty beta and stays empty. node-share
                    ;; (N TestNodes fanning off one shared join) made that flicker:
                    ;; oracle-derived changed every run, sometimes []. Native sorts
                    ;; (sorted_node_ids); the spec must too.
                    node-ids (:wat::core::sort
                                (:wat::core::into (:wat::core::Vector :wat::core::i64)
                                  (:wat::core::PersistentMap/keys network)))
                    empty    (:wat::core::PersistentMap)
                    new-amem (:wat::rete::walk-sorted-ids 0 facts network rules empty empty node-ids 0 empty)
                    new-bmem (:wat::rete::walk-sorted-ids 1 facts network rules new-amem empty node-ids 0 empty)
                    filtered-bmem (:wat::rete::walk-sorted-ids 2 facts network rules new-amem new-bmem node-ids 0 new-bmem)
                    new-pmem (:wat::rete::walk-sorted-ids 3 facts network rules new-amem filtered-bmem node-ids 0 empty)
                    qmem     (:wat::rete::collect-query-memory network filtered-bmem)]
    (:wat::rete::Session
      :network (:wat::rete::Session/network session)
      :rules (:wat::rete::Session/rules   session)
      :alpha-memory new-amem
      :beta-memory filtered-bmem
      :production-memory new-pmem
      :facts facts
      :next-id (:wat::rete::Session/next-id session)
      :query-memory qmem)))

;; collect-derived — flatten production-memory's per-node PV<Record> values into one PV<:wat::core::Record>.
;; WHY foldl-over-values: production-memory is a PersistentMap from node-id to PV<Record>;
;; the outer foldl visits each node's PV, the inner foldl conj's each record into the accumulator.
(:wat::core::defn :wat::rete::collect-derived
  [prod-mem <- :wat::core::PersistentMap]
  -> :wat::core::PersistentVector
  (:wat::core::foldl
    (:wat::core::fn [acc <- :wat::core::PersistentVector
                     pv  <- :wat::core::PersistentVector]
      -> :wat::core::PersistentVector
      (:wat::core::foldl
        (:wat::core::fn [a <- :wat::core::PersistentVector
                         f <- :wat::core::Record]
          -> :wat::core::PersistentVector
          (:wat::core::PersistentVector/conj a f))
        acc
        pv))
    (:wat::core::PersistentVector)
    (:wat::core::PersistentMap/values prod-mem)))

;; merge-facts — fold derived facts into the existing fact PV, conj-ing only new ones (dedup by value-equality).
;; WHY contains?-before-conj: the dedup guard is the termination invariant — if a derived fact is already in
;; facts, re-adding it would grow facts every round and spin the fixpoint forever.
(:wat::core::defn :wat::rete::merge-facts
  [facts   <- :wat::core::PersistentVector
   derived <- :wat::core::PersistentVector]
  -> :wat::core::PersistentVector
  (:wat::core::foldl
    (:wat::core::fn [acc <- :wat::core::PersistentVector
                     f   <- :wat::core::Record]
      -> :wat::core::PersistentVector
      (:wat::core::if (:wat::core::PersistentVector/contains? acc f)
        acc
        (:wat::core::PersistentVector/conj acc f)))
    facts
    derived))

;; fire-fixpoint — internal fixpoint driver over fire-once: re-run the full match over a
;; dedup-growing fact set until a round adds no new fact (monotone-finite termination — datalog property).
;; Re-run-from-scratch (pure replay) each round: fire-once recomputes all memories from Session.facts,
;; so derived facts in facts are matched exactly like input facts on the next round. No incremental
;; delta-propagation (deferred perf path).
;; Internal: the returned Session.facts = the whole closure (input + derived), which is what the
;; matching machinery needs across rounds. The PUBLIC caller (fire-rules) restores facts = input only.
(:wat::core::defn :wat::rete::fire-fixpoint
  [session <- :wat::rete::Session]
  -> :wat::rete::Session
  (:wat::core::let [fired     (:wat::rete::fire-once session)
                    derived   (:wat::rete::collect-derived (:wat::rete::Session/production-memory fired))
                    old-facts (:wat::rete::Session/facts session)
                    new-facts (:wat::rete::merge-facts old-facts derived)]
    (:wat::core::if (:wat::core::= (:wat::core::length new-facts) (:wat::core::length old-facts))
      fired
      (:wat::rete::fire-fixpoint
        (:wat::rete::Session
          :network (:wat::rete::Session/network fired)
          :rules (:wat::rete::Session/rules   fired)
          :alpha-memory (:wat::rete::Session/alpha-memory fired)
          :beta-memory (:wat::rete::Session/beta-memory  fired)
          :production-memory (:wat::rete::Session/production-memory fired)
          :facts new-facts
          :next-id (:wat::rete::Session/next-id fired)
          :query-memory (:wat::rete::Session/query-memory fired))))))

;; ─── stratified negation (arc 300 interstitial) ─────────────────────────────
;;
;; STRATIFICATION: partition rules so every rule negating type T fires only
;; AFTER all rules producing T have run to fixpoint. This fixes non-monotonic
;; negation: a rule consuming NOT(T) cannot fire before T is fully derived and
;; thereby leak a spurious derived fact that is never retracted.
;;
;; Standard stratified-datalog algorithm:
;;   1. Assign each produced-type a stratum number (init 0).
;;   2. Iterate: if rule R negates type N, all types R produces must be at
;;      stratum ≥ stratum[N]+1. Repeat until fixpoint or cycle detected.
;;   3. Group rules by stratum ascending → fire each group to fixpoint before
;;      advancing to the next, threading the accumulated facts forward so
;;      higher-stratum rules see the complete lower-stratum derivation.
;;
;; WHY this location: immediately before fire-rules-spec which it replaces.
;; WHY fire-fixpoint unchanged: it is correct within a stratum (monotone,
;; finite, no negation-ordering hazard). Stratification is the ordering layer.

;; StratifyAcc — sweep accumulator: current type-strata map + change flag.
;; type-strata: HashMap<String,i64> mapping produced-type FQDN → stratum number.
;; changed: true iff this sweep raised any stratum value.
(:wat::core::defrecord :wat::rete::StratifyAcc
  [type-strata <- :wat::core::HashMap<wat::core::String,wat::core::i64>
   changed     <- :wat::core::bool])

;; FireStratAcc — fold accumulator for fire-stratified.
;; facts:   accumulated Session.facts after each stratum (input + all derived so far).
;; derived: dedup union of all derived facts across completed strata.
(:wat::core::defrecord :wat::rete::FireStratAcc
  [facts   <- :wat::core::PersistentVector
   derived <- :wat::core::PersistentVector])

;; rule-produces — extract produced type-FQDNs (colon-free) from a Rule's RHS.
;; Arc 278 Stone A: each RHS entry IS the fact-form directly (:ProducedType …) — the
;; `:wat::rete::insert` wrapper is gone, so the type head is the first child of `form`
;; itself (no more unwrapping a second child).
(:wat::core::defn :wat::rete::rule-produces
  [rule <- :wat::rete::Rule]
  -> :wat::core::PersistentVector<wat::core::String>
  (:wat::core::let [rhs (:wat::rete::Rule/rhs rule)]
    (:wat::core::foldl
      (:wat::core::fn [acc  <- :wat::core::PersistentVector<wat::core::String>
                       form <- :wat::WatAST]
        -> :wat::core::PersistentVector<wat::core::String>
        (:wat::core::let [fact-ch   (:wat::core::ast->children form)
                          type-hd   (:wat::core::first fact-ch)
                          raw-nm    (:wat::core::ast-name type-hd)
                          ;; strip leading colon → bare FQDN matching (:wat::core::type fact)
                          type-nm   (:wat::core::if (:wat::core::= (:wat::core::string::subs raw-nm 0 1) ":")
                                      (:wat::core::string::subs raw-nm 1 (:wat::core::string::length raw-nm))
                                      raw-nm)]
          (:wat::core::PersistentVector/conj acc type-nm)))
      (:wat::core::PersistentVector)
      rhs)))

;; rule-negates — extract negated type-FQDNs (colon-free) from a Rule's LHS.
;; Only (:wat::rete::not <fact-form>) conditions create negative dependency edges;
;; positive conditions, :where, :exists, and accumulate are all ignored here.
(:wat::core::defn :wat::rete::rule-negates
  [rule <- :wat::rete::Rule]
  -> :wat::core::PersistentVector<wat::core::String>
  (:wat::core::let [lhs (:wat::rete::Rule/lhs rule)]
    (:wat::core::foldl
      (:wat::core::fn [acc  <- :wat::core::PersistentVector<wat::core::String>
                       form <- :wat::WatAST]
        -> :wat::core::PersistentVector<wat::core::String>
        (:wat::core::let [form-ch (:wat::core::ast->children form)
                          head    (:wat::core::first form-ch)
                          hd-nm   (:wat::core::ast-name head)]
          (:wat::core::if (:wat::core::= hd-nm ":wat::rete::not")
            ;; second child of (:not <fact-form>) is the negated fact pattern
            (:wat::core::let [fact-form (:wat::core::second form-ch)
                              fact-ch   (:wat::core::ast->children fact-form)
                              type-hd   (:wat::core::first fact-ch)
                              raw-nm    (:wat::core::ast-name type-hd)
                              type-nm   (:wat::core::if (:wat::core::= (:wat::core::string::subs raw-nm 0 1) ":")
                                          (:wat::core::string::subs raw-nm 1 (:wat::core::string::length raw-nm))
                                          raw-nm)]
              (:wat::core::PersistentVector/conj acc type-nm))
            acc)))
      (:wat::core::PersistentVector)
      lhs)))

;; stratify-sweep — one pass over all rules updating type-strata.
;; For each rule: required = max(stratum[n]+1 for n in negated, default 0).
;; For each produced type p: stratum[p] = max(stratum[p], required).
;; Returns StratifyAcc{updated type-strata, changed flag (true if any stratum rose)}.
;; rule-consumes — the fact types a rule reads POSITIVELY (task #94).
;;
;; The stratifier needs this and did not have it. Correct stratification requires BOTH
;;   stratum(r) >= stratum(p)  for every p used POSITIVELY
;;   stratum(r) >  stratum(p)  for every p NEGATED
;; Only the second was implemented, so a rule consuming a fact produced in a HIGHER stratum
;; was left in a LOWER one, fired to fixpoint before its input existed, and never re-fired.
;;
;; Engine forms (:wat::rete::not / where / accumulate / exists) are NOT fact patterns and are
;; excluded by prefix; everything else in an lhs is a user fact type.
(:wat::core::defn :wat::rete::rule-consumes
  [rule <- :wat::rete::Rule]
  -> :wat::core::PersistentVector<wat::core::String>
  (:wat::core::let [lhs (:wat::rete::Rule/lhs rule)]
    (:wat::core::foldl
      (:wat::core::fn [acc  <- :wat::core::PersistentVector<wat::core::String>
                       form <- :wat::WatAST]
        -> :wat::core::PersistentVector<wat::core::String>
        (:wat::core::let [form-ch (:wat::core::ast->children form)
                          head    (:wat::core::first form-ch)
                          hd-nm   (:wat::core::ast-name head)]
          ;; TOTAL prefix test: `subs` is PARTIAL, so the length guard comes FIRST — a head
          ;; shorter than the prefix (":nc::Item" is 9) must answer false, never raise.
          (:wat::core::if (:wat::core::if (:wat::core::i64::>= (:wat::core::string::length hd-nm) 12)
                            (:wat::core::= (:wat::core::string::subs hd-nm 0 12) ":wat::rete::")
                            false)
            acc
            (:wat::core::let [raw-nm  hd-nm
                              type-nm (:wat::core::if (:wat::core::= (:wat::core::string::subs raw-nm 0 1) ":")
                                        (:wat::core::string::subs raw-nm 1 (:wat::core::string::length raw-nm))
                                        raw-nm)]
              (:wat::core::PersistentVector/conj acc type-nm)))))
      (:wat::core::PersistentVector)
      lhs)))

(:wat::core::defn :wat::rete::stratify-sweep
  [rules       <- :wat::core::PersistentVector<wat::rete::Rule>
   type-strata <- :wat::core::HashMap<wat::core::String,wat::core::i64>]
  -> :wat::rete::StratifyAcc
  (:wat::core::foldl
    (:wat::core::fn [acc  <- :wat::rete::StratifyAcc
                     rule <- :wat::rete::Rule]
      -> :wat::rete::StratifyAcc
      (:wat::core::let [ts       (:wat::rete::StratifyAcc/type-strata acc)
                        changed  (:wat::rete::StratifyAcc/changed acc)
                        produced (:wat::rete::rule-produces rule)
                        negated  (:wat::rete::rule-negates rule)
                        consumed (:wat::rete::rule-consumes rule)
                        ;; req-neg = max(stratum[n]+1 for n in negated, default 0)
                        req-neg  (:wat::core::foldl
                                   (:wat::core::fn [mx  <- :wat::core::i64
                                                    neg <- :wat::core::String]
                                     -> :wat::core::i64
                                     (:wat::core::let [ns (:wat::core::match
                                                             (:wat::core::HashMap/get ts neg)
                                                             
                                                           ((:wat::core::Some v) v)
                                                           (:wat::core::None 0))
                                                       v  (:wat::core::i64::+ ns 1)]
                                       (:wat::core::if (:wat::core::i64::> v mx) v mx)))
                                   0
                                   negated)
                        ;; req-pos = max(stratum[c] for c in consumed, default 0) — task #94.
                        ;; NOT +1: a positive consumer may sit in the SAME stratum as its input
                        ;; (that is ordinary forward chaining); it merely may not sit BELOW it.
                        req-pos  (:wat::core::foldl
                                   (:wat::core::fn [mx  <- :wat::core::i64
                                                    con <- :wat::core::String]
                                     -> :wat::core::i64
                                     (:wat::core::let [cs (:wat::core::match
                                                             (:wat::core::HashMap/get ts con)
                                                           ((:wat::core::Some v) v)
                                                           (:wat::core::None 0))]
                                       (:wat::core::if (:wat::core::i64::> cs mx) cs mx)))
                                   0
                                   consumed)
                        required (:wat::core::if (:wat::core::i64::> req-neg req-pos) req-neg req-pos)
                        ;; for each produced type: raise stratum to required if higher
                        new-acc  (:wat::core::foldl
                                   (:wat::core::fn [inner <- :wat::rete::StratifyAcc
                                                    p     <- :wat::core::String]
                                     -> :wat::rete::StratifyAcc
                                     (:wat::core::let [its (:wat::rete::StratifyAcc/type-strata inner)
                                                       ich (:wat::rete::StratifyAcc/changed inner)
                                                       cur (:wat::core::match
                                                              (:wat::core::HashMap/get its p)
                                                              
                                                            ((:wat::core::Some v) v)
                                                            (:wat::core::None 0))]
                                       (:wat::core::if (:wat::core::i64::> required cur)
                                         (:wat::rete::StratifyAcc
                                           :type-strata (:wat::core::HashMap/assoc its p required)
                                           :changed true)
                                         inner)))
                                   (:wat::rete::StratifyAcc :type-strata ts :changed changed)
                                   produced)]
        new-acc))
    (:wat::rete::StratifyAcc :type-strata type-strata :changed false)
    rules))

;; stratify-fix — recursive fixpoint for stratification.
;; Sweeps until no stratum changes (converged) or remaining iterations run out.
;; Raises on negation cycle: rule set is not stratifiable (non-terminating strata).
(:wat::core::defn :wat::rete::stratify-fix
  [rules       <- :wat::core::PersistentVector<wat::rete::Rule>
   type-strata <- :wat::core::HashMap<wat::core::String,wat::core::i64>
   remaining   <- :wat::core::i64]
  -> :wat::core::HashMap<wat::core::String,wat::core::i64>
  (:wat::core::let [result  (:wat::rete::stratify-sweep rules type-strata)
                    changed (:wat::rete::StratifyAcc/changed result)
                    new-ts  (:wat::rete::StratifyAcc/type-strata result)]
    (:wat::core::if (:wat::core::not changed)
      new-ts
      ;; still changing — check for cycle before recursing
      (:wat::core::let [_cycle (:wat::core::Option/expect
                                  (:wat::core::if (:wat::core::i64::> remaining 0)
                                    (:wat::core::Some nil)
                                    :wat::core::None)
                                  "stratify: negation cycle detected — rule set is not stratifiable")]
        (:wat::rete::stratify-fix rules new-ts (:wat::core::i64::- remaining 1))))))

;; rule-stratum — compute the stratum of one rule given the final type-strata.
;; = max(max strata[p] for produced p, max strata[n]+1 for negated n).
(:wat::core::defn :wat::rete::rule-stratum
  [rule        <- :wat::rete::Rule
   type-strata <- :wat::core::HashMap<wat::core::String,wat::core::i64>]
  -> :wat::core::i64
  (:wat::core::let [produced (:wat::rete::rule-produces rule)
                    negated  (:wat::rete::rule-negates rule)
                    from-p   (:wat::core::foldl
                               (:wat::core::fn [mx <- :wat::core::i64
                                                p  <- :wat::core::String]
                                 -> :wat::core::i64
                                 (:wat::core::let [ps (:wat::core::match
                                                         (:wat::core::HashMap/get type-strata p)
                                                         
                                                       ((:wat::core::Some v) v)
                                                       (:wat::core::None 0))]
                                   (:wat::core::if (:wat::core::i64::> ps mx) ps mx)))
                               0
                               produced)
                    from-n   (:wat::core::foldl
                               (:wat::core::fn [mx <- :wat::core::i64
                                                n  <- :wat::core::String]
                                 -> :wat::core::i64
                                 (:wat::core::let [ns (:wat::core::match
                                                         (:wat::core::HashMap/get type-strata n)
                                                         
                                                       ((:wat::core::Some v) v)
                                                       (:wat::core::None 0))
                                                   v  (:wat::core::i64::+ ns 1)]
                                   (:wat::core::if (:wat::core::i64::> v mx) v mx)))
                               0
                               negated)]
    (:wat::core::if (:wat::core::i64::> from-n from-p) from-n from-p)))

;; stratify — compute the type→stratum HashMap for a rule set.
;; Returns HashMap<String,i64> mapping each produced-type FQDN to its stratum number.
;; Raises "negation cycle" if the rule set is not stratifiable (cyclic negation dependency).
(:wat::core::defn :wat::rete::stratify
  [rules <- :wat::core::PersistentVector<wat::rete::Rule>]
  -> :wat::core::HashMap<wat::core::String,wat::core::i64>
  (:wat::core::let [init-ts (:wat::core::HashMap :wat::core::String :wat::core::i64)
                    ;; length(rules)+1 sweeps is always enough for a stratifiable set
                    bound   (:wat::core::i64::+ (:wat::core::length rules) 1)]
    (:wat::rete::stratify-fix rules init-ts bound)))

;; fire-stratified-loop — recursive descent over strata [current..max-s].
;; Filters the original `rules` (typed PersistentVector<Rule>) to the current stratum
;; on each call, avoiding type erasure that would occur from storing rule groups in an
;; outer PersistentVector. Threads (acc-facts, acc-derived) forward across strata.
;;
;; WHY recursive rather than foldl-over-a-PV: foldl would require the inner elements
;; to be declared as PersistentVector (unparameterised), losing Rule type information
;; and causing compile to reject the argument at the call site. Recursive descent on
;; an index always filters the original typed PV — no type information is lost.
(:wat::core::defn :wat::rete::fire-stratified-loop
  [rules       <- :wat::core::PersistentVector<wat::rete::Rule>
   type-strata <- :wat::core::HashMap<wat::core::String,wat::core::i64>
   current     <- :wat::core::i64
   max-s       <- :wat::core::i64
   acc-facts   <- :wat::core::PersistentVector
   acc-derived <- :wat::core::PersistentVector]
  -> :wat::rete::FireStratAcc
  (:wat::core::if (:wat::core::i64::> current max-s)
    (:wat::rete::FireStratAcc :facts acc-facts :derived acc-derived)
    (:wat::core::let [;; Arc 118.2a — `filter` flipped LAZY; `compile` needs `PersistentVector<Rule>`
                      ;; eagerly, so materialize via `into` (was container-preserving from `rules`).
                      stratum-rules (:wat::core::into (:wat::core::PersistentVector)
                                      (:wat::core::filter
                                        (:wat::core::fn [r <- :wat::rete::Rule] -> :wat::core::bool
                                          (:wat::core::= (:wat::rete::rule-stratum r type-strata) current))
                                        rules))
                      ;; fresh compiled network for this stratum only — no shared-alpha edge
                      sub-sess    (:wat::rete::compile stratum-rules)
                      ;; seed with ALL accumulated facts so negation sees complete prior strata
                      sub-sess2   (:wat::core::foldl
                                    (:wat::core::fn [s <- :wat::rete::Session
                                                     f <- :wat::core::Record]
                                      -> :wat::rete::Session
                                      (:wat::rete::insert-spec s f))
                                    sub-sess
                                    acc-facts)
                      fired       (:wat::rete::fire-fixpoint sub-sess2)
                      new-derived (:wat::rete::collect-derived
                                     (:wat::rete::Session/production-memory fired))
                      merged-d    (:wat::rete::merge-facts acc-derived new-derived)
                      ;; advance facts to the post-fixpoint closure (input ∪ derived so far)
                      new-facts   (:wat::rete::Session/facts fired)]
      (:wat::rete::fire-stratified-loop
        rules type-strata
        (:wat::core::i64::+ current 1)
        max-s
        new-facts
        merged-d))))

;; fire-stratified — stratified fixpoint fire: the ORDER-CORRECT engine.
;; Computes type-strata (stratify), finds the highest stratum, then delegates to
;; fire-stratified-loop which fires each stratum [0..max-s] to its own fixpoint in
;; ascending order, threading accumulated facts forward across strata.
;;
;; WHY re-compile each stratum: each stratum's sub-session is a fresh compiled network
;; for ONLY that stratum's rules. This eliminates the shared-alpha duplicate-edge bug
;; (two rules sharing first condition → alpha.children=[join,join] → double derivation)
;; that made Bad=2 when both rules were compiled into a single network.
(:wat::core::defn :wat::rete::fire-stratified
  [session <- :wat::rete::Session]
  -> :wat::rete::Session
  (:wat::core::let [rules     (:wat::rete::Session/rules session)
                    facts     (:wat::rete::Session/facts session)
                    final-ts  (:wat::rete::stratify rules)
                    ;; compute highest stratum number across all rules (0 if rules is empty)
                    max-s     (:wat::core::foldl
                                (:wat::core::fn [mx   <- :wat::core::i64
                                                 rule <- :wat::rete::Rule]
                                  -> :wat::core::i64
                                  (:wat::core::let [rs (:wat::rete::rule-stratum rule final-ts)]
                                    (:wat::core::if (:wat::core::i64::> rs mx) rs mx)))
                                0
                                rules)
                    final-acc (:wat::rete::fire-stratified-loop
                                rules final-ts 0 max-s
                                facts
                                (:wat::core::PersistentVector))
                    all-d     (:wat::rete::FireStratAcc/derived final-acc)
                    ;; pack derived facts into a production-memory structure the caller can query
                    fprod-m   (:wat::core::PersistentMap/assoc (:wat::core::PersistentMap) 0 all-d)
                    closed    (:wat::rete::FireStratAcc/facts final-acc)
                    q-seed    (:wat::rete::Session
                                :network (:wat::rete::Session/network session)
                                :rules (:wat::rete::Session/rules   session)
                                :alpha-memory (:wat::core::PersistentMap)
                                :beta-memory (:wat::core::PersistentMap)
                                :production-memory fprod-m
                                :facts closed
                                :next-id (:wat::rete::Session/next-id session)
                                :query-memory (:wat::core::PersistentMap))
                    q-fired   (:wat::rete::fire-once q-seed)]
    (:wat::rete::Session
      :network (:wat::rete::Session/network session)
      :rules (:wat::rete::Session/rules   session)
      :alpha-memory (:wat::core::PersistentMap)
      :beta-memory (:wat::core::PersistentMap)
      :production-memory fprod-m
      :facts closed
      :next-id (:wat::rete::Session/next-id session)
      :query-memory (:wat::rete::Session/query-memory q-fired))))

;; fire-rules-spec — the wat reference engine (the SPEC / differential oracle).
;; Now delegates to fire-stratified (which handles negation-over-derived correctly)
;; instead of a bare fire-fixpoint. Within each stratum fire-stratified still uses
;; fire-fixpoint — the per-stratum logic is unchanged, only the ordering is fixed.
;; Restores Session.facts = input only (same invariant as before): retract-then-fire
;; recomputes the full closure from the reduced input, so consequences vanish transitively.
(:wat::core::defn :wat::rete::fire-rules-spec
  [session <- :wat::rete::Session]
  -> :wat::rete::Session
  (:wat::core::let [input (:wat::rete::Session/facts session)
                    fired (:wat::rete::fire-stratified session)]
    (:wat::rete::Session
      :network (:wat::rete::Session/network           fired)
      :rules (:wat::rete::Session/rules             fired)
      :alpha-memory (:wat::rete::Session/alpha-memory      fired)
      :beta-memory (:wat::rete::Session/beta-memory       fired)
      :production-memory (:wat::rete::Session/production-memory fired)
      :facts input
      :next-id (:wat::rete::Session/next-id           fired)
      :query-memory (:wat::rete::Session/query-memory fired))))

;; fire-rules — THE PUBLIC PRODUCTION VERB. Delegates to the native Rust delta kernel (`fire-rules'`):
;; semi-naive incremental activation, keyed joins, transient-during-fire/persistent-at-rest. This is what
;; `defrule`/`query` users and the north star run. Observationally equivalent to `fire-rules-spec` (proven
;; by the P4a/deep-cascade/P4c differentials); the native kernel is the fast impl, the spec keeps it honest.
;; (arc 278 P5: wat orchestrates Rust — the user writes `fire-rules`, the engine is native underneath.)
(:wat::core::defn :wat::rete::fire-rules
  [session <- :wat::rete::Session]
  -> :wat::rete::Session
  (:wat::rete::fire-rules' session))

;; fire-rules-explain — OPT-IN diagnostic fire; same closure as fire-rules but also records the
;; support index (derived-fact → Support{rule, token}). Returns Explained{session, support}.
;; EPHEMERAL result — never serialized. Delegates to the native Rust explain entry.
;; (arc 278 P12a: purely additive — the fast fire-rules' / fire-rules path is byte-identical.)
(:wat::core::defn :wat::rete::fire-rules-explain
  [session <- :wat::rete::Session]
  -> :wat::rete::Explained
  (:wat::rete::fire-rules-explain' session))

;; retract — stage a fact removal from Session.facts, by value equality. Zero activation.
;; Symmetric with insert: the caller re-fires (fire-rules recomputes from the reduced input).
;; WHY foldl + not-equals guard: mirrors merge-facts' foldl + contains? idiom; structural = on
;; records makes removal type-safe and value-precise (not identity/pointer removal).
;; WHY stage-only (no fire): same discipline as insert — the WM stays open for multiple staged
;; removals before the caller locks them in with fire-rules.
(:wat::core::defn :wat::rete::retract
  [session <- :wat::rete::Session
   fact    <- :wat::core::Record]
  -> :wat::rete::Session
  (:wat::core::let [old-facts (:wat::rete::Session/facts session)
                    new-facts (:wat::core::foldl
                                 (:wat::core::fn [acc <- :wat::core::PersistentVector<wat::core::Record>
                                                  f   <- :wat::core::Record]
                                   -> :wat::core::PersistentVector<wat::core::Record>
                                   (:wat::core::if (:wat::core::not (:wat::core::= f fact))
                                     (:wat::core::PersistentVector/conj acc f)
                                     acc))
                                 (:wat::core::PersistentVector)
                                 old-facts)]
    (:wat::rete::Session
      :network (:wat::rete::Session/network           session)
      :rules (:wat::rete::Session/rules             session)
      :alpha-memory (:wat::rete::Session/alpha-memory      session)
      :beta-memory (:wat::rete::Session/beta-memory       session)
      :production-memory (:wat::rete::Session/production-memory session)
      :facts new-facts
      :next-id (:wat::rete::Session/next-id           session)
      :query-memory (:wat::rete::Session/query-memory session))))

;; ─── query — ONE mouth: QueryNode harvest, filtered by params ───────────────

;; query-read — binding maps parked on QueryNode at fire, filtered by params.
(:wat::core::defn :wat::rete::query-read
  [session <- :wat::rete::Session
   q       <- :wat::rete::Query
   params  <- :wat::core::PersistentMap]
  -> :wat::core::PersistentVector<wat::core::PersistentMap>
  (:wat::core::let [want (:wat::rete::Query/params q)
                    got  (:wat::core::foldl
                           (:wat::core::fn [acc <- :wat::core::PersistentVector<wat::core::String>
                                            k   <- :wat::core::String]
                             -> :wat::core::PersistentVector<wat::core::String>
                             (:wat::core::PersistentVector/conj acc k))
                           (:wat::core::PersistentVector)
                           (:wat::core::PersistentMap/keys params))
                    missing (:wat::rete::keys-minus want got)
                    extra   (:wat::rete::keys-minus got want)
                    _params (:wat::core::Option/expect
                              (:wat::core::if
                                (:wat::core::if (:wat::core::= (:wat::core::length missing) 0)
                                  (:wat::core::= (:wat::core::length extra) 0)
                                  false)
                                (:wat::core::Some nil)
                                :wat::core::None)
                              "query: params must match the query's :params")
                    raw (:wat::core::match
                           (:wat::core::PersistentMap/get
                             (:wat::rete::Session/query-memory session)
                             (:wat::rete::Query/name q))
                           ((:wat::core::Some pv) pv)
                           (:wat::core::None (:wat::core::PersistentVector)))]
    (:wat::core::if (:wat::core::= (:wat::core::length want) 0)
      raw
      (:wat::core::into (:wat::core::PersistentVector)
        (:wat::core::filter
          (:wat::core::fn [m <- :wat::core::PersistentMap] -> :wat::core::bool
            (:wat::core::foldl
              (:wat::core::fn [ok <- :wat::core::bool
                               k  <- :wat::core::String]
                -> :wat::core::bool
                (:wat::core::if ok
                  (:wat::core::= (:wat::core::PersistentMap/get m k)
                                 (:wat::core::PersistentMap/get params k))
                  false))
              true
              want))
          raw)))))

;; query-params-form — expand-time map builder (a MACRO: a defn head is refused
;; inside another macro by the F5 pure-combinator gate). Recurses in the
;; template so a PersistentVector of mixed keyword/value kwargs never exists.
(:wat::core::defmacro :wat::rete::query-params-form
  [acc <- :wat::WatAST
   & items <- :wat::core::Vector<wat::WatAST>]
  -> :wat::WatAST
  (:wat::core::if (:wat::core::empty? items)
    acc
    (:wat::core::if (:wat::core::empty? (:wat::core::rest items))
      (:wat::core::macro-error "query: param kwargs must come in key/value pairs")
      (:wat::core::let [k (:wat::core::first items)
                        v (:wat::core::first (:wat::core::rest items))
                        knm (:wat::core::ast-name k)
                        kstr (:wat::core::if
                               (:wat::core::= (:wat::core::string::subs knm 0 1) ":")
                               (:wat::core::string::subs knm 1
                                 (:wat::core::string::length knm))
                               knm)]
        `(:wat::rete::query-params-form
           (:wat::core::PersistentMap/assoc ~acc ~kstr ~v)
           ~@(:wat::core::rest (:wat::core::rest items)))))))

;; query — ONE mouth. q is a Query. Optional Clara-shaped kwargs.
;;   (:wat::rete::query session (:wq::all-wind))
;;   (:wat::rete::query session (:wq::temps-at) :?loc "MCI")
(:wat::core::defmacro :wat::rete::query
  [session <- :wat::WatAST
   q       <- :wat::WatAST
   & rest  <- :wat::core::Vector<wat::WatAST>]
  -> :wat::WatAST
  `(:wat::rete::query-read ~session ~q
     (:wat::rete::query-params-form (:wat::core::PersistentMap) ~@rest)))

;; ─── cond — rete's OWN macro, expanding into rete's `if` ──────────────────
;;
;; BRIEF-rete-cond-is-its-own-macro.md (2026-08-05) — builder's ruling: "i think we need
;; rete's cond to just be a macro itself that expands into rete's if?". This REPLACES the
;; earlier attempt (a `freeze::env::build_env` loop that cloned core's registered `MacroDef`
;; and re-registered it under the rete name): a clone carries core's TEMPLATE, so it emitted
;; `:wat::core::if`/`:wat::core::cond` regardless of which name invoked it — a second door
;; that launders straight back through core's spelling (arc 179's `()`-vs-`nil` shape).
;;
;; This is instead its own `defmacro`, an exact copy of core's `cond` (wat/core.wat:1237)
;; with ONLY the emitted head keywords moved to the rete namespace: every backtick-quoted
;; `(:wat::core::if …)` becomes `(:wat::rete::core::if …)`, and every recursive
;; `(:wat::core::cond ~@…)` becomes `(:wat::rete::core::cond ~@…)` — including the
;; annotated-form branch's recursive call, which recurses WITHOUT emitting an `if` and is
;; therefore the spelling most easily missed by eye. The macro-error text on a
;; non-exhaustive clause list is kept byte-identical (still a located expansion error, same
;; first-class primitive). The `:else` structural comparison, the `List?` head test, and the
;; annotated-form strip are unchanged LOGIC — they call genuine core primitives
;; (empty?/List?/first/second/rest/let/=), not the emitted if/cond spelling, so they stay
;; `:wat::core::`.
;;
;; Rete `if` (`:wat::rete::core::if`, RETE_OPS `Form` row) re-dispatches at runtime to core
;; `if`'s genuine eval arm, so it fires inside a real `defrule` — proven by
;; `wat-scripts/scratch-pad/probe-rete-if-in-where.wat` (`hits=1`). Expanding into rete `if`
;; is therefore a correct target, not a hope.
;;
;; GROUNDED GAP, NOT fixed here: a `(:wat::rete::where …)` clause is never macro-expanded at
;; all (`defrule` quotes `:when`/`:then` verbatim; `matcher.rs`'s `eval_test_core` evaluates
;; that raw AST directly, never touching the macro registry) — see
;; `NOTE-a-where-body-is-never-macro-expanded.md` for the full grounding. This macro is the
;; correct, necessary prerequisite for whatever later change closes that gap; it does not
;; close it itself.
(:wat::core::defmacro :wat::rete::core::cond
  [& clauses <- :wat::core::Vector<wat::WatAST>]
  -> :wat::WatAST
  (:wat::core::if (:wat::core::empty? clauses)
    ;; empty clause list — non-exhaustive / no terminal :else. Same located diagnostic as
    ;; core's cond, byte-identical text.
    (:wat::core::macro-error "cond: non-exhaustive — needs a terminal :else arm")
    (:wat::core::if (:wat::core::List? (:wat::core::first clauses))
      ;; First clause is a List — bare form: (cond (test body) … (:else body))
      (:wat::core::let [arm  (:wat::core::first clauses)
                        head (:wat::core::first arm)]
        (:wat::core::if (:wat::core::List? head)
          ;; test arm — head is a sub-list like (= 1 2): (if head body (cond rest…)) —
          ;; rete-spelled all the way down.
          `(:wat::rete::core::if
              ~head
              ~(:wat::core::second arm)
              (:wat::rete::core::cond ~@(:wat::core::rest clauses)))
          ;; non-List head — detect :else by structural comparison with the :else keyword form.
          (:wat::core::if (:wat::core::= head (:wat::core::first `(:else)))
            ;; :else terminal arm — emit body unconditionally
            (:wat::core::second arm)
            ;; other non-List head — treat as test arm (v1 fallback for malformed input)
            `(:wat::rete::core::if
                ~head
                ~(:wat::core::second arm)
                (:wat::rete::core::cond ~@(:wat::core::rest clauses))))))
      ;; First clause is NOT a List (it is the -> symbol) — annotated form. Strip -> and :T
      ;; (first two elements) and re-expand as bare cond — the recursive spelling most easily
      ;; missed by eye, because it recurses WITHOUT emitting an if.
      `(:wat::rete::core::cond ~@(:wat::core::rest (:wat::core::rest clauses))))))

;; ─── make-rule + defrule ────────────────────────────────────────────────────

;; make-rule — runtime helper: split quoted vector nodes into PVs and build a Rule.
;; when-ast / then-ast are quoted VECTOR nodes; ast->children yields the per-element WatASTs.
;; Converts the std Vector from ast->children to a PersistentVector<WatAST> via foldl/conj.
(:wat::core::defn :wat::rete::make-rule
  [name     <- :wat::core::String
   when-ast <- :wat::WatAST
   then-ast <- :wat::WatAST]
  -> :wat::rete::Rule
  (:wat::core::let [lhs-pv (:wat::core::foldl
                               (:wat::core::fn [acc <- :wat::core::PersistentVector<wat::WatAST>
                                                c   <- :wat::WatAST]
                                 -> :wat::core::PersistentVector<wat::WatAST>
                                 (:wat::core::PersistentVector/conj acc c))
                               (:wat::core::PersistentVector)
                               (:wat::core::ast->children when-ast))
                    rhs-pv (:wat::core::foldl
                               (:wat::core::fn [acc <- :wat::core::PersistentVector<wat::WatAST>
                                                c   <- :wat::WatAST]
                                 -> :wat::core::PersistentVector<wat::WatAST>
                                 (:wat::core::PersistentVector/conj acc c))
                               (:wat::core::PersistentVector)
                               (:wat::core::ast->children then-ast))]
    (:wat::rete::Rule :name name :lhs lhs-pv :rhs rhs-pv)))

;; defrule — homoiconic rule macro: expand a readable rule form into a zero-arg defn
;; returning a Rule. The zero-arg fn is the reflection marker for collect-rules (stone 5b).
;;
;; Surface (arc 278 DESIGN-STONE-then-is-a-vector-of-singular-facts, Stone A):
;;   (:wat::rete::defrule :weather::cold-and-windy
;;     :when [<cond1> <cond2> …]
;;     :then [<fact1> <fact2> …])
;;
;; Both :when and :then are VECTORS. Each :then member is a single fact to insert — the
;; `:wat::rete::insert` RHS-marker wrapper is GONE (the engine is inserts-only by doctrine,
;; so naming it per entry said nothing; see matcher.rs's `build_insert_fact`).
;;
;; Expands to:
;;   (:wat::core::defn :weather::cold-and-windy [] -> :wat::rete::Rule
;;     (:wat::rete::make-rule "weather::cold-and-windy"
;;       (:wat::core::quote [<cond1> <cond2> …])
;;       (:wat::core::quote [<fact1> <fact2> …])))
;;
;; The macro is kept TRIVIAL: it quotes both vectors as-is — symmetric with when-vec.
;; make-rule (above) does the per-element split at runtime.
;; Assumes canonical :when then :then order (STOP if a general parse is needed).
(:wat::core::defmacro :wat::rete::defrule
  [name <- :wat::WatAST
   & rest <- :wat::core::Vector<wat::WatAST>]
  -> :wat::WatAST
  (:wat::core::let [;; name-str: ast-name returns the raw keyword text WITH leading colon;
                    ;; strip it to get the bare FQDN matching (:wat::core::type fact).
                    raw-name  (:wat::core::ast-name name)
                    ;; strip-leading-colon inline (can't call user-defn from program-body macro)
                    name-str  (:wat::core::if (:wat::core::= (:wat::core::string::subs raw-name 0 1) ":")
                                 (:wat::core::string::subs raw-name 1 (:wat::core::string::length raw-name))
                                 raw-name)
                    ;; rest = (:when <when-vec> :then <then-vec>); canonical order assumed.
                    when-vec  (:wat::core::Option/expect
                                 (:wat::core::get rest 1)
                                 "defrule: missing :when conditions vector")
                    then-vec  (:wat::core::Option/expect
                                 (:wat::core::get rest 3)
                                 "defrule: missing :then facts vector")]
    `(:wat::core::defn ~name [] -> :wat::rete::Rule
       (:wat::rete::make-rule ~name-str
         (:wat::core::quote ~when-vec)
         (:wat::core::quote ~then-vec)))))

;; make-query — split quoted :params / :when vectors into a Query.
(:wat::core::defn :wat::rete::make-query
  [name       <- :wat::core::String
   params-ast <- :wat::WatAST
   when-ast   <- :wat::WatAST]
  -> :wat::rete::Query
  (:wat::core::let [params-pv (:wat::core::foldl
                                 (:wat::core::fn [acc <- :wat::core::PersistentVector<wat::core::String>
                                                  p   <- :wat::WatAST]
                                   -> :wat::core::PersistentVector<wat::core::String>
                                   (:wat::core::PersistentVector/conj acc (:wat::core::ast-name p)))
                                 (:wat::core::PersistentVector)
                                 (:wat::core::ast->children params-ast))
                    lhs-pv (:wat::core::foldl
                              (:wat::core::fn [acc <- :wat::core::PersistentVector<wat::WatAST>
                                               c   <- :wat::WatAST]
                                -> :wat::core::PersistentVector<wat::WatAST>
                                (:wat::core::PersistentVector/conj acc c))
                              (:wat::core::PersistentVector)
                              (:wat::core::ast->children when-ast))]
    (:wat::rete::Query :name name :params params-pv :lhs lhs-pv)))

;; defquery — Clara `[defquery q [:?loc] …]`. Zero-arg defn returning Query.
;;   (:wat::rete::defquery :wq::temps-at
;;     :params [?loc]
;;     :when   […])
(:wat::core::defmacro :wat::rete::defquery
  [name <- :wat::WatAST
   & rest <- :wat::core::Vector<wat::WatAST>]
  -> :wat::WatAST
  (:wat::core::let [raw-name  (:wat::core::ast-name name)
                    name-str  (:wat::core::if (:wat::core::= (:wat::core::string::subs raw-name 0 1) ":")
                                 (:wat::core::string::subs raw-name 1 (:wat::core::string::length raw-name))
                                 raw-name)
                    params-vec (:wat::core::Option/expect
                                  (:wat::core::get rest 1)
                                  "defquery: missing :params vector")
                    when-vec   (:wat::core::Option/expect
                                  (:wat::core::get rest 3)
                                  "defquery: missing :when conditions vector")]
    `(:wat::core::defn ~name [] -> :wat::rete::Query
       (:wat::rete::make-query ~name-str
         (:wat::core::quote ~params-vec)
         (:wat::core::quote ~when-vec)))))

;; ─── acc:: — pure wat accumulator fold library (Stone 8-i) ─────────────────
;;
;; Each fn folds a PersistentVector<Element> into a reduced value.
;; An Element = (:wat::rete::Element fact bindings) where
;;   fact     = the original typed :wat::core::Record
;;   bindings = a PersistentMap<String,Value> of variable bindings.
;;
;; Value-folds read a bound ?var (a String key) from each element's bindings map:
;;   (:wat::core::Option/expect -> :wat::core::i64
;;     (:wat::core::PersistentMap/get (:wat::rete::Element/bindings e) var)
;;     "acc: var unbound")
;;
;; THE RETURN TYPE ENCODES THE EMPTY CASE (make illegal states unrepresentable):
;;   count / sum     → BARE value (0 on empty — always concrete; never Option)
;;   distinct / all  → BARE PV   ([] on empty)
;;   group-by        → BARE PM   ({} on empty)
;;   min / max / mean → Option   (None on empty — there is no minimum/maximum/mean of nothing)
;; Only the folds whose empty case has NO value are Option. (count's None can never happen.)
;;
;; mean = (/ sum count) — literal composition of the two sibling fns.
;; v1: numeric folds are i64; distinct element type is i64 (the probe stores i64 port/bytes).

;; acc::count — length els. ALWAYS concrete (length [] = 0) → bare i64, never Option.
(:wat::core::defn :wat::rete::acc::count
  [els <- :wat::core::PersistentVector<wat::rete::Element>]
  -> :wat::core::i64
  (:wat::core::length els))

;; acc::sum — Σ bindings[var]. Empty sum = 0 → bare i64, never Option.
(:wat::core::defn :wat::rete::acc::sum
  [var <- :wat::core::String
   els <- :wat::core::PersistentVector<wat::rete::Element>]
  -> :wat::core::i64
  (:wat::core::foldl
    (:wat::core::fn [acc <- :wat::core::i64
                     e   <- :wat::rete::Element]
      -> :wat::core::i64
      (:wat::core::+ acc
        (:wat::core::Option/expect  
          (:wat::core::PersistentMap/get (:wat::rete::Element/bindings e) var)
          "acc: var unbound")))
    0
    els))

;; acc::min — Some(min bindings[var]) via a < fold starting from None.
;; None seed: first element sets the initial value; subsequent elements narrow down.
;; empty → None.
(:wat::core::defn :wat::rete::acc::min
  [var <- :wat::core::String
   els <- :wat::core::PersistentVector<wat::rete::Element>]
  -> :wat::core::Option<wat::core::i64>
  (:wat::core::foldl
    (:wat::core::fn [acc <- :wat::core::Option<wat::core::i64>
                     e   <- :wat::rete::Element]
      -> :wat::core::Option<wat::core::i64>
      (:wat::core::let [v (:wat::core::Option/expect  
                             (:wat::core::PersistentMap/get (:wat::rete::Element/bindings e) var)
                             "acc: var unbound")]
        (:wat::core::match acc 
          ((:wat::core::Some cur)
           (:wat::core::Some (:wat::core::if (:wat::core::< v cur) v cur)))
          (:wat::core::None (:wat::core::Some v)))))
    :wat::core::None
    els))

;; acc::max — Some(max bindings[var]) via a > fold starting from None. empty → None.
(:wat::core::defn :wat::rete::acc::max
  [var <- :wat::core::String
   els <- :wat::core::PersistentVector<wat::rete::Element>]
  -> :wat::core::Option<wat::core::i64>
  (:wat::core::foldl
    (:wat::core::fn [acc <- :wat::core::Option<wat::core::i64>
                     e   <- :wat::rete::Element]
      -> :wat::core::Option<wat::core::i64>
      (:wat::core::let [v (:wat::core::Option/expect  
                             (:wat::core::PersistentMap/get (:wat::rete::Element/bindings e) var)
                             "acc: var unbound")]
        (:wat::core::match acc 
          ((:wat::core::Some cur)
           (:wat::core::Some (:wat::core::if (:wat::core::> v cur) v cur)))
          (:wat::core::None (:wat::core::Some v)))))
    :wat::core::None
    els))

;; acc::mean — COMPOSITION: (/ sum count). empty → None (count = 0 → no token).
;; Calls acc::sum and acc::count on the SAME element set — no re-fold; the ops are the oracle.
(:wat::core::defn :wat::rete::acc::mean
  [var <- :wat::core::String
   els <- :wat::core::PersistentVector<wat::rete::Element>]
  -> :wat::core::Option<wat::core::i64>
  ;; sum + count now return bare i64 (always concrete) — no Option/expect needed.
  (:wat::core::let [s (:wat::rete::acc::sum var els)
                    n (:wat::rete::acc::count els)]
    (:wat::core::if (:wat::core::= n 0)
      :wat::core::None
      (:wat::core::Some (:wat::core::/ s n)))))

;; acc::distinct — dedup bindings[var] via fold + contains?. empty → [] → bare PV, never Option.
;; v1: element type is i64 (the probe stores i64 port/bytes values).
(:wat::core::defn :wat::rete::acc::distinct
  [var <- :wat::core::String
   els <- :wat::core::PersistentVector<wat::rete::Element>]
  -> :wat::core::PersistentVector<wat::core::i64>
  (:wat::core::foldl
    (:wat::core::fn [acc <- :wat::core::PersistentVector<wat::core::i64>
                     e   <- :wat::rete::Element]
      -> :wat::core::PersistentVector<wat::core::i64>
      (:wat::core::let [v (:wat::core::Option/expect  
                             (:wat::core::PersistentMap/get (:wat::rete::Element/bindings e) var)
                             "acc: var unbound")]
        (:wat::core::if (:wat::core::PersistentVector/contains? acc v)
          acc
          (:wat::core::PersistentVector/conj acc v))))
    (:wat::core::PersistentVector)
    els))

;; acc::all — PV of each element's fact. empty → [] → bare PV, never Option.
(:wat::core::defn :wat::rete::acc::all
  [els <- :wat::core::PersistentVector<wat::rete::Element>]
  -> :wat::core::PersistentVector<wat::core::Record>
  (:wat::core::foldl
    (:wat::core::fn [acc <- :wat::core::PersistentVector<wat::core::Record>
                     e   <- :wat::rete::Element]
      -> :wat::core::PersistentVector<wat::core::Record>
      (:wat::core::PersistentVector/conj acc (:wat::rete::Element/fact e)))
    (:wat::core::PersistentVector)
    els))

;; acc::group-by — map bindings[var] → PV<fact> via foldl into a PersistentMap.
;; Each key is the bound var's value; each value is a PV of matching element facts.
;; empty → {} → bare PersistentMap, never Option.
(:wat::core::defn :wat::rete::acc::group-by
  [var <- :wat::core::String
   els <- :wat::core::PersistentVector<wat::rete::Element>]
  -> :wat::core::PersistentMap<wat::core::i64,wat::core::PersistentVector<wat::core::Record>>
  (:wat::core::foldl
    (:wat::core::fn [acc  <- :wat::core::PersistentMap<wat::core::i64,wat::core::PersistentVector<wat::core::Record>>
                     e    <- :wat::rete::Element]
      -> :wat::core::PersistentMap<wat::core::i64,wat::core::PersistentVector<wat::core::Record>>
      (:wat::core::let [k    (:wat::core::Option/expect  
                                (:wat::core::PersistentMap/get (:wat::rete::Element/bindings e) var)
                                "acc: var unbound")
                        fact (:wat::rete::Element/fact e)
                        pv   (:wat::core::match (:wat::core::PersistentMap/get acc k)
                               
                               ((:wat::core::Some existing) existing)
                               (:wat::core::None (:wat::core::PersistentVector)))]
        (:wat::core::PersistentMap/assoc acc k (:wat::core::PersistentVector/conj pv fact))))
    (:wat::core::PersistentMap)
    els))

;; acc::gather-vals (8-custom) — gather bindings[var] into a Vector<i64> in gather order
;; (NO dedup; the custom fold fn sees every value). A `Vector` (not PV) so it splices via
;; `~@` into the synthetic call AST (`unquote-splicing` flattens a Value::Vec element-wise).
;; This is the oracle mirror of the native `other` arm's PV gather.
(:wat::core::defn :wat::rete::acc::gather-vals
  [var <- :wat::core::String
   els <- :wat::core::PersistentVector<wat::rete::Element>]
  -> :wat::core::Vector<wat::core::i64>
  (:wat::core::foldl
    (:wat::core::fn [acc <- :wat::core::Vector<wat::core::i64>
                     e   <- :wat::rete::Element]
      -> :wat::core::Vector<wat::core::i64>
      (:wat::core::Vector/conj acc
        (:wat::core::Option/expect  
          (:wat::core::PersistentMap/get (:wat::rete::Element/bindings e) var)
          "acc: var unbound")))
    (:wat::core::Vector :wat::core::i64)
    els))

;; ─── the accumulate dispatch (Stone 8-a) ────────────────────────────────────
;;
;; WHY there is no single `apply-accumulator -> Option<Value>` fn: the wat type system has INVARIANT
;; parametric types — `Option<i64>` is NOT a subtype of `Option<Value>` even though i64 <: Value
;; (STONE-Value's `is_subtype` root rule fires only for `sup == ":wat::core::Value"` Path-to-Path, NOT
;; for `Option<T>` covariance). So the dispatch is inlined per-fold in accumulate-pass-for-token, where
;; each fold's concrete return type is handled directly: bare folds (count/sum/distinct/all/group-by)
;; assoc their result into the token's bindings; Option folds (min/max/mean) match inline (None → drop).

;; accumulate-pass-for-token — apply the acc-form over gathered elements for ONE token.
;; Returns the updated beta-mem: extends the token with result-var → aggregate if the
;; accumulator produces a value, or leaves beta-mem unchanged (drop) if it produces None
;; (empty min/max/mean).
;;
;; DESIGN: Each branch calls a specific acc::* fold and handles that fold's return type
;; directly. Bare folds (count/sum/distinct/all/group-by) always produce a value → assoc
;; into bindings. Option folds (min/max/mean) produce Option<i64> → match on that, then
;; assoc or drop. The PersistentMap/assoc on the BARE bindings PM accepts any value
;; (i64/PV/PM) via STONE-Value UP (i64 <: Value, PV <: Value, PM <: Value).
(:wat::core::defn :wat::rete::accumulate-pass-for-token
  [acc-form   <- :wat::WatAST
   gathered   <- :wat::core::PersistentVector<wat::rete::Element>
   result-var <- :wat::core::String
   tok        <- :wat::rete::Token
   node-id    <- :wat::core::i64
   bm         <- :wat::core::PersistentMap]
  -> :wat::core::PersistentMap
  (:wat::core::let [acc-ch (:wat::core::ast->children acc-form)
                    acc-hd (:wat::core::first acc-ch)
                    acc-nm (:wat::core::ast-name acc-hd)
                    ;; helper: extend tok's bindings with result-var → v, append to bm at node-id
                    ;; (inlined below per case to keep each branch's v-type concrete)
                    tok-binds (:wat::rete::Token/bindings tok)
                    tok-matches (:wat::rete::Token/matches tok)]
    (:wat::core::cond
      ;; count — bare i64 result (always); assoc directly
      ((:wat::core::= acc-nm ":wat::rete::acc::count")
       (:wat::core::let [v   (:wat::rete::acc::count gathered)
                         nb  (:wat::core::PersistentMap/assoc tok-binds result-var v)
                         ntk (:wat::rete::Token :matches tok-matches :bindings nb)]
         (:wat::rete::append-token bm node-id ntk)))
      ;; sum — bare i64 result (always); assoc directly
      ((:wat::core::= acc-nm ":wat::rete::acc::sum")
       (:wat::core::let [var (:wat::core::ast-name
                               (:wat::core::Option/expect  
                                 (:wat::core::get acc-ch 1)
                                 "accumulate-pass-for-token: sum missing ?var"))
                         v   (:wat::rete::acc::sum var gathered)
                         nb  (:wat::core::PersistentMap/assoc tok-binds result-var v)
                         ntk (:wat::rete::Token :matches tok-matches :bindings nb)]
         (:wat::rete::append-token bm node-id ntk)))
      ;; min — Option<i64>; Some → assoc, None → drop
      ((:wat::core::= acc-nm ":wat::rete::acc::min")
       (:wat::core::let [var (:wat::core::ast-name
                               (:wat::core::Option/expect  
                                 (:wat::core::get acc-ch 1)
                                 "accumulate-pass-for-token: min missing ?var"))]
         (:wat::core::match (:wat::rete::acc::min var gathered) 
           ((:wat::core::Some v)
            (:wat::rete::append-token bm node-id
              (:wat::rete::Token :matches tok-matches
                :bindings (:wat::core::PersistentMap/assoc tok-binds result-var v))))
           (:wat::core::None bm))))
      ;; max — Option<i64>; Some → assoc, None → drop
      ((:wat::core::= acc-nm ":wat::rete::acc::max")
       (:wat::core::let [var (:wat::core::ast-name
                               (:wat::core::Option/expect  
                                 (:wat::core::get acc-ch 1)
                                 "accumulate-pass-for-token: max missing ?var"))]
         (:wat::core::match (:wat::rete::acc::max var gathered) 
           ((:wat::core::Some v)
            (:wat::rete::append-token bm node-id
              (:wat::rete::Token :matches tok-matches
                :bindings (:wat::core::PersistentMap/assoc tok-binds result-var v))))
           (:wat::core::None bm))))
      ;; mean — Option<i64>; Some → assoc, None → drop
      ((:wat::core::= acc-nm ":wat::rete::acc::mean")
       (:wat::core::let [var (:wat::core::ast-name
                               (:wat::core::Option/expect  
                                 (:wat::core::get acc-ch 1)
                                 "accumulate-pass-for-token: mean missing ?var"))]
         (:wat::core::match (:wat::rete::acc::mean var gathered) 
           ((:wat::core::Some v)
            (:wat::rete::append-token bm node-id
              (:wat::rete::Token :matches tok-matches
                :bindings (:wat::core::PersistentMap/assoc tok-binds result-var v))))
           (:wat::core::None bm))))
      ;; distinct — bare PV result (always; empty → []); assoc directly
      ((:wat::core::= acc-nm ":wat::rete::acc::distinct")
       (:wat::core::let [var (:wat::core::ast-name
                               (:wat::core::Option/expect  
                                 (:wat::core::get acc-ch 1)
                                 "accumulate-pass-for-token: distinct missing ?var"))
                         v   (:wat::rete::acc::distinct var gathered)
                         nb  (:wat::core::PersistentMap/assoc tok-binds result-var v)
                         ntk (:wat::rete::Token :matches tok-matches :bindings nb)]
         (:wat::rete::append-token bm node-id ntk)))
      ;; all — bare PV<Record> result (always; empty → []); assoc directly
      ((:wat::core::= acc-nm ":wat::rete::acc::all")
       (:wat::core::let [v   (:wat::rete::acc::all gathered)
                         nb  (:wat::core::PersistentMap/assoc tok-binds result-var v)
                         ntk (:wat::rete::Token :matches tok-matches :bindings nb)]
         (:wat::rete::append-token bm node-id ntk)))
      ;; group-by — bare PM result (always; empty → {}); assoc directly
      ((:wat::core::= acc-nm ":wat::rete::acc::group-by")
       (:wat::core::let [var (:wat::core::ast-name
                               (:wat::core::Option/expect  
                                 (:wat::core::get acc-ch 1)
                                 "accumulate-pass-for-token: group-by missing ?var"))
                         v   (:wat::rete::acc::group-by var gathered)
                         nb  (:wat::core::PersistentMap/assoc tok-binds result-var v)
                         ntk (:wat::rete::Token :matches tok-matches :bindings nb)]
         (:wat::rete::append-token bm node-id ntk)))
      ;; 8-custom — a non-built-in head is a USER fold fn. Gather the ?var values into a
      ;; Vector<i64>, build the call `(user-fn (:wat::core::PersistentVector v0 v1 …))`
      ;; via quasiquote (~acc-hd splices the head; ~@vals splices the literal values into a
      ;; PV constructor), then eval-ast! it. The result (any Value) assocs into the binding.
      ;; The compile fence (compile-condition) has already proven the fn is pure∧det.
      (:else
       (:wat::core::let [var  (:wat::core::ast-name
                                (:wat::core::Option/expect  
                                  (:wat::core::get acc-ch 1)
                                  "accumulate-pass-for-token: custom fold missing ?var"))
                         vals (:wat::rete::acc::gather-vals var gathered)
                         call (:wat::core::quasiquote
                                ((:wat::core::unquote acc-hd)
                                 (:wat::core::PersistentVector
                                   (:wat::core::unquote-splicing vals))))
                         v    (:wat::core::Result/expect  
                                (:wat::eval-ast! call)
                                "accumulate-pass-for-token: custom fold eval failed")
                         nb   (:wat::core::PersistentMap/assoc tok-binds result-var v)
                         ntk  (:wat::rete::Token :matches tok-matches :bindings nb)]
         (:wat::rete::append-token bm node-id ntk))))))

;; acc-operand-keys — `?var` args of the acc-form (`max ?v` → [?v]; count → []).
(:wat::core::defn :wat::rete::acc-operand-keys
  [acc-form <- :wat::WatAST]
  -> :wat::core::PersistentVector<wat::core::String>
  (:wat::core::let [ch (:wat::core::ast->children acc-form)
                    n  (:wat::core::length ch)]
    (:wat::core::foldl
      (:wat::core::fn [acc <- :wat::core::PersistentVector<wat::core::String>
                       i   <- :wat::core::i64]
        -> :wat::core::PersistentVector<wat::core::String>
        (:wat::core::let [kid (:wat::core::Option/expect
                                (:wat::core::get ch i)
                                "acc-operand-keys")]
          (:wat::core::if (:wat::core::= (:wat::core::ast-kind kid) "symbol")
            (:wat::core::let [nm (:wat::core::ast-name kid)]
              (:wat::core::if (:wat::core::string::starts-with? nm "?")
                (:wat::core::PersistentVector/conj acc nm)
                acc))
            acc)))
      (:wat::core::PersistentVector)
      (:wat::core::range 1 n))))

;; keys-minus — `from` without any name in `drop`.
(:wat::core::defn :wat::rete::keys-minus
  [from <- :wat::core::PersistentVector<wat::core::String>
   drop <- :wat::core::PersistentVector<wat::core::String>]
  -> :wat::core::PersistentVector<wat::core::String>
  (:wat::core::foldl
    (:wat::core::fn [acc <- :wat::core::PersistentVector<wat::core::String>
                     k   <- :wat::core::String]
      -> :wat::core::PersistentVector<wat::core::String>
      (:wat::core::if (:wat::core::PersistentVector/contains? drop k)
        acc
        (:wat::core::PersistentVector/conj acc k)))
    (:wat::core::PersistentVector)
    from))

;; project-group-keys — element's bindings restricted to `keys` (the group key).
(:wat::core::defn :wat::rete::project-group-keys
  [el   <- :wat::rete::Element
   keys <- :wat::core::PersistentVector<wat::core::String>]
  -> :wat::core::PersistentMap
  (:wat::core::let [eb (:wat::rete::Element/bindings el)]
    (:wat::core::foldl
      (:wat::core::fn [acc <- :wat::core::PersistentMap
                       k   <- :wat::core::String]
        -> :wat::core::PersistentMap
        (:wat::core::match (:wat::core::PersistentMap/get eb k)
          ((:wat::core::Some v) (:wat::core::PersistentMap/assoc acc k v))
          (:wat::core::None acc)))
      (:wat::core::PersistentMap)
      keys)))

;; ─── accumulate-pass (Stone 8-a) ────────────────────────────────────────────
;;
;; Fold step: for each AccumulateNode, for each token in beta-memory[parent],
;; gather token-compatible elements from alpha-memory[from-alpha-id], group by
;; `:from` binds that are not already on the token and not acc-form operands
;; (Clara unbound grouping), call accumulate-pass-for-token per group, and
;; append the extended token (or drop it for empty min/max/mean).
;; Empty gather + non-empty group keys: no bag-wide 0 (Clara new-bindings).
;; Runs AFTER hash-join-pass and BEFORE filter-pass (so a :where on ?result sees the binding).
;;
;; STOP-3 resolution: apply-accumulator cannot return Option<Value> because the wat type system
;; has invariant parametric types — Option<i64> is not Option<Value>. The dispatch is inlined
;; in accumulate-pass-for-token where each fold's specific return type is handled directly.
(:wat::core::defn :wat::rete::accumulate-pass
  [network   <- :wat::core::PersistentMap
   alpha-mem <- :wat::core::PersistentMap
   beta-mem  <- :wat::core::PersistentMap
   node-id   <- :wat::core::i64]
  -> :wat::core::PersistentMap
  (:wat::core::let [node (:wat::core::Option/expect  
                             (:wat::core::PersistentMap/get network node-id)
                             "accumulate-pass: node not found")
                    kind (:wat::rete::node-kind-label node)]
    (:wat::core::if (:wat::core::= kind "AccumulateNode")
      (:wat::core::let [result-var    (:wat::rete::AccumulateNode/result-var    node)
                        acc-form      (:wat::rete::AccumulateNode/acc-form      node)
                        from-alpha-id (:wat::rete::AccumulateNode/from-alpha-id node)
                        tokens        (:wat::rete::tokens-or-empty-seed
                                        network beta-mem node-id)
                        from-els      (:wat::core::match
                                         (:wat::core::PersistentMap/get alpha-mem from-alpha-id)
                                         
                                       ((:wat::core::Some pv) pv)
                                       (:wat::core::None (:wat::core::PersistentVector)))
                        from-alpha    (:wat::core::Option/expect
                                         (:wat::core::PersistentMap/get network from-alpha-id)
                                         "accumulate-pass: from alpha missing")
                        from-cond     (:wat::core::Option/expect
                                         (:wat::core::get (:wat::rete::AlphaNode/tests from-alpha) 0)
                                         "accumulate-pass: from alpha has no cond")
                        from-keys     (:wat::rete::cond-bind-keys from-cond)
                        operand-keys  (:wat::rete::acc-operand-keys acc-form)]
        (:wat::core::foldl
          (:wat::core::fn [bm  <- :wat::core::PersistentMap
                           tok <- :wat::rete::Token]
            -> :wat::core::PersistentMap
            (:wat::core::let [gathered (:wat::core::foldl
                                          (:wat::core::fn [acc <- :wat::core::PersistentVector<wat::rete::Element>
                                                           el  <- :wat::rete::Element]
                                            -> :wat::core::PersistentVector<wat::rete::Element>
                                            (:wat::core::if (:wat::rete::token-element-compatible? tok el)
                                              (:wat::core::PersistentVector/conj acc el)
                                              acc))
                                          (:wat::core::PersistentVector)
                                          from-els)
                              tok-keys (:wat::core::foldl
                                          (:wat::core::fn [acc <- :wat::core::PersistentVector<wat::core::String>
                                                           k   <- :wat::core::String]
                                            -> :wat::core::PersistentVector<wat::core::String>
                                            (:wat::core::PersistentVector/conj acc k))
                                          (:wat::core::PersistentVector)
                                          (:wat::core::PersistentMap/keys
                                            (:wat::rete::Token/bindings tok)))
                              group-keys (:wat::rete::keys-minus
                                           (:wat::rete::keys-minus from-keys tok-keys)
                                           operand-keys)]
              (:wat::core::if (:wat::core::= (:wat::core::length group-keys) 0)
                (:wat::rete::accumulate-pass-for-token
                   acc-form gathered result-var tok node-id bm)
                (:wat::core::if (:wat::core::= (:wat::core::length gathered) 0)
                  bm
                  (:wat::core::let [key-maps
                                    (:wat::rete::distinct-maps
                                      (:wat::core::foldl
                                        (:wat::core::fn [acc <- :wat::core::PersistentVector<wat::core::PersistentMap>
                                                         el  <- :wat::rete::Element]
                                          -> :wat::core::PersistentVector<wat::core::PersistentMap>
                                          (:wat::core::PersistentVector/conj
                                            acc
                                            (:wat::rete::project-group-keys el group-keys)))
                                        (:wat::core::PersistentVector)
                                        gathered))]
                    (:wat::core::foldl
                      (:wat::core::fn [bm2 <- :wat::core::PersistentMap
                                       km  <- :wat::core::PersistentMap]
                        -> :wat::core::PersistentMap
                        (:wat::core::let [group-els
                                          (:wat::core::foldl
                                            (:wat::core::fn [acc <- :wat::core::PersistentVector<wat::rete::Element>
                                                             el  <- :wat::rete::Element]
                                              -> :wat::core::PersistentVector<wat::rete::Element>
                                              (:wat::core::if
                                                (:wat::core::PersistentVector/contains?
                                                  (:wat::core::PersistentVector/conj
                                                    (:wat::core::PersistentVector) km)
                                                  (:wat::rete::project-group-keys el group-keys))
                                                (:wat::core::PersistentVector/conj acc el)
                                                acc))
                                            (:wat::core::PersistentVector)
                                            gathered)
                                          km-keys (:wat::core::PersistentMap/keys km)
                                          ext-binds
                                          (:wat::core::foldl
                                            (:wat::core::fn [nb <- :wat::core::PersistentMap
                                                             k  <- :wat::core::String]
                                              -> :wat::core::PersistentMap
                                              (:wat::core::match (:wat::core::PersistentMap/get km k)
                                                ((:wat::core::Some v)
                                                 (:wat::core::PersistentMap/assoc nb k v))
                                                (:wat::core::None nb)))
                                            (:wat::rete::Token/bindings tok)
                                            km-keys)
                                          ext-tok
                                          (:wat::rete::Token
                                            :matches (:wat::rete::Token/matches tok)
                                            :bindings ext-binds)]
                          (:wat::rete::accumulate-pass-for-token
                             acc-form group-els result-var ext-tok node-id bm2)))
                      bm
                      key-maps))))))
          beta-mem
          tokens))
      beta-mem)))
