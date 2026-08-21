;; wat/rete/oracle/pass.wat — interpreted alpha / join / production passes.
;;
;; activate-fact through production-pass. Loads after compile.wat (alpha-id-for-cond
;; / cond helpers live there). fire-once walks these as fold steps.
;;
;; Namespace: :wat::rete::

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
  (:wat::core::let [match-result (:wat::core::if
                                   (:wat::rete::cond-has-deferred-constraint? cond)
                                   (:wat::rete::alpha-match-local cond fact)
                                   (:wat::rete::alpha-match cond fact))]
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

;; any-seeded-element? — rete exists/not over alpha elements, rematched with
;; the token's left bindings (`alpha-match-under`). Compatible-only drops
;; leftover `?v < ?m` (Clara test-accum-result-in-negation).
(:wat::core::defn :wat::rete::any-seeded-element?
  [cond     <- :wat::WatAST
   bindings <- :wat::core::PersistentMap
   els      <- :wat::core::PersistentVector<wat::rete::Element>]
  -> :wat::core::bool
  (:wat::core::foldl
    (:wat::core::fn [found <- :wat::core::bool
                     el    <- :wat::rete::Element]
      -> :wat::core::bool
      (:wat::core::if found
        true
        (:wat::core::match (:wat::rete::alpha-match-under cond
                             (:wat::rete::Element/fact el) bindings)
          ((:wat::core::Some _) true)
          (:wat::core::None false))))
    false
    els))

;; alpha-id-for-cond — the AlphaNode whose tests[0] write-forms equals cond.
(:wat::core::defn :wat::rete::alpha-id-for-cond
  [network <- :wat::core::PersistentMap
   cond    <- :wat::WatAST]
  -> :wat::core::Option<wat::core::i64>
  (:wat::core::let [want (:wat::core::write-forms cond)]
    (:wat::core::let [found (:wat::core::foldl
                              (:wat::core::fn [acc     <- :wat::core::i64
                                               node-id <- :wat::core::i64]
                                -> :wat::core::i64
                                (:wat::core::if (:wat::core::i64::>= acc 0)
                                  acc
                                  (:wat::core::let [node (:wat::core::Option/expect
                                                           (:wat::core::PersistentMap/get network node-id)
                                                           "alpha-id-for-cond: node missing")]
                                    (:wat::core::if (:wat::core::= (:wat::rete::node-kind-label node) "AlphaNode")
                                      (:wat::core::let [t0 (:wat::core::Option/expect
                                                              (:wat::core::get (:wat::rete::AlphaNode/tests node) 0)
                                                              "alpha-id-for-cond: no tests")]
                                        (:wat::core::if (:wat::core::= (:wat::core::write-forms t0) want)
                                          node-id
                                          -1))
                                      -1))))
                              -1
                              (:wat::core::PersistentMap/keys network))]
      (:wat::core::if (:wat::core::i64::>= found 0)
        (:wat::core::Some found)
        :wat::core::None))))

;; alpha-els-for-cond — Some(els) if that cond has an alpha (possibly empty);
;; None if no alpha was minted (legacy facts-scan fallback).
(:wat::core::defn :wat::rete::alpha-els-for-cond
  [network   <- :wat::core::PersistentMap
   alpha-mem <- :wat::core::PersistentMap
   cond      <- :wat::WatAST]
  -> :wat::core::Option<wat::core::PersistentVector<wat::rete::Element>>
  (:wat::core::match (:wat::rete::alpha-id-for-cond network cond)
    ((:wat::core::Some id)
     (:wat::core::match (:wat::core::PersistentMap/get alpha-mem id)
       ((:wat::core::Some pv) (:wat::core::Some pv))
       (:wat::core::None (:wat::core::Some (:wat::core::PersistentVector)))))
    (:wat::core::None :wat::core::None)))

;; token-exists-under — mid-chain :exists / :not. Fact inner → seeded rematch
;; over that node's alpha. Combinator / where inner → exists-cond-under, which
;; now probes each leaf's alpha.
(:wat::core::defn :wat::rete::token-exists-under
  [tok       <- :wat::rete::Token
   cond      <- :wat::WatAST
   facts     <- :wat::core::PersistentVector
   els       <- :wat::core::PersistentVector<wat::rete::Element>
   network   <- :wat::core::PersistentMap
   alpha-mem <- :wat::core::PersistentMap]
  -> :wat::core::bool
  (:wat::core::if (:wat::rete::exists-uses-alpha-probe? cond)
    (:wat::rete::any-seeded-element? cond (:wat::rete::Token/bindings tok) els)
    (:wat::rete::exists-cond-under cond facts (:wat::rete::Token/bindings tok)
      network alpha-mem)))

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
;; Rematch the right cond under the token's bindings (`alpha-match-under`) so an
;; inline leftover (`?w > ?c`, Clara beta) is a join filter, not dropped.
;; Shared-var agreement is included in that rematch (bind of ?loc against seed).
(:wat::core::defn :wat::rete::cross-join-node
  [tokens   <- :wat::core::PersistentVector
   elements <- :wat::core::PersistentVector
   hj-id    <- :wat::core::i64
   alpha-id <- :wat::core::i64
   cond     <- :wat::WatAST
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
          (:wat::core::match (:wat::rete::alpha-match-under cond
                               (:wat::rete::Element/fact el)
                               (:wat::rete::Token/bindings tok))
            ((:wat::core::Some _)
             (:wat::rete::append-token bm2 hj-id (:wat::rete::extend-token tok el alpha-id)))
            (:wat::core::None bm2)))
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
                      (:wat::core::let [alpha-node (:wat::core::Option/expect
                                                      (:wat::core::PersistentMap/get network aid)
                                                      "hash-join-pass: feeding alpha missing")
                                        cond      (:wat::core::Option/expect
                                                      (:wat::core::get (:wat::rete::AlphaNode/tests alpha-node) 0)
                                                      "hash-join-pass: feeding alpha has no cond")]
                        (:wat::rete::cross-join-node tokens els child-id aid cond bm)))
                     (:wat::core::None bm)))
                 bm)))
           beta-mem
           (:wat::rete::node-children-ids node)))
        (:wat::core::None beta-mem))
      beta-mem)))

;; ─── production pass (stone 4a) ────────────────────────────────────────────

;; node-parents — every node that names `child-id` as a child. Condition `:or`
;; wires one ProductionNode to N arm terminals; fire must read ALL of them.
;; WHY kind-agnostic via node-children-ids: a ProductionNode's parent is a RootJoinNode
;; (1-condition rule) OR a HashJoinNode (multi-condition rule).
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

;; binding-extensions — every binding map that satisfies `cond` under `bindings`.
;; Fact: each matching fact. `:and`: backtrack. `:or`: concat arms. `:where`: keep or drop.
(:wat::core::defn :wat::rete::binding-extensions
  [cond      <- :wat::WatAST
   facts     <- :wat::core::PersistentVector
   bindings  <- :wat::core::PersistentMap
   network   <- :wat::core::PersistentMap
   alpha-mem <- :wat::core::PersistentMap]
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
                 (:wat::rete::binding-extensions kid facts ext network alpha-mem)))
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
             (:wat::rete::binding-extensions kid facts bindings network alpha-mem)))
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
                         facts bindings network alpha-mem)
         (:wat::core::PersistentVector)
         (:wat::core::PersistentVector/conj (:wat::core::PersistentVector) bindings)))
      (:else
       (:wat::core::match (:wat::rete::alpha-els-for-cond network alpha-mem cond)
         ((:wat::core::Some els)
          (:wat::core::foldl
            (:wat::core::fn [acc <- :wat::core::PersistentVector<wat::core::PersistentMap>
                             el  <- :wat::rete::Element]
              -> :wat::core::PersistentVector<wat::core::PersistentMap>
              (:wat::core::match (:wat::rete::alpha-match-under cond
                                   (:wat::rete::Element/fact el) bindings)
                ((:wat::core::Some b)
                 (:wat::core::PersistentVector/conj acc b))
                (:wat::core::None acc)))
            (:wat::core::PersistentVector)
            els))
         (:wat::core::None
          (:wat::core::foldl
            (:wat::core::fn [acc  <- :wat::core::PersistentVector<wat::core::PersistentMap>
                             fact <- :wat::core::Record]
              -> :wat::core::PersistentVector<wat::core::PersistentMap>
              (:wat::core::match (:wat::rete::alpha-match-under cond fact bindings)
                ((:wat::core::Some b)
                 (:wat::core::PersistentVector/conj acc b))
                (:wat::core::None acc)))
            (:wat::core::PersistentVector)
            facts)))))))

;; exists-cond-under — does the inner :not/:exists condition hold under bindings?
;; A fact: any-fact-matches-under. `:and` of facts: some join of the children exists
;; (Clara [:not [:and [Wind] [Temp]]]).
(:wat::core::defn :wat::rete::exists-cond-under
  [cond      <- :wat::WatAST
   facts     <- :wat::core::PersistentVector
   bindings  <- :wat::core::PersistentMap
   network   <- :wat::core::PersistentMap
   alpha-mem <- :wat::core::PersistentMap]
  -> :wat::core::bool
  (:wat::core::let [head-nm (:wat::core::ast-name
                              (:wat::core::first (:wat::core::ast->children cond)))]
    (:wat::core::cond
      ((:wat::core::= head-nm ":wat::rete::and")
       (:wat::core::i64::> (:wat::core::length
                             (:wat::rete::binding-extensions cond facts bindings
                               network alpha-mem))
                           0))
      ((:wat::core::= head-nm ":wat::rete::or")
       (:wat::core::foldl
         (:wat::core::fn [found <- :wat::core::bool
                          kid   <- :wat::WatAST]
           -> :wat::core::bool
           (:wat::core::if found
             true
             (:wat::rete::exists-cond-under kid facts bindings network alpha-mem)))
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
           facts bindings network alpha-mem)))
      (:else
       (:wat::core::match (:wat::rete::alpha-els-for-cond network alpha-mem cond)
         ((:wat::core::Some els)
          (:wat::rete::any-seeded-element? cond bindings els))
         (:wat::core::None
          (:wat::rete::any-fact-matches-under cond facts bindings)))))))

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
       ;; pass the un-extended token iff ZERO compatible elements in the
       ;; negated alpha (fact-shaped inner). Combinator / :where inners keep
       ;; exists-cond-under. Leading :not seeds one empty token.
       (:wat::core::let [neg-alpha-id (:wat::rete::NegationNode/negated-alpha-id node)
                         tokens       (:wat::rete::tokens-or-empty-seed
                                        network beta-mem node-id)
                         alpha-node   (:wat::core::Option/expect
                                         (:wat::core::PersistentMap/get network neg-alpha-id)
                                         "filter-pass: negated alpha missing")
                         cond         (:wat::core::Option/expect
                                         (:wat::core::get (:wat::rete::AlphaNode/tests alpha-node) 0)
                                         "filter-pass: negated alpha has no cond")
                         els          (:wat::core::match
                                         (:wat::core::PersistentMap/get alpha-mem neg-alpha-id)
                                         ((:wat::core::Some pv) pv)
                                         (:wat::core::None (:wat::core::PersistentVector)))]
         (:wat::core::foldl
           (:wat::core::fn [bm  <- :wat::core::PersistentMap
                            tok <- :wat::rete::Token]
             -> :wat::core::PersistentMap
             (:wat::core::if
               (:wat::rete::token-exists-under tok cond facts els network alpha-mem)
               bm
               (:wat::rete::append-token bm node-id tok)))
           beta-mem
           tokens)))
      ((:wat::core::= kind "ExistsNode")
       ;; Mid-chain: pass the un-extended parent token once if the inner holds.
       ;; Leading: one token per DISTINCT inner binding (Clara test-simple-exists)
       ;; — those bindings are the alpha elements, not a rescan of every fact.
       (:wat::core::let [ex-alpha-id (:wat::rete::ExistsNode/exists-alpha-id node)
                         pids        (:wat::rete::node-parents node-id network)
                         alpha-node  (:wat::core::Option/expect
                                        (:wat::core::PersistentMap/get network ex-alpha-id)
                                        "filter-pass: exists alpha missing")
                         cond        (:wat::core::Option/expect
                                        (:wat::core::get (:wat::rete::AlphaNode/tests alpha-node) 0)
                                        "filter-pass: exists alpha has no cond")
                         els         (:wat::core::match
                                        (:wat::core::PersistentMap/get alpha-mem ex-alpha-id)
                                        ((:wat::core::Some pv) pv)
                                        (:wat::core::None (:wat::core::PersistentVector)))]
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
               (:wat::core::if (:wat::rete::exists-uses-alpha-probe? cond)
                 (:wat::core::foldl
                   (:wat::core::fn [acc <- :wat::core::PersistentVector<wat::core::PersistentMap>
                                    el  <- :wat::rete::Element]
                     -> :wat::core::PersistentVector<wat::core::PersistentMap>
                     (:wat::core::PersistentVector/conj acc
                       (:wat::rete::Element/bindings el)))
                   (:wat::core::PersistentVector)
                   els)
                 (:wat::rete::binding-extensions
                   cond facts (:wat::core::PersistentMap)
                   network alpha-mem))))
           (:wat::core::foldl
             (:wat::core::fn [bm  <- :wat::core::PersistentMap
                              tok <- :wat::rete::Token]
               -> :wat::core::PersistentMap
               (:wat::core::if
                 (:wat::rete::token-exists-under tok cond facts els network alpha-mem)
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

