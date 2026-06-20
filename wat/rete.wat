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
;; Load order: after :wat::Record::def (uses the Record macro). Record.wat's
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
(:wat::Record::def :wat::rete::Token
  [matches  <- :wat::core::PersistentVector<(wat::Record,wat::core::i64)>
   bindings <- :wat::core::PersistentMap])

;; Element — a fact presented to an alpha node; flows RIGHT into a join.
;; fact: the record fact itself (type-preserving; no conversion needed for provenance/TM/query-by-type).
;; bindings: alpha-bindings extracted by the alpha node's tests.
(:wat::Record::def :wat::rete::Element
  [fact     <- :wat::Record
   bindings <- :wat::core::PersistentMap])

;; Activation — a ProductionNode queued to fire.
;; production-id: the id of the ProductionNode to fire (intueri: NOT node-id).
;; token: the matching Token that triggered this activation.
(:wat::Record::def :wat::rete::Activation
  [production-id <- :wat::core::i64
   token         <- :wat::rete::Token])

;; ─── rules as data ──────────────────────────────────────────────────────────

;; Rule — a rule as pure data (not yet compiled into network nodes).
;; name: the namespaced rule name.
;; lhs:  conditions (form::matches?-shaped clauses) — PersistentVector<WatAST> so foldl works.
;; rhs:  consequence forms (data; pure — applied by a consumer).
(:wat::Record::def :wat::rete::Rule
  [name <- :wat::core::String
   lhs  <- :wat::core::PersistentVector<wat::WatAST>
   rhs  <- :wat::core::PersistentVector<wat::WatAST>])

;; ─── the network nodes (MVP set) ────────────────────────────────────────────
;; Negation / Test / Accumulate / ExpressionJoin nodes arrive at stones 6–8.

;; AlphaNode — filters facts by structural tests; fans out to beta joins.
;; id:       unique node id (i64).
;; tests:    PersistentVector of test forms (form::matches? clauses) — typed for foldl.
;; children: PersistentVector of child node ids — typed as i64 for foldl.
(:wat::Record::def :wat::rete::AlphaNode
  [id       <- :wat::core::i64
   tests    <- :wat::core::PersistentVector<wat::WatAST>
   children <- :wat::core::PersistentVector<wat::core::i64>])

;; RootJoinNode — the leftmost beta join (no left memory needed; seeds the token).
;; id:           unique node id.
;; children:     PersistentVector of child node ids — typed as i64 for foldl.
;; binding-keys: PersistentVector of variable keys (Strings) bound at this join.
(:wat::Record::def :wat::rete::RootJoinNode
  [id           <- :wat::core::i64
   children     <- :wat::core::PersistentVector<wat::core::i64>
   binding-keys <- :wat::core::PersistentVector<wat::core::String>])

;; HashJoinNode — a standard two-input beta join node.
;; id:           unique node id.
;; children:     PersistentVector of child node ids — typed as i64 for foldl.
;; binding-keys: PersistentVector of join-key variable names (Strings).
(:wat::Record::def :wat::rete::HashJoinNode
  [id           <- :wat::core::i64
   children     <- :wat::core::PersistentVector<wat::core::i64>
   binding-keys <- :wat::core::PersistentVector<wat::core::String>])

;; ProductionNode — the terminal node; triggers an activation on a full token.
;; id:        unique node id.
;; rule-name: the namespaced rule name whose RHS this node fires.
(:wat::Record::def :wat::rete::ProductionNode
  [id        <- :wat::core::i64
   rule-name <- :wat::core::String])

;; TestNode — a left-only filter node (stone 6b-ii-a): keeps a token iff eval-test(expr, bindings) is true.
;; id:       unique node id.
;; expr:     the pure∧deterministic WatAST predicate (stored as a value; fence checked at compile).
;; children: PersistentVector of child node ids (ProductionNode or further TestNodes).
(:wat::Record::def :wat::rete::TestNode
  [id       <- :wat::core::i64
   expr     <- :wat::WatAST
   children <- :wat::core::PersistentVector<wat::core::i64>])

;; QueryNode — a named query endpoint; like a production but returns answers.
;; id:         unique node id.
;; query-name: the namespaced query name.
;; param-keys: PersistentVector of query parameter variable names (Strings).
(:wat::Record::def :wat::rete::QueryNode
  [id         <- :wat::core::i64
   query-name <- :wat::core::String
   param-keys <- :wat::core::PersistentVector<wat::core::String>])

;; Node — the sum type over all MVP node records (exact defenum syntax per wat/service.wat).
;; Variants wrap their respective record. Used by compile + fire (stones 1b+);
;; the Session.network stores raw node records in v1 (the probe hand-builds with raw records).
(:wat::core::defenum :wat::rete::Node
  :AlphaNode      [node <- :wat::rete::AlphaNode]
  :RootJoinNode   [node <- :wat::rete::RootJoinNode]
  :HashJoinNode   [node <- :wat::rete::HashJoinNode]
  :ProductionNode [node <- :wat::rete::ProductionNode]
  :TestNode       [node <- :wat::rete::TestNode]
  :QueryNode      [node <- :wat::rete::QueryNode])

;; ─── the session (the whole engine state) ───────────────────────────────────
;; intueri: NOT WorkingMemory — Session names the whole caller-facing engine state.

;; Session — the complete rete engine state; the caller-facing handle.
;;   network:           id → Node (raw node records) — the compiled DAG, id-indexed.
;;   rules:             PersistentVector of Rule (the rule-set as data).
;;   alpha-memory:      node-id → {join-bindings → [Element …]}
;;   beta-memory:       node-id → {join-bindings → [Token …]}
;;   production-memory: node-id → PV<:wat::Record>  flat derived facts in 4a; grows to the {token → [facts]} support store in 4c (TM)
;;   facts:             PersistentVector of asserted facts.
;;   next-id:           the next free node id (i64).
(:wat::Record::def :wat::rete::Session
  [network           <- :wat::core::PersistentMap
   rules             <- :wat::core::PersistentVector<wat::rete::Rule>
   alpha-memory      <- :wat::core::PersistentMap
   beta-memory       <- :wat::core::PersistentMap
   production-memory <- :wat::core::PersistentMap
   facts             <- :wat::core::PersistentVector
   next-id           <- :wat::core::i64])

;; ─── P12a: explain substrate ────────────────────────────────────────────────

;; Support — the producing support record for one derived fact.
;;   rule:  the rule name that derived the fact (for Why.rule in P12b).
;;   token: the producing Token; token.matches = the support chain (for :via in P12b).
;; EPHEMERAL — carried only in Explained; never serialized / from-edn.
(:wat::Record::def :wat::rete::Support
  [rule  <- :wat::core::String
   token <- :wat::rete::Token])

;; Explained — the opt-in diagnostic result of fire-rules-explain.
;;   session: the same frozen Session the fast path produces (same closure, same derived facts).
;;   support: PersistentMap<derived-fact, Support> — the provenance index.
;; EPHEMERAL — re-derived per explain; never serialized.
(:wat::Record::def :wat::rete::Explained
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
(:wat::Record::def :wat::rete::DerivationNode
  [fact <- :wat::Record
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
(:wat::Record::def :wat::rete::DerivationStep
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
   sfact          <- :wat::Record
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
   fact <- :wat::Record]
  -> :wat::rete::DerivationNode
  (:wat::core::let [support (:wat::rete::Explained/support ex)
                    sv-opt  (:wat::core::PersistentMap/get support fact)]
    (:wat::core::match sv-opt -> :wat::rete::DerivationNode
      ((:wat::core::Some sv)
       ;; derived fact — recurse on each supporting fact in the token's matches chain.
       ;; matches is PersistentVector<(wat::Record, wat::core::i64)>; each tuple is (sfact, alpha-id).
       (:wat::core::let [tok      (:wat::rete::Support/token sv)
                         matches  (:wat::rete::Token/matches tok)
                         bindings (:wat::rete::Token/bindings tok)
                         rule     (:wat::rete::Support/rule sv)
                         session  (:wat::rete::Explained/session ex)
                         via      (:wat::core::map
                                    (:wat::core::fn [m <- :(wat::Record,wat::core::i64)]
                                      -> :wat::rete::DerivationStep
                                      (:wat::core::let [sfact    (:wat::core::first m)
                                                        alpha-id (:wat::core::second m)]
                                        (:wat::rete::step-payload session alpha-id bindings sfact
                                          (:wat::rete::explain ex sfact))))
                                    matches)]
         (:wat::rete::DerivationNode fact (:wat::core::Some rule) via)))
      (:wat::core::None
       ;; base/asserted fact — leaf node, rule=None, via is empty.
       (:wat::rete::DerivationNode fact :wat::core::None (:wat::core::PersistentVector))))))

;; ─── render-dag ─────────────────────────────────────────────────────────────

;; node-kind-label — derive a short readable label from a raw node record's
;; declared type FQDN. Returns the last segment (e.g. "RootJoinNode").
;; (:wat::core::type node) returns the class FQDN without leading colon,
;; e.g. "wat::rete::RootJoinNode". We take the text after the last "::".
(:wat::core::defn :wat::rete::node-kind-label
  [node <- :wat::Record]
  -> :wat::core::String
  (:wat::core::let [fqdn   (:wat::core::type node)
                    parts  (:wat::core::string::split fqdn "::")
                    n      (:wat::core::length parts)]
    (:wat::core::if (:wat::core::i64::> n 0)
      (:wat::core::Option/expect -> :wat::core::String
        (:wat::core::get parts (:wat::core::i64::- n 1))
        "node-kind-label: last segment")
      fqdn)))

;; node-children-ids — read the children PersistentVector from a raw node record.
;; Dispatches on kind label: Alpha/RootJoin/HashJoin have children; leaves return empty.
;; WHY: record accessors are class-guarded at runtime; dispatch ensures we only call
;; AlphaNode/children when the node IS an AlphaNode, satisfying the guard.
(:wat::core::defn :wat::rete::node-children-ids
  [node <- :wat::Record]
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
        (:wat::core::let [node  (:wat::core::Option/expect -> :wat::Record
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
(:wat::Record::def :wat::rete::CompileState
  [network <- :wat::core::PersistentMap
   next-id <- :wat::core::i64
   dedup   <- :wat::core::HashMap<wat::core::String,wat::core::i64>])

;; MintResult — result of find-or-mint: the resolved node id + updated state.
;; WHY a record: named fields communicate intent at call sites better than positional.
(:wat::Record::def :wat::rete::MintResult
  [id    <- :wat::core::i64
   state <- :wat::rete::CompileState])

;; network-add-child — add child-id to the children of the node at node-id in network.
;; Returns the updated PersistentMap.
;; WHY: wiring edges = conj child-id onto the existing children PersistentVector and
;; re-assoc the node; :wat::Record/assoc does name-based field update on any Record.
(:wat::core::defn :wat::rete::network-add-child
  [network  <- :wat::core::PersistentMap
   node-id  <- :wat::core::i64
   child-id <- :wat::core::i64]
  -> :wat::core::PersistentMap
  (:wat::core::let [node     (:wat::core::Option/expect -> :wat::Record
                                  (:wat::core::PersistentMap/get network node-id)
                                  "network-add-child: node not found")
                    old-ch   (:wat::rete::node-children-ids node)
                    new-ch   (:wat::core::PersistentVector/conj old-ch child-id)
                    new-node (:wat::Record/assoc node :children new-ch)]
    (:wat::core::PersistentMap/assoc network node-id new-node)))

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
    (:wat::core::match found-opt -> :wat::rete::MintResult
      ((:wat::core::Some existing-id)
       (:wat::rete::MintResult existing-id state))
      (:wat::core::None
       (:wat::core::let [alpha     (:wat::rete::AlphaNode
                                      next-id
                                      (:wat::core::PersistentVector cond)
                                      (:wat::core::PersistentVector))
                         new-net   (:wat::core::PersistentMap/assoc network next-id alpha)
                         new-dedup (:wat::core::HashMap/assoc dedup dkey next-id)
                         new-state (:wat::rete::CompileState
                                      new-net
                                      (:wat::core::i64::+ next-id 1)
                                      new-dedup)]
         (:wat::rete::MintResult next-id new-state))))))

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
    (:wat::core::match found-opt -> :wat::rete::MintResult
      ((:wat::core::Some existing-id)
       (:wat::rete::MintResult existing-id state))
      (:wat::core::None
       (:wat::core::let [join-node (:wat::rete::RootJoinNode
                                      next-id
                                      (:wat::core::PersistentVector)
                                      (:wat::core::PersistentVector))
                         new-net   (:wat::core::PersistentMap/assoc network next-id join-node)
                         new-dedup (:wat::core::HashMap/assoc dedup dkey next-id)
                         new-state (:wat::rete::CompileState
                                      new-net
                                      (:wat::core::i64::+ next-id 1)
                                      new-dedup)]
         (:wat::rete::MintResult next-id new-state))))))

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
    (:wat::core::match found-opt -> :wat::rete::MintResult
      ((:wat::core::Some existing-id)
       (:wat::rete::MintResult existing-id state))
      (:wat::core::None
       (:wat::core::let [join-node (:wat::rete::HashJoinNode
                                      next-id
                                      (:wat::core::PersistentVector)
                                      (:wat::core::PersistentVector))
                         new-net   (:wat::core::PersistentMap/assoc network next-id join-node)
                         new-dedup (:wat::core::HashMap/assoc dedup dkey next-id)
                         new-state (:wat::rete::CompileState
                                      new-net
                                      (:wat::core::i64::+ next-id 1)
                                      new-dedup)]
         (:wat::rete::MintResult next-id new-state))))))

;; compile-condition — fold step: process one condition form in a rule.
;; acc = (CompileState, i64) where the i64 is the current parent-id (-1 = no parent yet).
;; WHY -1 sentinel: lets us distinguish first-condition (RootJoinNode) from rest
;; (HashJoinNode) without an Option; node ids start at 0.
;; Algorithm per DESIGN-1b (and 6b-ii-a extension):
;;   TOP branch — if cond is (:wat::rete::where <expr>): fence pure∧det, mint TestNode,
;;     wire parent→test (parent must exist: a where is never the first condition).
;;   ELSE branch — existing alpha+join path:
;;   1. find-or-mint AlphaNode for cond (alpha sharing)
;;   2. find-or-mint RootJoinNode or HashJoinNode for (cond, parent-id) (beta-prefix sharing)
;;   3. wire alpha→join child edge
;;   4. wire prev-parent→join child edge (if prev-parent >= 0)
;;   5. return updated state with parent-id = join-id
(:wat::Record::def :wat::rete::CondFoldAcc
  [state     <- :wat::rete::CompileState
   parent-id <- :wat::core::i64])

(:wat::core::defn :wat::rete::compile-condition
  [acc  <- :wat::rete::CondFoldAcc
   cond <- :wat::WatAST]
  -> :wat::rete::CondFoldAcc
  (:wat::core::let [state0    (:wat::rete::CondFoldAcc/state     acc)
                    parent-id (:wat::rete::CondFoldAcc/parent-id acc)
                    ;; TOP: detect (:wat::rete::where <expr>) form
                    ;; All conditions are non-empty list forms with a keyword head; Option/expect is safe.
                    cond-ch   (:wat::core::ast->children cond)
                    head      (:wat::core::Option/expect -> :wat::WatAST
                                  (:wat::core::first cond-ch)
                                  "compile-condition: condition form has no head")
                    head-nm   (:wat::core::ast-name head)
                    is-where  (:wat::core::= head-nm ":wat::rete::where")]
    (:wat::core::if is-where
      ;; ── where branch (6b-ii-a) ──────────────────────────────────────────────
      (:wat::core::let [expr      (:wat::core::Option/expect -> :wat::WatAST
                                      (:wat::core::second cond-ch)
                                      "compile-condition: where missing expr")
                        ;; fence: pure ∧ deterministic — raise at compile if false
                        is-pure   (:wat::rete::pure? expr)
                        is-det    (:wat::rete::deterministic? expr)
                        _fence    (:wat::core::Option/expect -> :wat::core::nil
                                      (:wat::core::if (:wat::core::and is-pure is-det)
                                        (:wat::core::Some nil)
                                        (:wat::core::None))
                                      "compile-condition: where expr must be pure and deterministic")
                        ;; mint the TestNode
                        network0  (:wat::rete::CompileState/network state0)
                        next-id0  (:wat::rete::CompileState/next-id state0)
                        dedup0    (:wat::rete::CompileState/dedup   state0)
                        test-node (:wat::rete::TestNode next-id0 expr (:wat::core::PersistentVector))
                        net1      (:wat::core::PersistentMap/assoc network0 next-id0 test-node)
                        state1    (:wat::rete::CompileState
                                     net1
                                     (:wat::core::i64::+ next-id0 1)
                                     dedup0)
                        ;; wire parent → test (a where always has a prior join parent)
                        net2      (:wat::core::if (:wat::core::i64::>= parent-id 0)
                                     (:wat::rete::network-add-child
                                        (:wat::rete::CompileState/network state1)
                                        parent-id
                                        next-id0)
                                     (:wat::rete::CompileState/network state1))
                        state2    (:wat::rete::CompileState
                                     net2
                                     (:wat::rete::CompileState/next-id state1)
                                     (:wat::rete::CompileState/dedup   state1))]
        (:wat::rete::CondFoldAcc state2 next-id0))
      ;; ── alpha+join branch (existing) ────────────────────────────────────────
      (:wat::core::let [;; 1. find-or-mint the AlphaNode
                        alpha-res  (:wat::rete::find-or-mint-alpha cond state0)
                        alpha-id   (:wat::rete::MintResult/id    alpha-res)
                        state1     (:wat::rete::MintResult/state alpha-res)
                        ;; 2. find-or-mint the join node; -1 parent = first condition → RootJoinNode
                        is-first  (:wat::core::i64::< parent-id 0)
                        join-res  (:wat::core::if is-first
                                     (:wat::rete::find-or-mint-root-join cond state1)
                                     (:wat::rete::find-or-mint-hash-join cond parent-id state1))
                        join-id    (:wat::rete::MintResult/id    join-res)
                        state2     (:wat::rete::MintResult/state join-res)
                        ;; 3. wire alpha → join
                        net3       (:wat::rete::network-add-child
                                      (:wat::rete::CompileState/network state2)
                                      alpha-id
                                      join-id)
                        state3     (:wat::rete::CompileState
                                      net3
                                      (:wat::rete::CompileState/next-id state2)
                                      (:wat::rete::CompileState/dedup   state2))
                        ;; 4. wire prev-parent → join (only if there IS a prev parent)
                        net4       (:wat::core::if (:wat::core::i64::>= parent-id 0)
                                      (:wat::rete::network-add-child
                                         (:wat::rete::CompileState/network state3)
                                         parent-id
                                         join-id)
                                      (:wat::rete::CompileState/network state3))
                        state4     (:wat::rete::CompileState
                                      net4
                                      (:wat::rete::CompileState/next-id state3)
                                      (:wat::rete::CompileState/dedup   state3))]
        ;; 5. advance parent to join-id for the next condition
        (:wat::rete::CondFoldAcc state4 join-id)))))

;; compile-rule — fold step: process one Rule into the network.
;; WHY: folds over the rule's lhs conditions with compile-condition, then mints
;; the ProductionNode as a child of the final join (the "leaf" terminal).
(:wat::core::defn :wat::rete::compile-rule
  [state <- :wat::rete::CompileState
   rule  <- :wat::rete::Rule]
  -> :wat::rete::CompileState
  (:wat::core::let [lhs       (:wat::rete::Rule/lhs rule)
                    rname     (:wat::rete::Rule/name rule)
                    ;; fold conditions left→right; parent-id starts at -1 (none)
                    init-acc  (:wat::rete::CondFoldAcc state -1)
                    final-acc (:wat::core::foldl
                                  :wat::rete::compile-condition
                                  init-acc
                                  lhs)
                    state2    (:wat::rete::CondFoldAcc/state     final-acc)
                    final-par (:wat::rete::CondFoldAcc/parent-id final-acc)
                    ;; mint the ProductionNode (never shared — one per rule)
                    network2  (:wat::rete::CompileState/network state2)
                    next-id2  (:wat::rete::CompileState/next-id state2)
                    prod      (:wat::rete::ProductionNode next-id2 rname)
                    net3      (:wat::core::PersistentMap/assoc network2 next-id2 prod)
                    ;; wire final-join → production
                    net4      (:wat::core::if (:wat::core::i64::>= final-par 0)
                                  (:wat::rete::network-add-child net3 final-par next-id2)
                                  net3)
                    dedup3    (:wat::rete::CompileState/dedup state2)]
    (:wat::rete::CompileState net4 (:wat::core::i64::+ next-id2 1) dedup3)))

;; compile — turn a PersistentVector of Rules into a Session with the compiled network.
;; The session constructor for arc 278: (compile rules) → insert → fire → query.
;; Empty memories and facts; next-id reflects the actual count of minted nodes.
(:wat::core::defn :wat::rete::compile
  [rules <- :wat::core::PersistentVector<wat::rete::Rule>]
  -> :wat::rete::Session
  (:wat::core::let [init-state (:wat::rete::CompileState
                                  (:wat::core::PersistentMap)
                                  0
                                  (:wat::core::HashMap :wat::core::String :wat::core::i64))
                    final-state (:wat::core::foldl
                                    :wat::rete::compile-rule
                                    init-state
                                    rules)
                    network  (:wat::rete::CompileState/network final-state)
                    next-id  (:wat::rete::CompileState/next-id final-state)
                    empty-pm (:wat::core::PersistentMap)
                    empty-pv (:wat::core::PersistentVector)]
    (:wat::rete::Session
       network
       rules
       empty-pm
       empty-pm
       empty-pm
       empty-pv
       next-id)))

;; ─── insert + fire-rules ────────────────────────────────────────────────────────

;; insert — stage a fact into the session's working memory. Zero activation.
;; WHY zero activation: the WM stays open while the caller stages multiple facts;
;; fire-rules is the lock that runs them through the network all at once.
;; WHY reconstruct Session: Record/assoc returns the base :wat::Record type; the
;; typed Session constructor preserves the concrete return type for the checker.
(:wat::core::defn :wat::rete::insert
  [session <- :wat::rete::Session
   fact    <- :wat::Record]
  -> :wat::rete::Session
  (:wat::rete::Session
    (:wat::rete::Session/network           session)
    (:wat::rete::Session/rules             session)
    (:wat::rete::Session/alpha-memory      session)
    (:wat::rete::Session/beta-memory       session)
    (:wat::rete::Session/production-memory session)
    (:wat::core::PersistentVector/conj (:wat::rete::Session/facts session) fact)
    (:wat::rete::Session/next-id           session)))

;; activate-fact — fold step: try one fact against a single AlphaNode's condition.
;; On a match, appends an Element(fact, bindings) to alpha-memory at alpha-id;
;; on no match, returns alpha-memory unchanged.
;; WHY two assoc branches: avoids a nested intermediate PV match (the empty PV
;; would be typed PersistentVector<?> and conflict with the Some-arm's PV type).
(:wat::core::defn :wat::rete::activate-fact
  [alpha-id  <- :wat::core::i64
   cond      <- :wat::WatAST
   alpha-mem <- :wat::core::PersistentMap
   fact      <- :wat::Record]
  -> :wat::core::PersistentMap
  (:wat::core::let [match-result (:wat::rete::alpha-match cond fact)]
    (:wat::core::match match-result -> :wat::core::PersistentMap
      ((:wat::core::Some bindings)
       ;; WHY staged-fact = Element(record, bindings): stores the original typed record
       ;; (not a map) so downstream queries + TM provenance can use the fact type directly.
       (:wat::core::let [staged-fact (:wat::rete::Element fact bindings)]
         (:wat::core::match (:wat::core::PersistentMap/get alpha-mem alpha-id) -> :wat::core::PersistentMap
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
  (:wat::core::let [node (:wat::core::Option/expect -> :wat::Record
                             (:wat::core::PersistentMap/get network node-id)
                             "activate-alpha: node not found")
                    kind (:wat::rete::node-kind-label node)]
    (:wat::core::cond
      ((:wat::core::= kind "AlphaNode")
       ;; WHY get tests[0]: AlphaNode.tests is a PV; the first element is the single
       ;; condition form (WatAST) compiled from the rule's LHS clause.
       (:wat::core::let [cond (:wat::core::Option/expect -> :wat::WatAST
                                  (:wat::core::get (:wat::rete::AlphaNode/tests node) 0)
                                  "activate-alpha: AlphaNode has no tests")]
         (:wat::core::foldl
           (:wat::core::fn [acc  <- :wat::core::PersistentMap
                            fact <- :wat::Record]
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
    (:wat::core::PersistentVector
      (:wat::core::Tuple (:wat::rete::Element/fact el) alpha-id))
    (:wat::rete::Element/bindings el)))

;; append-token — append a Token to beta-memory at root-join-id; create the PV if absent.
;; WHY two assoc branches: same rationale as activate-fact — avoids a nested intermediate
;; PV match where the empty branch would have an under-typed PersistentVector<?>.
(:wat::core::defn :wat::rete::append-token
  [beta-mem     <- :wat::core::PersistentMap
   root-join-id <- :wat::core::i64
   tok          <- :wat::rete::Token]
  -> :wat::core::PersistentMap
  (:wat::core::match (:wat::core::PersistentMap/get beta-mem root-join-id) -> :wat::core::PersistentMap
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
  (:wat::core::let [alpha-node (:wat::core::Option/expect -> :wat::Record
                                   (:wat::core::PersistentMap/get network alpha-id)
                                   "seed-root-join-children: alpha node not found")
                    child-ids  (:wat::rete::AlphaNode/children alpha-node)]
    (:wat::core::foldl
      (:wat::core::fn [bm       <- :wat::core::PersistentMap
                       child-id <- :wat::core::i64]
        -> :wat::core::PersistentMap
        (:wat::core::let [child-node (:wat::core::Option/expect -> :wat::Record
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
  (:wat::core::let [node (:wat::core::Option/expect -> :wat::Record
                             (:wat::core::PersistentMap/get network node-id)
                             "root-join-pass: node not found")]
    (:wat::core::cond
      ((:wat::core::= (:wat::rete::node-kind-label node) "AlphaNode")
       (:wat::core::match (:wat::core::PersistentMap/get alpha-mem node-id) -> :wat::core::PersistentMap
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
        (:wat::core::let [node (:wat::core::Option/expect -> :wat::Record
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
                                                        -> :wat::core::PersistentMap
                                       ((:wat::core::Some v)
                                        (:wat::core::PersistentMap/assoc bm k v))
                                       (:wat::core::None bm)))
                                   (:wat::rete::Token/bindings tok)
                                   (:wat::core::PersistentMap/keys e-binds))]
    (:wat::rete::Token new-matches new-binds)))

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
;; For each node-id: if it is a RootJoinNode or HashJoinNode with tokens in beta-memory,
;; for each HashJoinNode child J, cross beta-memory[here] × alpha-memory[alpha-feeding(J)].
;; WHY topological pass in node-id order: compile assigns IDs left-to-right (root-join < hash-join),
;; so iterating ascending node-ids processes parents before children — one pass is a valid fixpoint
;; for the v1 linear-chain topology. No cycle possible (DAG; monotone insertions only).
(:wat::core::defn :wat::rete::hash-join-pass
  [alpha-mem <- :wat::core::PersistentMap
   network   <- :wat::core::PersistentMap
   beta-mem  <- :wat::core::PersistentMap
   node-id   <- :wat::core::i64]
  -> :wat::core::PersistentMap
  (:wat::core::let [node (:wat::core::Option/expect -> :wat::Record
                             (:wat::core::PersistentMap/get network node-id)
                             "hash-join-pass: node not found")
                    kind (:wat::rete::node-kind-label node)]
    (:wat::core::if (:wat::core::or (:wat::core::= kind "RootJoinNode")
                                    (:wat::core::= kind "HashJoinNode"))
      (:wat::core::match (:wat::core::PersistentMap/get beta-mem node-id) -> :wat::core::PersistentMap
        ((:wat::core::Some tokens)
         (:wat::core::foldl
           (:wat::core::fn [bm       <- :wat::core::PersistentMap
                            child-id <- :wat::core::i64]
             -> :wat::core::PersistentMap
             (:wat::core::let [child (:wat::core::Option/expect -> :wat::Record
                                         (:wat::core::PersistentMap/get network child-id)
                                         "hash-join-pass: child not found")]
               (:wat::core::if (:wat::core::= (:wat::rete::node-kind-label child) "HashJoinNode")
                 ;; WHY match on alpha-mem: no elements on the right → no matches possible;
                 ;; skip the cross to avoid building an empty PV (avoids the untyped-PV hazard).
                 (:wat::core::let [aid (:wat::rete::alpha-feeding child-id network)]
                   (:wat::core::match (:wat::core::PersistentMap/get alpha-mem aid)
                                      -> :wat::core::PersistentMap
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
        (:wat::core::let [node (:wat::core::Option/expect -> :wat::Record
                                   (:wat::core::PersistentMap/get network node-id)
                                   "node-parent: node not found")]
          (:wat::core::if (:wat::core::PersistentVector/contains?
                             (:wat::rete::node-children-ids node)
                             child-id)
            node-id
            -1))))
    -1
    (:wat::core::PersistentMap/keys network)))

;; rule-by-name — linear find: given a rule name String, return the matching Rule from rules PV.
;; WHY foldl carrying Option: PersistentVector has no early-exit find; foldl short-circuits by
;; passing found values through unchanged (match Some → pass; None → test name; conj on hit).
;; The caller panics on None (a missing rule = a compile bug).
(:wat::core::defn :wat::rete::rule-by-name
  [rules <- :wat::core::PersistentVector<wat::rete::Rule>
   rname <- :wat::core::String]
  -> :wat::rete::Rule
  (:wat::core::Option/expect -> :wat::rete::Rule
    (:wat::core::foldl
      (:wat::core::fn [found <- :wat::core::Option<wat::rete::Rule>
                       rule  <- :wat::rete::Rule]
        -> :wat::core::Option<wat::rete::Rule>
        (:wat::core::match found -> :wat::core::Option<wat::rete::Rule>
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
  (:wat::core::let [prod-node  (:wat::core::Option/expect -> :wat::Record
                                   (:wat::core::PersistentMap/get network prod-id)
                                   "fire-production: prod node not found")
                    rname      (:wat::rete::ProductionNode/rule-name prod-node)
                    parent-id  (:wat::rete::node-parent prod-id network)
                    rule       (:wat::rete::rule-by-name rules rname)
                    rhs        (:wat::rete::Rule/rhs rule)]
    (:wat::core::match (:wat::core::PersistentMap/get beta-mem parent-id) -> :wat::core::PersistentMap
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
                 (:wat::core::match (:wat::core::PersistentMap/get pm2 prod-id) -> :wat::core::PersistentMap
                   ((:wat::core::Some pv)
                    (:wat::core::PersistentMap/assoc pm2 prod-id
                      (:wat::core::PersistentVector/conj pv derived)))
                   (:wat::core::None
                    (:wat::core::PersistentMap/assoc pm2 prod-id
                      (:wat::core::PersistentVector/conj (:wat::core::PersistentVector) derived))))))
             pm
             rhs))
         prod-mem
         tokens))
      (:wat::core::None prod-mem))))

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
  (:wat::core::let [node (:wat::core::Option/expect -> :wat::Record
                             (:wat::core::PersistentMap/get network node-id)
                             "test-pass: node not found")]
    (:wat::core::if (:wat::core::= (:wat::rete::node-kind-label node) "TestNode")
      (:wat::core::let [expr      (:wat::rete::TestNode/expr node)
                        parent-id (:wat::rete::node-parent node-id network)]
        (:wat::core::match (:wat::core::PersistentMap/get beta-mem parent-id) -> :wat::core::PersistentMap
          ((:wat::core::Some tokens)
           (:wat::core::foldl
             (:wat::core::fn [bm  <- :wat::core::PersistentMap
                              tok <- :wat::rete::Token]
               -> :wat::core::PersistentMap
               (:wat::core::if (:wat::rete::eval-test expr (:wat::rete::Token/bindings tok))
                 (:wat::rete::append-token bm node-id tok)
                 bm))
             beta-mem
             tokens))
          (:wat::core::None beta-mem)))
      beta-mem)))

;; production-pass — fold step: if this node is a ProductionNode, fire it; else pass through.
;; Mirrors hash-join-pass as a fold step over node-ids; seeds with the existing production-memory.
(:wat::core::defn :wat::rete::production-pass
  [network  <- :wat::core::PersistentMap
   beta-mem <- :wat::core::PersistentMap
   rules    <- :wat::core::PersistentVector<wat::rete::Rule>
   prod-mem <- :wat::core::PersistentMap
   node-id  <- :wat::core::i64]
  -> :wat::core::PersistentMap
  (:wat::core::let [node (:wat::core::Option/expect -> :wat::Record
                             (:wat::core::PersistentMap/get network node-id)
                             "production-pass: node not found")]
    (:wat::core::if (:wat::core::= (:wat::rete::node-kind-label node) "ProductionNode")
      (:wat::rete::fire-production node-id network beta-mem rules prod-mem)
      prod-mem)))

;; fire-once — single-pass fire cycle: alpha → root-join → hash-join → production.
;; Pure value-semantics: takes a Session, returns a new frozen Session with fresh memories.
;; Recomputes all memories from Session.facts each call (re-run-from-scratch); derived facts
;; go to production-memory only — they do not re-enter facts here (cascade is fire-rules' job).
;; WHY reconstruct Session: same reason as insert (Record/assoc returns :wat::Record).
(:wat::core::defn :wat::rete::fire-once
  [session <- :wat::rete::Session]
  -> :wat::rete::Session
  (:wat::core::let [network  (:wat::rete::Session/network session)
                    rules    (:wat::rete::Session/rules   session)
                    facts    (:wat::rete::Session/facts   session)
                    node-ids (:wat::core::PersistentMap/keys network)
                    ;; alpha pass (2b) — populate alpha-memory
                    new-amem (:wat::core::foldl
                                (:wat::core::fn [acc     <- :wat::core::PersistentMap
                                                 node-id <- :wat::core::i64]
                                  -> :wat::core::PersistentMap
                                  (:wat::rete::activate-alpha facts network acc node-id))
                                (:wat::core::PersistentMap)
                                node-ids)
                    ;; root-join pass (3a) — seed RootJoinNode beta-memory from alpha Elements
                    new-bmem (:wat::core::foldl
                                (:wat::core::fn [acc     <- :wat::core::PersistentMap
                                                 node-id <- :wat::core::i64]
                                  -> :wat::core::PersistentMap
                                  (:wat::rete::root-join-pass new-amem network acc node-id))
                                (:wat::core::PersistentMap)
                                node-ids)
                    ;; hash-join pass (3b) — propagate tokens LEFT→RIGHT through HashJoinNodes
                    ;; WHY one ordered pass: compile assigns IDs in topological order (see hash-join-pass)
                    joined-bmem (:wat::core::foldl
                                   (:wat::core::fn [acc     <- :wat::core::PersistentMap
                                                    node-id <- :wat::core::i64]
                                     -> :wat::core::PersistentMap
                                     (:wat::rete::hash-join-pass new-amem network acc node-id))
                                   new-bmem
                                   node-ids)
                    ;; test pass (6b-ii-a) — filter tokens through TestNodes (where conditions)
                    ;; WHY after joins: tests filter already-joined tokens; join IDs < test IDs by construction.
                    tested-bmem (:wat::core::foldl
                                   (:wat::core::fn [acc     <- :wat::core::PersistentMap
                                                    node-id <- :wat::core::i64]
                                     -> :wat::core::PersistentMap
                                     (:wat::rete::test-pass network acc node-id))
                                   joined-bmem
                                   node-ids)
                    ;; production pass (4a) — fire each ProductionNode's RHS into production-memory
                    new-pmem (:wat::core::foldl
                                (:wat::core::fn [acc     <- :wat::core::PersistentMap
                                                 node-id <- :wat::core::i64]
                                  -> :wat::core::PersistentMap
                                  (:wat::rete::production-pass network tested-bmem rules acc node-id))
                                (:wat::core::PersistentMap)
                                node-ids)]
    (:wat::rete::Session
      (:wat::rete::Session/network session)
      (:wat::rete::Session/rules   session)
      new-amem
      tested-bmem
      new-pmem
      facts
      (:wat::rete::Session/next-id session))))

;; collect-derived — flatten production-memory's per-node PV<Record> values into one PV<:wat::Record>.
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
                         f <- :wat::Record]
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
                     f   <- :wat::Record]
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
          (:wat::rete::Session/network fired)
          (:wat::rete::Session/rules   fired)
          (:wat::rete::Session/alpha-memory fired)
          (:wat::rete::Session/beta-memory  fired)
          (:wat::rete::Session/production-memory fired)
          new-facts
          (:wat::rete::Session/next-id fired))))))

;; fire-rules-spec — the wat reference engine (the SPEC / differential oracle). Run fire-fixpoint, then
;; restore Session.facts = input only. Re-run-from-scratch each call: simple and obviously correct, so it
;; is the reference the fast native kernel (`fire-rules'`) is differential-tested against. This is NOT the
;; production verb — the public `fire-rules` (below) dispatches to the native kernel; `fire-rules-spec` is
;; kept as the executable specification (arc 278 P5 close: wat = spec, Rust = impl).
;; WHY the facts=input split: fire-fixpoint accumulates derived facts into Session.facts across rounds so
;; cascades match (input ∪ derived visible to each round); but the RETURNED Session.facts must hold ONLY the
;; asserted/input facts (the retractable base). This is the fact-model fix for 4c TM: with facts = input
;; only, retract-then-fire recomputes the closure from a smaller input → consequences vanish transitively.
(:wat::core::defn :wat::rete::fire-rules-spec
  [session <- :wat::rete::Session]
  -> :wat::rete::Session
  (:wat::core::let [input (:wat::rete::Session/facts session)
                    fired (:wat::rete::fire-fixpoint session)]
    (:wat::rete::Session
      (:wat::rete::Session/network           fired)
      (:wat::rete::Session/rules             fired)
      (:wat::rete::Session/alpha-memory      fired)
      (:wat::rete::Session/beta-memory       fired)
      (:wat::rete::Session/production-memory fired)
      input
      (:wat::rete::Session/next-id           fired))))

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
   fact    <- :wat::Record]
  -> :wat::rete::Session
  (:wat::core::let [old-facts (:wat::rete::Session/facts session)
                    new-facts (:wat::core::foldl
                                 (:wat::core::fn [acc <- :wat::core::PersistentVector<wat::Record>
                                                  f   <- :wat::Record]
                                   -> :wat::core::PersistentVector<wat::Record>
                                   (:wat::core::if (:wat::core::not (:wat::core::= f fact))
                                     (:wat::core::PersistentVector/conj acc f)
                                     acc))
                                 (:wat::core::PersistentVector)
                                 old-facts)]
    (:wat::rete::Session
      (:wat::rete::Session/network           session)
      (:wat::rete::Session/rules             session)
      (:wat::rete::Session/alpha-memory      session)
      (:wat::rete::Session/beta-memory       session)
      (:wat::rete::Session/production-memory session)
      new-facts
      (:wat::rete::Session/next-id           session))))

;; ─── query — read derived facts of a type from a fired session ──────────────

;; query-by-type-string — runtime helper: filter production-memory by a colon-free type FQDN.
;; Flattens all production-node PVs into one PV, then filters by (:wat::core::type f) == ty-str.
;; Private; called by the `query` macro which resolves the type FQDN string at expand time.
;; Returns an empty PV if the type was never derived — never raises.
(:wat::core::defn :wat::rete::query-by-type-string
  [session <- :wat::rete::Session
   ty-str  <- :wat::core::String]
  -> :wat::core::PersistentVector
  (:wat::core::let [all (:wat::core::foldl
                           (:wat::core::fn [acc <- :wat::core::PersistentVector
                                            pv  <- :wat::core::PersistentVector]
                             -> :wat::core::PersistentVector
                             (:wat::core::foldl
                               (:wat::core::fn [a <- :wat::core::PersistentVector
                                                f <- :wat::Record]
                                 -> :wat::core::PersistentVector
                                 (:wat::core::PersistentVector/conj a f))
                               acc
                               pv))
                           (:wat::core::PersistentVector)
                           (:wat::core::PersistentMap/values
                             (:wat::rete::Session/production-memory session)))]
    (:wat::core::filter
      (:wat::core::fn [f <- :wat::Record] -> :wat::core::bool
        (:wat::core::= (:wat::core::type f) ty-str))
      all)))

;; query — runtime fn: read derived facts of a type from a fired session.
;; ty is the type's Record constructor fn: in the types-as-forms surface a bare type name
;; (:weather::ColdAndWindy) evaluates to that type's CONSTRUCTOR (a fn VALUE, not a keyword —
;; defined names resolve to their bindings). return-type-of (arc 278 intrinsic) reads the
;; constructor's declared return type (= the record type) as a colon-free FQDN string, directly
;; comparable to (:wat::core::type fact). Returns an empty PV if the type was never derived — never raises.
(:wat::core::defn :wat::rete::query
  [session <- :wat::rete::Session
   ty      <- :wat::core::fn]
  -> :wat::core::PersistentVector
  (:wat::rete::query-by-type-string session (:wat::runtime::return-type-of ty)))

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
    (:wat::rete::Rule name lhs-pv rhs-pv)))

;; defrule — homoiconic rule macro: expand a readable rule form into a zero-arg defn
;; returning a Rule. The zero-arg fn is the reflection marker for collect-rules (stone 5b).
;;
;; Surface:
;;   (:wat::rete::defrule :weather::cold-and-windy
;;     :when [<cond1> <cond2> …]
;;     :then <insert1> <insert2> …)
;;
;; Expands to:
;;   (:wat::core::defn :weather::cold-and-windy [] -> :wat::rete::Rule
;;     (:wat::rete::make-rule "weather::cold-and-windy"
;;       (:wat::core::quote [<cond1> <cond2> …])
;;       (:wat::core::quote [<insert1> <insert2> …])))
;;
;; The macro is kept TRIVIAL: it quotes the whole :when vector and splices all :then
;; forms into a vector literal. make-rule (above) does the per-element split at runtime.
;; Assumes canonical :when then :then order (STOP if a general parse is needed).
(:wat::core::defmacro :wat::rete::defrule
  [name <- :wat::WatAST
   & rest <- :wat::core::Vector<wat::WatAST>]
  -> :wat::WatAST
  (:wat::core::let [;; name-str: ast-name returns the raw keyword text WITH leading colon;
                    ;; strip it to get the bare FQDN matching (:wat::core::type fact).
                    raw-name  (:wat::core::ast-name name)
                    name-str  (:wat::core::if (:wat::core::= (:wat::core::string::subs raw-name 0 1) ":")
                                 (:wat::core::string::subs raw-name 1 (:wat::core::string::length raw-name))
                                 raw-name)
                    ;; rest = (:when <when-vec> :then <insert-form> …); canonical order assumed.
                    when-vec  (:wat::core::Option/expect -> :wat::WatAST
                                 (:wat::core::get rest 1)
                                 "defrule: missing :when conditions vector")
                    then-forms (:wat::core::drop rest 3)]
    `(:wat::core::defn ~name [] -> :wat::rete::Rule
       (:wat::rete::make-rule ~name-str
         (:wat::core::quote ~when-vec)
         (:wat::core::quote [~@then-forms])))))
