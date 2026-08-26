;; wat/rete.wat — the rete data model.
;;
;; Records (Token / Element / Session / node kinds / Export / explain
;; substrate) and render-dag. Compile lives in
;; wat/rete/compile.wat, acc::* in wat/rete/acc.wat, $oracle in
;; wat/rete/oracle/{insert,pass,accum-pass,fire,explain}.wat, query/defrule in wat/rete/syntax.wat. Native
;; fire is the unprimed public name (`fire-rules`); the wat reference is
;; `fire-rules$oracle`. VSA seam (`holon::cosine`/`dot`/`coincident?`/`presence?`)
;; fires on both mouths — see `probe_arc278_vsa_where_native_differential`.
;; Persistent collections throughout.
;; EDN-round-trippable.
;;
;; Names by the 3rd intueri cast (2026-06-17):
;;   NOT WorkingMemory → Session
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
  [matches  <- (:wat::core::PersistentVector :- [(:wat::core::Tuple :- [:wat::core::Record :wat::core::i64])])
   bindings <- :wat::core::PersistentMap])

;; Element — a fact presented to an alpha node; flows RIGHT into a join.
;; fact: the record fact itself (type-preserving; no conversion needed for provenance/TM/query).
;; bindings: alpha-bindings extracted by the alpha node's tests.
(:wat::core::defrecord :wat::rete::Element
  [fact     <- :wat::core::Record
   bindings <- :wat::core::PersistentMap])

;; ─── rules as data ──────────────────────────────────────────────────────────

;; Rule — a rule as pure data (not yet compiled into network nodes).
;; name: the namespaced rule name.
;; lhs:  rete `:when` conditions (`<-`, FQDN ops, `:wat::rete::and/or/not`) — (PersistentVector :- [WatAST]) so foldl works.
;; rhs:  consequence forms (data; pure — applied by a consumer).
(:wat::core::defrecord :wat::rete::Rule
  [name <- :wat::core::String
   lhs  <- (:wat::core::PersistentVector :- [:wat::WatAST])
   rhs  <- (:wat::core::PersistentVector :- [:wat::WatAST])])

;; Query — a named parametric query (Clara defquery). No :then; answers are
;; binding maps, filtered by param values at `query` time.
(:wat::core::defrecord :wat::rete::Query
  [name   <- :wat::core::String
   params <- (:wat::core::PersistentVector :- [:wat::core::String])
   lhs    <- (:wat::core::PersistentVector :- [:wat::WatAST])])

;; ─── the network nodes ──────────────────────────────────────────────────────
;; Alpha, RootJoin, HashJoin, Test, Negation, Exists, Accumulate, Production,
;; Query. ExpressionJoinNode stayed banked (`DESIGN-STONE-6b-where-test`).

;; AlphaNode — filters facts by structural tests; fans out to beta joins.
;; id:       unique node id (i64).
;; tests:    PersistentVector of rete alpha conditions — typed for foldl.
;; children: PersistentVector of child node ids — typed as i64 for foldl.
(:wat::core::defrecord :wat::rete::AlphaNode
  [id       <- :wat::core::i64
   tests    <- (:wat::core::PersistentVector :- [:wat::WatAST])
   children <- (:wat::core::PersistentVector :- [:wat::core::i64])])

;; RootJoinNode — the leftmost beta join (no left memory needed; seeds the token).
;; id:           unique node id.
;; children:     PersistentVector of child node ids — typed as i64 for foldl.
(:wat::core::defrecord :wat::rete::RootJoinNode
  [id       <- :wat::core::i64
   children <- (:wat::core::PersistentVector :- [:wat::core::i64])])

;; HashJoinNode — a standard two-input beta join node.
;; id:           unique node id.
;; children:     PersistentVector of child node ids — typed as i64 for foldl.
(:wat::core::defrecord :wat::rete::HashJoinNode
  [id       <- :wat::core::i64
   children <- (:wat::core::PersistentVector :- [:wat::core::i64])])

;; ProductionNode — the terminal node; triggers an activation on a full token.
;; id:        unique node id.
;; rule-name: the namespaced rule name whose RHS this node fires.
(:wat::core::defrecord :wat::rete::ProductionNode
  [id        <- :wat::core::i64
   rule-name <- :wat::core::String])

;; TestNode — a left-only filter node (stone 6b-ii-a): keeps a token iff eval-test(expr, bindings) is true.
;; id:       unique node id.
;; expr:     the pure∧det∧total∧rete WatAST predicate (stored as a value; four-axis fence at compile).
;; children: PersistentVector of child node ids (ProductionNode or further TestNodes).
(:wat::core::defrecord :wat::rete::TestNode
  [id       <- :wat::core::i64
   expr     <- :wat::WatAST
   children <- (:wat::core::PersistentVector :- [:wat::core::i64])])

;; NegationNode — a left-only filter node (stone 7-a): passes a token iff ZERO elements in the
;; negated alpha-memory are compatible with the token's bindings. Hash-join inverted: pure replay
;; dissolves the two-sided delta (the negated alpha-memory is fixed within a fire).
;; id:              unique node id.
;; negated-alpha-id: the AlphaNode id whose alpha-memory holds the facts to check absence against.
;; children:        PersistentVector of child node ids (ProductionNode or further filter nodes).
(:wat::core::defrecord :wat::rete::NegationNode
  [id              <- :wat::core::i64
   negated-alpha-id <- :wat::core::i64
   children        <- (:wat::core::PersistentVector :- [:wat::core::i64])])

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
   children        <- (:wat::core::PersistentVector :- [:wat::core::i64])])

;; AccumulateNode — a left-input aggregate join node (stone 8-a): for each parent token,
;; gathers the token-compatible elements from from-alpha-id's alpha-memory, folds them with
;; accumulate-pass-for-token (compiled AccFold over the 8-i acc::* folds), and extends the
;; token with result-var → aggregate.
;; Pure replay: re-accumulates on every fire (no retract-fn needed).
;; id:            unique node id.
;; result-var:    the ?var name stored WITH the "?" prefix (`head-nm` as bound).
;; acc-form:      the accumulator form (WatAST), e.g. (:wat::rete::acc::count) or (:wat::rete::acc::sum ?v).
;; from-alpha-id: the AlphaNode id whose alpha-memory holds the :from facts.
;; children:      PersistentVector of child node ids (ProductionNode, TestNode, NegationNode, etc.).
(:wat::core::defrecord :wat::rete::AccumulateNode
  [id            <- :wat::core::i64
   result-var    <- :wat::core::String
   acc-form      <- :wat::WatAST
   from-alpha-id <- :wat::core::i64
   children      <- (:wat::core::PersistentVector :- [:wat::core::i64])])

;; QueryNode — a named query endpoint; like a production but returns answers.
;; id:         unique node id.
;; query-name: the namespaced query name.
;; param-keys: PersistentVector of query parameter variable names (Strings).
(:wat::core::defrecord :wat::rete::QueryNode
  [id         <- :wat::core::i64
   query-name <- :wat::core::String
   param-keys <- (:wat::core::PersistentVector :- [:wat::core::String])])

;; ─── the session (the whole engine state) ───────────────────────────────────
;; intueri: NOT WorkingMemory — Session names the whole caller-facing engine state.

(:wat::core::typealias :wat::rete::AlphaMemory
  (:wat::core::PersistentMap :- [:wat::core::i64 (:wat::core::PersistentVector :- [:wat::rete::Element])]))
(:wat::core::typealias :wat::rete::BetaMemory
  (:wat::core::PersistentMap :- [:wat::core::i64 (:wat::core::PersistentVector :- [:wat::rete::Token])]))
(:wat::core::typealias :wat::rete::ProductionMemory
  (:wat::core::PersistentMap :- [:wat::core::i64 (:wat::core::PersistentVector :- [:wat::core::Record])]))

;; Overlay — the thing with-overlay hands its body: facts in, a FIRED Session out. The Session
;; is a fact overlay over circuits it does not own (arm.rs:572) and is immutable, so re-seeding
;; from the compiled base is the only "reset" there is. Named by the 2026-08-24 intueri cast
;; (rete-scoped-work-naming.wat.intueri) — "overlay" is this corpus's own word (arm.rs:572,
;; session.rs:1114, kernel/tests.rs:3068), not imported vocabulary.
(:wat::core::typealias :wat::rete::Overlay
  [(:wat::core::PersistentVector :- [:wat::core::Record]) :-> :wat::rete::Session])

;; Session — the complete rete engine state; the caller-facing handle.
;;   network:           id → raw node record — the compiled DAG, id-indexed.
;;   rules:             PersistentVector of Rule (the rule-set as data).
;;   alpha-memory:      PersistentMap — flat `node-id → (PV :- [Element])` (AlphaMemory is the
;;                      walker view; freeze fields stay unparameterized PersistentMap).
;;                      FIRE-SCOPED scratch, rebuilt from `facts` on every fire.
;;                      `fire-once` populates it; `fire-rules` returns it empty.
;;   beta-memory:       PersistentMap — flat `node-id → (PV :- [Token])` (BetaMemory walker view).
;;   production-memory: PersistentMap — flat `node-id → (PV :- [Record])` of derived facts
;;                      (ProductionMemory walker view). Support lives on Explained.
;;   facts:             PersistentVector of asserted facts.
;;   next-id:           the next free node id (i64).
;;   query-memory:      query-name → PV of binding maps (QueryNode answers; survives fire).
(:wat::core::defrecord :wat::rete::Session
  [network           <- :wat::core::PersistentMap
   rules             <- (:wat::core::PersistentVector :- [:wat::rete::Rule])
   alpha-memory      <- :wat::core::PersistentMap
   beta-memory       <- :wat::core::PersistentMap
   production-memory <- :wat::core::PersistentMap
   facts             <- :wat::core::PersistentVector
   next-id           <- :wat::core::i64
   query-memory      <- :wat::core::PersistentMap])

(:wat::core::typealias :wat::rete::GroupByMap
  (:wat::core::PersistentMap :- [:wat::core::i64 (:wat::core::PersistentVector :- [:wat::core::Record])]))
(:wat::core::typealias :wat::rete::ClassFields
  (:wat::core::PersistentVector :- [(:wat::core::PersistentVector :- [:wat::core::String])]))

;; Export — the compiled program as one EDN value. Not a Session.
;; No facts, no memories, no source forms. Native fire only.
;;   v:       format version (1).
;;   abi:     TypeEnv field-order + RETE_OPS fingerprint. Import refuses a miss.
;;   classes: interned fact-class FQDNs (colon-free).
;;   fields:  per-class declared field names (parallel to classes).
;;   nodes:   packed topology (kind, id, edges). No WatAST.
;;   conds / drivers / progs / folds / rhs: packed circuits.
;;   deps:   [name [produced…] [negated…] [consumed…] [exists-and-from…]] — stratify schedule.
;;           Residual, packed from interned arm.rule_deps (not Session.rules AST).
;;           Import without deps refuses production fire — empty residual would
;;           lie about negation-over-derived (not max_s=0).
(:wat::core::defrecord :wat::rete::Export
  [v       <- :wat::core::i64
   abi     <- :wat::core::String
   classes <- (:wat::core::PersistentVector :- [:wat::core::String])
   fields  <- :wat::rete::ClassFields
   nodes   <- :wat::core::PersistentVector
   conds   <- :wat::core::PersistentVector
   drivers <- :wat::core::PersistentVector
   progs   <- :wat::core::PersistentVector
   folds   <- :wat::core::PersistentVector
   rhs     <- :wat::core::PersistentVector
   deps    <- :wat::core::PersistentVector])

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
;;   support: (PersistentMap :- [derived-fact Support]) — the provenance index.
;; EPHEMERAL — re-derived per explain; never serialized.
(:wat::core::defrecord :wat::rete::Explained
  [session <- :wat::rete::Session
   support <- :wat::core::PersistentMap])

;; ─── P12b+P12c: derivation-tree records + explain walk ─────────────────────

;; DerivationNode — one node in the provenance tree. P12c: adds rule (Option :- [String])
;; and changes via to (PV :- [DerivationStep]) (the edge payload from P12c).
;;   fact: the derived (or base) fact this node represents.
;;   rule: Some(rule-name) for a derived fact; None for a base/asserted leaf.
;;   via:  the supporting edges — one DerivationStep per supporting fact.
;;         Empty (length 0) ⟺ base/asserted fact (the leaf).
;;         Non-empty ⟺ derived fact (each step explains one supporting input).
;; EPHEMERAL — produced by explain; never serialized.
(:wat::core::defrecord :wat::rete::DerivationNode
  [fact <- :wat::core::Record
   rule <- (:wat::core::Option :- [:wat::core::String])
   via  <- (:wat::core::PersistentVector :- [:wat::rete::DerivationStep])])

;; DerivationStep — one edge in the provenance tree. Carries the payload that
;; makes the derivation readable without knowing the rule.
;;   supporting:  the supporting fact's own DerivationNode (recurse; leaf = empty via).
;;   pattern:     the matched condition's fact-type FQDN (e.g. "weather::Temperature").
;;   bindings:    per-step bound vars: only the variables this condition bound.
;;   constraints: the rule's satisfied predicates with bound values substituted.
;;                Rendered as WatAST, e.g. (:wat::i64::< -5 0) from
;;                (:wat::i64::< ?c 0) with ?c=-5.
;; EPHEMERAL — produced by explain; never serialized.
(:wat::core::defrecord :wat::rete::DerivationStep
  [supporting  <- :wat::rete::DerivationNode
   pattern     <- :wat::core::String
   bindings    <- (:wat::core::PersistentMap :- [:wat::core::String :wat::core::Value])
   constraints <- (:wat::core::PersistentVector :- [:wat::WatAST])])

;; step-payload is a rust primitive (`:wat::rete::step-payload`). explain calls it.

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
       ;; matches is (PersistentVector :- [(wat::core::Tuple :- [wat::core::Record wat::core::i64])]); each tuple is (sfact, alpha-id).
       (:wat::core::let [tok      (:wat::rete::Support/token sv)
                         matches  (:wat::rete::Token/matches tok)
                         bindings (:wat::rete::Token/bindings tok)
                         rule     (:wat::rete::Support/rule sv)
                         session  (:wat::rete::Explained/session ex)
                         ;; Arc 118.2a — `map` flipped LAZY; `DerivationNode`'s 3rd field is
                         ;; `(PersistentVector :- [DerivationStep])`, so materialize via `into`.
                         via      (:wat::core::into (:wat::core::PersistentVector)
                                    (:wat::core::map
                                      (:wat::core::fn [m <- (:wat::core::Tuple :- [:wat::core::Record :wat::core::i64])]
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
                    parts  (:wat::string::split fqdn "::")
                    n      (:wat::core::length parts)]
    (:wat::core::if (:wat::i64::> n 0)
      (:wat::core::Option/expect  
        (:wat::core::get parts (:wat::i64::- n 1))
        "node-kind-label: last segment")
      fqdn)))

;; node-children-ids — read the children PersistentVector from a raw node record.
;; Dispatches on kind label: Alpha/RootJoin/HashJoin/Test/Negation/Exists/Accumulate
;; have children; Production/Query return empty.
;; WHY: record accessors are class-guarded at runtime; dispatch ensures we only call
;; AlphaNode/children when the node IS an AlphaNode, satisfying the guard.
(:wat::core::defn :wat::rete::node-children-ids
  [node <- :wat::core::Record]
  -> (:wat::core::PersistentVector :- [:wat::core::i64])
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

;; children-ids-text — format a (PersistentVector :- [i64]) as "[id id ...]" for render-dag.
;; WHY: foldl builds space-separated ids so render-dag can emit the edge list inline.
(:wat::core::defn :wat::rete::children-ids-text
  [ids <- (:wat::core::PersistentVector :- [:wat::core::i64])]
  -> :wat::core::String
  (:wat::core::let [inner (:wat::core::foldl
                             (:wat::core::fn [acc <- :wat::core::String
                                              id  <- :wat::core::i64]
                               -> :wat::core::String
                               (:wat::core::let [id-s (:wat::i64::to-string id)]
                                 (:wat::core::if (:wat::core::= acc "")
                                   id-s
                                   (:wat::string::interpolate "{acc} {id-s}" :acc acc :id-s id-s))))
                             ""
                             ids)]
    (:wat::string::interpolate "[{inner}]" :inner inner)))

;; render-dag — walk Session.network (id→Node records), emit one readable line
;; per node: "  <id>  <kind> -> [<child-ids>]\n". Returns the whole graph as a String.
;;
;; Strategy: get keys from the PersistentMap as a (Vec :- [i64]), foldl over them,
;; for each key fetch the node (Option/expect), derive the kind label, emit edges.
;; Uses PersistentMap/keys (returns (Vec :- [K])) + foldl + PersistentMap/get.
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
                          id-s  (:wat::i64::to-string k)
                          edge  (:wat::rete::children-ids-text
                                   (:wat::rete::node-children-ids node))
                          ;; rune:exigere(scope-affirmative) — arc 278 proof-by-diff fixture:
                          ;; nested string::concat is left intentionally. The arc-277 auto-fix
                          ;; is bare-symbol-only and cannot reach this compound case.
                          ;; Do NOT hand-fix.
                          line  (:wat::string::concat
                                   "  "
                                   (:wat::string::concat
                                     id-s
                                     (:wat::string::concat
                                       "  "
                                       (:wat::string::concat
                                         kind
                                         (:wat::string::concat
                                           " -> "
                                           (:wat::string::concat edge "\n"))))))]
          (:wat::string::concat acc line)))
      ""
      keys)))
