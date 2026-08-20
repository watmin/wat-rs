;; wat/rete/oracle.wat — the rete interpreted fire oracle.
;;
;; insert$oracle, fire-once$oracle / fire-rules$oracle /
;; fire-rules-explain$oracle, stratify, hash-join, production,
;; accumulate-pass. Loads after wat/rete/compile.wat and wat/rete/acc.wat.
;; Public names are unprimed; rust is `$native`; this file is `$oracle`.
;; `$impl` is kwargs/bracket/service — not rete.
;;
;; Namespace: :wat::rete::

;; ─── insert + fire-rules ────────────────────────────────────────────────────────

;; insert-spec — the wat reference engine (the SPEC / differential oracle). Stages a fact into
;; the session's working memory. Zero activation.
;; WHY zero activation: the WM stays open while the caller stages multiple facts;
;; fire-rules is the lock that runs them through the network all at once.
;; WHY reconstruct Session: Record/assoc returns the base :wat::core::Record type; the
;; typed Session constructor preserves the concrete return type for the checker.
(:wat::core::defn :wat::rete::insert$oracle
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
(:wat::core::defn :wat::rete::insert-all$oracle
  [session <- :wat::rete::Session
   facts   <- :wat::core::PersistentVector<wat::core::Record>]
  -> :wat::rete::Session
  (:wat::core::foldl
    :wat::rete::insert$oracle
    session
    facts))

;; insert-all — public batch verb. Keyword-head calls are intercepted by rust
;; (`insert-all`). This defn exists so `:wat::rete::insert-all` is a first-class Fn.
(:wat::core::defn :wat::rete::insert-all
  [session <- :wat::rete::Session
   facts   <- :wat::core::PersistentVector<wat::core::Record>]
  -> :wat::rete::Session
  (:wat::rete::insert-all$native session facts))

;; insert — public production verb. Runtime intercepts the keyword head
;; (`eval_insert_public`: 2-ary native, 3+ insert-all). This defclause is the
;; type surface and the first-class Fn; bodies re-enter the keyword head.
(:wat::core::defclause :wat::rete::insert
  ([session <- :wat::rete::Session
    fact    <- :T] -> :wat::rete::Session
    (:wat::rete::insert$native session fact))
  ([session <- :wat::rete::Session
    fact    <- :T
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
;; None if no alpha was minted (legacy WM-scan fallback).
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
;; Fact: each matching WM fact. `:and`: backtrack. `:or`: concat arms. `:where`: keep or drop.
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
(:wat::core::defn :wat::rete::fire-once$oracle
  [session <- :wat::rete::Session]
  -> :wat::rete::Session
  (:wat::core::let [network  (:wat::rete::Session/network session)
                    rules    (:wat::rete::Session/rules   session)
                    _export (:wat::core::Option/expect
                              (:wat::core::if
                                (:wat::core::if (:wat::core::empty? rules)
                                  (:wat::rete::network-has-production? network)
                                  false)
                                :wat::core::None
                                (:wat::core::Some nil))
                              "fire-once: oracle cannot consume an Export — empty rules, live network")
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

;; fire-once — public single-pass verb. Keyword-head is rust; this defn is the first-class Fn.
(:wat::core::defn :wat::rete::fire-once
  [session <- :wat::rete::Session]
  -> :wat::rete::Session
  (:wat::rete::fire-once$native session))

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
;; so derived facts in facts are matched exactly like input facts on the next round. The oracle
;; never incrementals — native fire is `fire_fixpoint_delta` (P4b).
;; Internal: the returned Session.facts = the whole closure (input + derived), which is what the
;; matching machinery needs across rounds. The PUBLIC caller (fire-rules) restores facts = input only.
(:wat::core::defn :wat::rete::fire-fixpoint
  [session <- :wat::rete::Session]
  -> :wat::rete::Session
  (:wat::core::let [fired     (:wat::rete::fire-once$oracle session)
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

;; type-name-of — colon-stripped fact-type head, or None for engine forms / ?var.
(:wat::core::defn :wat::rete::type-name-of
  [form <- :wat::WatAST] -> :wat::core::Option<wat::core::String>
  (:wat::core::let [ch (:wat::core::ast->children form)]
    (:wat::core::if (:wat::core::empty? ch)
      :wat::core::None
      (:wat::core::let [raw (:wat::core::ast-name (:wat::core::first ch))
                        n   (:wat::core::string::length raw)
                        q?  (:wat::core::if (:wat::core::i64::>= n 1)
                              (:wat::core::= (:wat::core::string::subs raw 0 1) "?")
                              false)
                        rete? (:wat::core::if (:wat::core::i64::>= n 12)
                                (:wat::core::= (:wat::core::string::subs raw 0 12) ":wat::rete::")
                                false)]
        (:wat::core::if (:wat::core::if q? true rete?)
          :wat::core::None
          (:wat::core::Some
            (:wat::core::if (:wat::core::= (:wat::core::string::subs raw 0 1) ":")
              (:wat::core::string::subs raw 1 n)
              raw)))))))

;; negated-types-under — leaves under :not, including :and/:or combinators.
(:wat::core::defn :wat::rete::negated-types-under
  [form <- :wat::WatAST] -> :wat::core::PersistentVector<wat::core::String>
  (:wat::core::let [ch (:wat::core::ast->children form)
                    hd (:wat::core::if (:wat::core::empty? ch)
                         ""
                         (:wat::core::ast-name (:wat::core::first ch)))]
    (:wat::core::if (:wat::core::if (:wat::core::= hd ":wat::rete::and")
                      true
                      (:wat::core::= hd ":wat::rete::or"))
      (:wat::core::foldl
        (:wat::core::fn [acc <- :wat::core::PersistentVector<wat::core::String>
                         kid <- :wat::WatAST]
          -> :wat::core::PersistentVector<wat::core::String>
          (:wat::core::foldl
            (:wat::core::fn [a <- :wat::core::PersistentVector<wat::core::String>
                             t <- :wat::core::String]
              -> :wat::core::PersistentVector<wat::core::String>
              (:wat::core::PersistentVector/conj a t))
            acc
            (:wat::rete::negated-types-under kid)))
        (:wat::core::PersistentVector)
        (:wat::core::rest ch))
      (:wat::core::if (:wat::core::= hd ":wat::rete::not")
        (:wat::rete::negated-types-under (:wat::core::second ch))
        (:wat::core::match (:wat::rete::type-name-of form)
          ((:wat::core::Some n) (:wat::core::PersistentVector n))
          (:wat::core::None (:wat::core::PersistentVector)))))))

;; rule-negates — :not of a fact AND :not of :and/:or. Leaves, not "wat::rete::and".
(:wat::core::defn :wat::rete::rule-negates
  [rule <- :wat::rete::Rule]
  -> :wat::core::PersistentVector<wat::core::String>
  (:wat::core::let [lhs (:wat::rete::Rule/lhs rule)]
    (:wat::core::foldl
      (:wat::core::fn [acc  <- :wat::core::PersistentVector<wat::core::String>
                       form <- :wat::WatAST]
        -> :wat::core::PersistentVector<wat::core::String>
        (:wat::core::let [ch (:wat::core::ast->children form)
                          hd (:wat::core::if (:wat::core::empty? ch)
                               ""
                               (:wat::core::ast-name (:wat::core::first ch)))]
          (:wat::core::if (:wat::core::= hd ":wat::rete::not")
            (:wat::core::foldl
              (:wat::core::fn [a <- :wat::core::PersistentVector<wat::core::String>
                               t <- :wat::core::String]
                -> :wat::core::PersistentVector<wat::core::String>
                (:wat::core::PersistentVector/conj a t))
              acc
              (:wat::rete::negated-types-under (:wat::core::second ch)))
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
;; Engine forms :not / :where are NOT positive reads. :exists inner and
;; accumulate :from ARE — lockstep with native `rule_consumes`. A `?n`
;; accumulate head is not a type.
(:wat::core::defn :wat::rete::rule-consumes
  [rule <- :wat::rete::Rule]
  -> :wat::core::PersistentVector<wat::core::String>
  (:wat::core::let [lhs (:wat::rete::Rule/lhs rule)]
    (:wat::core::foldl
      (:wat::core::fn [acc  <- :wat::core::PersistentVector<wat::core::String>
                       form <- :wat::WatAST]
        -> :wat::core::PersistentVector<wat::core::String>
        (:wat::core::let [ch (:wat::core::ast->children form)
                          hd (:wat::core::if (:wat::core::empty? ch)
                               ""
                               (:wat::core::ast-name (:wat::core::first ch)))
                          n  (:wat::core::string::length hd)
                          q? (:wat::core::if (:wat::core::i64::>= n 1)
                               (:wat::core::= (:wat::core::string::subs hd 0 1) "?")
                               false)]
          (:wat::core::if (:wat::core::= hd ":wat::rete::exists")
            (:wat::core::match (:wat::rete::type-name-of (:wat::core::second ch))
              ((:wat::core::Some t) (:wat::core::PersistentVector/conj acc t))
              (:wat::core::None acc))
            (:wat::core::if (:wat::core::if q?
                              (:wat::core::if (:wat::core::i64::>= (:wat::core::length ch) 5)
                                (:wat::core::= (:wat::core::ast-name
                                                 (:wat::core::Option/expect
                                                   (:wat::core::get ch 3)
                                                   "rule-consumes: acc :from"))
                                  ":from")
                                false)
                              false)
              (:wat::core::match (:wat::rete::type-name-of
                                   (:wat::core::Option/expect
                                     (:wat::core::get ch 4)
                                     "rule-consumes: acc :from inner"))
                ((:wat::core::Some t) (:wat::core::PersistentVector/conj acc t))
                (:wat::core::None acc))
              (:wat::core::if (:wat::core::if (:wat::core::i64::>= n 12)
                                (:wat::core::= (:wat::core::string::subs hd 0 12) ":wat::rete::")
                                false)
                acc
                (:wat::core::match (:wat::rete::type-name-of form)
                  ((:wat::core::Some t) (:wat::core::PersistentVector/conj acc t))
                  (:wat::core::None acc)))))))
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
                                      (:wat::rete::insert$oracle s f))
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
                    q-fired   (:wat::rete::fire-once$oracle q-seed)]
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
;;
;; Query-only compile-all (empty rules, QueryNodes, no ProductionNode) is legal —
;; the oracle walks QueryNodes. An imported Export of production rules has empty
;; rules AND ProductionNodes (no AST) — refuse that, do not silently harvest 0.
(:wat::core::defn :wat::rete::network-has-production?
  [net <- :wat::core::PersistentMap]
  -> :wat::core::bool
  (:wat::core::foldl
    (:wat::core::fn [acc <- :wat::core::bool
                     k   <- :wat::core::i64]
      -> :wat::core::bool
      (:wat::core::if acc
        true
        (:wat::core::let [node (:wat::core::Option/expect
                                  (:wat::core::PersistentMap/get net k)
                                  "network-has-production?: node")]
          (:wat::core::= (:wat::rete::node-kind-label node) "ProductionNode"))))
    false
    (:wat::core::PersistentMap/keys net)))

(:wat::core::defn :wat::rete::fire-rules$oracle
  [session <- :wat::rete::Session]
  -> :wat::rete::Session
  (:wat::core::let [input (:wat::rete::Session/facts session)
                    rules (:wat::rete::Session/rules session)
                    net   (:wat::rete::Session/network session)
                    _export (:wat::core::Option/expect
                              (:wat::core::if
                                (:wat::core::if (:wat::core::empty? rules)
                                  (:wat::rete::network-has-production? net)
                                  false)
                                :wat::core::None
                                (:wat::core::Some nil))
                              "fire-rules-spec: oracle cannot consume an Export — empty rules, live network")
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

;; harvest-support — first-producer-wins index: derived-fact → Support{rule, token}.
;; Replay on a session whose beta is still live (fire-once$oracle of the closure).
(:wat::core::defn :wat::rete::harvest-support
  [network  <- :wat::core::PersistentMap
   beta-mem <- :wat::core::PersistentMap
   rules    <- :wat::core::PersistentVector<wat::rete::Rule>]
  -> :wat::core::PersistentMap
  (:wat::core::foldl
    (:wat::core::fn [sup     <- :wat::core::PersistentMap
                     node-id <- :wat::core::i64]
      -> :wat::core::PersistentMap
      (:wat::core::let [node (:wat::core::Option/expect
                                (:wat::core::PersistentMap/get network node-id)
                                "harvest-support: node")]
        (:wat::core::if (:wat::core::= (:wat::rete::node-kind-label node) "ProductionNode")
          (:wat::core::let [rname (:wat::rete::ProductionNode/rule-name node)
                            rule  (:wat::rete::rule-by-name rules rname)
                            rhs   (:wat::rete::Rule/rhs rule)
                            toks  (:wat::rete::tokens-from-parents beta-mem
                                    (:wat::rete::node-parents node-id network))]
            (:wat::core::foldl
              (:wat::core::fn [s   <- :wat::core::PersistentMap
                               tok <- :wat::rete::Token]
                -> :wat::core::PersistentMap
                (:wat::core::foldl
                  (:wat::core::fn [s2   <- :wat::core::PersistentMap
                                   form <- :wat::WatAST]
                    -> :wat::core::PersistentMap
                    (:wat::core::let [derived (:wat::rete::eval-insert form
                                                 (:wat::rete::Token/bindings tok))]
                      (:wat::core::match (:wat::core::PersistentMap/get s2 derived)
                        ((:wat::core::Some _) s2)
                        (:wat::core::None
                         (:wat::core::PersistentMap/assoc s2 derived
                           (:wat::rete::Support :rule rname :token tok))))))
                  s
                  rhs))
              sup
              toks))
          sup)))
    (:wat::core::PersistentMap)
    (:wat::core::PersistentMap/keys network)))

;; fire-rules-explain$oracle — wat reference for explain. Same session as
;; fire-rules$oracle; support harvested from a fire-once$oracle replay of the
;; closure so beta is live. First-producer-wins, matching the native index.
(:wat::core::defn :wat::rete::fire-rules-explain$oracle
  [session <- :wat::rete::Session]
  -> :wat::rete::Explained
  (:wat::core::let [input       (:wat::rete::Session/facts session)
                    oracle-sess (:wat::rete::fire-rules$oracle session)
                    derived     (:wat::rete::collect-derived
                                  (:wat::rete::Session/production-memory oracle-sess))
                    closed      (:wat::rete::merge-facts input derived)
                    empty       (:wat::core::PersistentMap)
                    replay      (:wat::rete::fire-once$oracle
                                  (:wat::rete::Session
                                    :network (:wat::rete::Session/network session)
                                    :rules (:wat::rete::Session/rules session)
                                    :alpha-memory empty
                                    :beta-memory empty
                                    :production-memory empty
                                    :facts closed
                                    :next-id (:wat::rete::Session/next-id session)
                                    :query-memory empty))
                    support     (:wat::rete::harvest-support
                                  (:wat::rete::Session/network replay)
                                  (:wat::rete::Session/beta-memory replay)
                                  (:wat::rete::Session/rules session))]
    (:wat::rete::Explained :session oracle-sess :support support)))

;; fire-rules — public production verb. Keyword-head calls are intercepted by
;; rust (`eval_fire_rules_native`). This defn is the first-class Fn; the body
;; re-enters the keyword head.
(:wat::core::defn :wat::rete::fire-rules
  [session <- :wat::rete::Session]
  -> :wat::rete::Session
  (:wat::rete::fire-rules$native session))

;; fire-rules-explain — opt-in diagnostic fire. Same intercept/Fn split.
(:wat::core::defn :wat::rete::fire-rules-explain
  [session <- :wat::rete::Session]
  -> :wat::rete::Explained
  (:wat::rete::fire-rules-explain$native session))

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
                                            (:wat::core::match (:wat::rete::alpha-match-under from-cond
                                                                 (:wat::rete::Element/fact el)
                                                                 (:wat::rete::Token/bindings tok))
                                              ((:wat::core::Some _)
                                               (:wat::core::PersistentVector/conj acc el))
                                              (:wat::core::None acc)))
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
