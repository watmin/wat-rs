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
;; matches: [[fact node-id] …] — the support chain.
;; bindings: {?var → value} — variable bindings accumulated left-to-right.
(:wat::Record::def :wat::rete::Token
  [matches  <- :wat::core::PersistentVector
   bindings <- :wat::core::PersistentMap])

;; Element — a fact presented to an alpha node; flows RIGHT into a join.
;; fact: a fact represented as a field→value PersistentMap (v1 record-as-map).
;; bindings: alpha-bindings extracted by the alpha node's tests.
(:wat::Record::def :wat::rete::Element
  [fact     <- :wat::core::PersistentMap
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
;; lhs:  conditions (form::matches?-shaped clauses).
;; rhs:  consequence forms (data; pure — applied by a consumer).
(:wat::Record::def :wat::rete::Rule
  [name <- :wat::core::String
   lhs  <- :wat::core::PersistentVector
   rhs  <- :wat::core::PersistentVector])

;; ─── the network nodes (MVP set) ────────────────────────────────────────────
;; Negation / Test / Accumulate / ExpressionJoin nodes arrive at stones 6–8.

;; AlphaNode — filters facts by structural tests; fans out to beta joins.
;; id:       unique node id (i64).
;; tests:    PersistentVector of test forms (form::matches? clauses).
;; children: PersistentVector of child node ids (i64).
(:wat::Record::def :wat::rete::AlphaNode
  [id       <- :wat::core::i64
   tests    <- :wat::core::PersistentVector
   children <- :wat::core::PersistentVector])

;; RootJoinNode — the leftmost beta join (no left memory needed; seeds the token).
;; id:           unique node id.
;; children:     PersistentVector of child node ids.
;; binding-keys: PersistentVector of variable keys bound at this join.
(:wat::Record::def :wat::rete::RootJoinNode
  [id           <- :wat::core::i64
   children     <- :wat::core::PersistentVector
   binding-keys <- :wat::core::PersistentVector])

;; HashJoinNode — a standard two-input beta join node.
;; id:           unique node id.
;; children:     PersistentVector of child node ids.
;; binding-keys: PersistentVector of join-key variable names.
(:wat::Record::def :wat::rete::HashJoinNode
  [id           <- :wat::core::i64
   children     <- :wat::core::PersistentVector
   binding-keys <- :wat::core::PersistentVector])

;; ProductionNode — the terminal node; triggers an activation on a full token.
;; id:        unique node id.
;; rule-name: the namespaced rule name whose RHS this node fires.
(:wat::Record::def :wat::rete::ProductionNode
  [id        <- :wat::core::i64
   rule-name <- :wat::core::String])

;; QueryNode — a named query endpoint; like a production but returns answers.
;; id:         unique node id.
;; query-name: the namespaced query name.
;; param-keys: PersistentVector of query parameter variable names.
(:wat::Record::def :wat::rete::QueryNode
  [id         <- :wat::core::i64
   query-name <- :wat::core::String
   param-keys <- :wat::core::PersistentVector])

;; Node — the sum type over all MVP node records (exact defenum syntax per wat/service.wat).
;; Variants wrap their respective record. Used by compile + fire (stones 1b+);
;; the Session.network stores raw node records in v1 (the probe hand-builds with raw records).
(:wat::core::defenum :wat::rete::Node
  :AlphaNode      [node <- :wat::rete::AlphaNode]
  :RootJoinNode   [node <- :wat::rete::RootJoinNode]
  :HashJoinNode   [node <- :wat::rete::HashJoinNode]
  :ProductionNode [node <- :wat::rete::ProductionNode]
  :QueryNode      [node <- :wat::rete::QueryNode])

;; ─── the session (the whole engine state) ───────────────────────────────────
;; intueri: NOT WorkingMemory — Session names the whole caller-facing engine state.

;; Session — the complete rete engine state; the caller-facing handle.
;;   network:           id → Node (raw node records) — the compiled DAG, id-indexed.
;;   rules:             PersistentVector of Rule (the rule-set as data).
;;   alpha-memory:      node-id → {join-bindings → [Element …]}
;;   beta-memory:       node-id → {join-bindings → [Token …]}
;;   production-memory: node-id → {token → [[facts] …]}  (the TM support store)
;;   facts:             PersistentVector of asserted facts.
;;   next-id:           the next free node id (i64).
(:wat::Record::def :wat::rete::Session
  [network           <- :wat::core::PersistentMap
   rules             <- :wat::core::PersistentVector
   alpha-memory      <- :wat::core::PersistentMap
   beta-memory       <- :wat::core::PersistentMap
   production-memory <- :wat::core::PersistentMap
   facts             <- :wat::core::PersistentVector
   next-id           <- :wat::core::i64])

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

;; render-dag — walk Session.network (id→Node records), emit one readable line
;; per node: "  <id>  <kind>\n". Returns the whole graph as a String.
;;
;; Strategy: get keys from the PersistentMap as a Vec<i64>, foldl over them,
;; for each key fetch the node (Option/expect), derive the kind label, concat
;; a line. Uses PersistentMap/keys (returns Vec<K>) + foldl + PersistentMap/get.
(:wat::core::defn :wat::rete::render-dag
  [session <- :wat::rete::Session]
  -> :wat::core::String
  (:wat::core::let [network (:wat::rete::Session/network session)
                    keys    (:wat::core::PersistentMap/keys network)]
    (:wat::core::foldl
      (:wat::core::fn [acc <- :wat::core::String
                       k   <- :wat::core::i64]
        -> :wat::core::String
        (:wat::core::let [node (:wat::core::Option/expect -> :wat::Record
                                   (:wat::core::PersistentMap/get network k)
                                   "render-dag: node not found")
                          kind (:wat::rete::node-kind-label node)
                          id-s (:wat::core::i64::to-string k)
                          ;; DELIBERATE proof-by-diff FIXTURE (arc 278): this nested string::concat is
                          ;; below-bar (it should be one `format`), but it is left intentionally — the
                          ;; arc-277 auto-fix is bare-symbol-only and CANNOT reach this COMPOUND/nested
                          ;; case (deferred to RETE). The wat-rete engine's own `compound-concat-collapse`
                          ;; rule will clean it; that diff is the proof the rule works. Do NOT hand-fix.
                          line (:wat::core::string::concat
                                  "  "
                                  (:wat::core::string::concat
                                    id-s
                                    (:wat::core::string::concat
                                      "  "
                                      (:wat::core::string::concat kind "\n"))))]
          (:wat::core::string::concat acc line)))
      ""
      keys)))
