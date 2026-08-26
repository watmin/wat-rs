;; wat/rete/oracle/fire.wat — interpreted fire-once / fire-rules.
;;
;; walk-alpha-ids / walk-beta-ids / walk-filter-ids / walk-prod-ids,
;; fire-once$oracle, fire-fixpoint, fire-stratified (drive),
;; fire-rules$oracle, public fire-rules / fire-once / fire-rules-explain.
;; Loads after stratify.wat (numbering) and accum-pass.wat (walk-filter-ids).
;;
;; Namespace: :wat::rete::

;; Four monomorphic TCO walkers. `walk-sorted-ids` used to take a `phase <- i64`
;; and `cond` across four bodies that return three different memory types
;; through one `acc`. Every call site passed a literal phase; the recursive
;; self-call threaded it unchanged. Split: each walker recurses on itself,
;; `acc` is the memory that walker actually writes, `phase`/`cond` are gone.

;; walk-alpha-ids — activate-alpha over sorted node-ids.
;; acc: node-id → (PV :- [Element]) (FLAT — assoc under alpha-id, not nested by bindings).
(:wat::core::defn :wat::rete::walk-alpha-ids
  [facts   <- :wat::core::PersistentVector
   network <- :wat::core::PersistentMap
   ids     <- (:wat::core::Vector :- [:wat::core::i64])
   i       <- :wat::core::i64
   acc     <- :wat::rete::AlphaMemory]
  -> :wat::rete::AlphaMemory
  (:wat::core::if (:wat::i64::>= i (:wat::core::length ids))
    acc
    (:wat::core::let [node-id (:wat::core::Option/expect
                                 (:wat::core::get ids i)
                                 "walk-alpha-ids: id")
                      acc1    (:wat::rete::activate-alpha facts network acc node-id)]
      (:wat::rete::walk-alpha-ids facts network ids
        (:wat::i64::+ i 1) acc1))))

;; walk-beta-ids — root-join-pass over sorted node-ids. Reads amem; writes beta.
;; acc: node-id → (PV :- [Token]).
(:wat::core::defn :wat::rete::walk-beta-ids
  [network <- :wat::core::PersistentMap
   amem    <- :wat::rete::AlphaMemory
   ids     <- (:wat::core::Vector :- [:wat::core::i64])
   i       <- :wat::core::i64
   acc     <- :wat::rete::BetaMemory]
  -> :wat::rete::BetaMemory
  (:wat::core::if (:wat::i64::>= i (:wat::core::length ids))
    acc
    (:wat::core::let [node-id (:wat::core::Option/expect
                                 (:wat::core::get ids i)
                                 "walk-beta-ids: id")
                      acc1    (:wat::rete::root-join-pass amem network acc node-id)]
      (:wat::rete::walk-beta-ids network amem ids
        (:wat::i64::+ i 1) acc1))))

;; walk-filter-ids — populate-then-emit walk (accumulate-pass, then filter-pass,
;; then hash-join-pass). Reads facts+amem; threads beta. acc: node-id → (PV :- [Token]).
;; rune:intueri(naming) — oracle populate-then-emit walker (acc+filter+hash-join);
;; the name is the historical walk-sorted-ids split, not filter-alone. Rename
;; would fork every oracle fire caller.
(:wat::core::defn :wat::rete::walk-filter-ids
  [facts   <- :wat::core::PersistentVector
   network <- :wat::core::PersistentMap
   amem    <- :wat::rete::AlphaMemory
   ids     <- (:wat::core::Vector :- [:wat::core::i64])
   i       <- :wat::core::i64
   acc     <- :wat::rete::BetaMemory]
  -> :wat::rete::BetaMemory
  (:wat::core::if (:wat::i64::>= i (:wat::core::length ids))
    acc
    (:wat::core::let [node-id (:wat::core::Option/expect
                                 (:wat::core::get ids i)
                                 "walk-filter-ids: id")
                      acc1    (:wat::rete::hash-join-pass amem network
                                (:wat::rete::filter-pass network amem facts
                                  (:wat::rete::accumulate-pass network amem acc node-id)
                                  node-id)
                                node-id)]
      (:wat::rete::walk-filter-ids facts network amem ids
        (:wat::i64::+ i 1) acc1))))

;; walk-prod-ids — production-pass over sorted node-ids. Reads bmem+rules; writes
;; production. acc: node-id → (PV :- [Record]).
(:wat::core::defn :wat::rete::walk-prod-ids
  [network <- :wat::core::PersistentMap
   bmem    <- :wat::rete::BetaMemory
   rules   <- (:wat::core::PersistentVector :- [:wat::rete::Rule])
   ids     <- (:wat::core::Vector :- [:wat::core::i64])
   i       <- :wat::core::i64
   acc     <- :wat::rete::ProductionMemory]
  -> :wat::rete::ProductionMemory
  (:wat::core::if (:wat::i64::>= i (:wat::core::length ids))
    acc
    (:wat::core::let [node-id (:wat::core::Option/expect
                                 (:wat::core::get ids i)
                                 "walk-prod-ids: id")
                      acc1    (:wat::rete::production-pass network bmem rules acc node-id)]
      (:wat::rete::walk-prod-ids network bmem rules ids
        (:wat::i64::+ i 1) acc1))))

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
                                     (:wat::core::fn [a   <- (:wat::core::PersistentVector :- [:wat::core::PersistentMap])
                                                      tok <- :wat::rete::Token]
                                       -> (:wat::core::PersistentVector :- [:wat::core::PersistentMap])
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
;; go to production-memory only — they do not re-enter facts here (cascade is `fire-rules`'s job).
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
                    new-amem (:wat::rete::walk-alpha-ids facts network node-ids 0
                                 (:wat::core::PersistentMap))
                    new-bmem (:wat::rete::walk-beta-ids network new-amem node-ids 0
                                 (:wat::core::PersistentMap))
                    filtered-bmem (:wat::rete::walk-filter-ids facts network new-amem node-ids 0 new-bmem)
                    new-pmem (:wat::rete::walk-prod-ids network filtered-bmem rules node-ids 0
                                 (:wat::core::PersistentMap))
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

;; collect-derived — flatten production-memory's per-node (PV :- [Record]) values into one (PV :- [:wat::core::Record]).
;; WHY foldl-over-values: production-memory is a PersistentMap from node-id to (PV :- [Record]);
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

;; Stratification numbering lives in wat/rete/oracle/stratify.wat
;; (StratifyAcc + rule-produces through stratify). Fire-stratified drive stays here.

;; FireStratAcc — fold accumulator for fire-stratified.
;; facts:   accumulated Session.facts after each stratum (input + all derived so far).
;; derived: dedup union of all derived facts across completed strata.
(:wat::core::defrecord :wat::rete::FireStratAcc
  [facts   <- :wat::core::PersistentVector
   derived <- :wat::core::PersistentVector])


;; fire-stratified-loop — recursive descent over strata [current..max-s].
;; Filters the original `rules` (typed (PersistentVector :- [Rule])) to the current stratum
;; on each call, avoiding type erasure that would occur from storing rule groups in an
;; outer PersistentVector. Threads (acc-facts, acc-derived) forward across strata.
;;
;; WHY recursive rather than foldl-over-a-PV: foldl would require the inner elements
;; to be declared as PersistentVector (unparameterised), losing Rule type information
;; and causing compile to reject the argument at the call site. Recursive descent on
;; an index always filters the original typed PV — no type information is lost.
(:wat::core::defn :wat::rete::fire-stratified-loop
  [rules       <- (:wat::core::PersistentVector :- [:wat::rete::Rule])
   type-strata <- (:wat::core::HashMap :- [:wat::core::String :wat::core::i64])
   current     <- :wat::core::i64
   max-s       <- :wat::core::i64
   acc-facts   <- :wat::core::PersistentVector
   acc-derived <- :wat::core::PersistentVector]
  -> :wat::rete::FireStratAcc
  (:wat::core::if (:wat::i64::> current max-s)
    (:wat::rete::FireStratAcc :facts acc-facts :derived acc-derived)
    (:wat::core::let [;; Arc 118.2a — `filter` flipped LAZY; `compile` needs `(PersistentVector :- [Rule])`
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
        (:wat::i64::+ current 1)
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
                                    (:wat::core::if (:wat::i64::> rs mx) rs mx)))
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

;; fire-rules$oracle — the wat reference engine (the SPEC / differential oracle).
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
                              "fire-rules$oracle: oracle cannot consume an Export — empty rules, live network")
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

;; fire-rules — public production verb. Keyword-head calls and this first-class
;; Fn body both reach rust through `$native` (`runtime.rs`).
(:wat::core::defn :wat::rete::fire-rules
  [session <- :wat::rete::Session]
  -> :wat::rete::Session
  (:wat::rete::fire-rules$native session))

;; fire-rules-explain — opt-in diagnostic fire. Same intercept/Fn split.
(:wat::core::defn :wat::rete::fire-rules-explain
  [session <- :wat::rete::Session]
  -> :wat::rete::Explained
  (:wat::rete::fire-rules-explain$native session))

