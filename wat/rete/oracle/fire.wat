;; wat/rete/oracle/fire.wat — interpreted fire-once / fire-rules / stratify.
;;
;; walk-alpha-ids / walk-beta-ids / walk-filter-ids / walk-prod-ids,
;; fire-once$oracle, fire-fixpoint, fire-stratified,
;; fire-rules$oracle, public fire-rules / fire-once / fire-rules-explain.
;; Loads after accum-pass.wat (walk-filter-ids calls accumulate-pass).
;;
;; Namespace: :wat::rete::

;; Four monomorphic TCO walkers. `walk-sorted-ids` used to take a `phase <- i64`
;; and `cond` across four bodies that return three different memory types
;; through one `acc`. Every call site passed a literal phase; the recursive
;; self-call threaded it unchanged. Split: each walker recurses on itself,
;; `acc` is the memory that walker actually writes, `phase`/`cond` are gone.

;; walk-alpha-ids — activate-alpha over sorted node-ids.
;; acc: node-id → PV<Element> (FLAT — assoc under alpha-id, not nested by bindings).
(:wat::core::defn :wat::rete::walk-alpha-ids
  [facts   <- :wat::core::PersistentVector
   network <- :wat::core::PersistentMap
   ids     <- :wat::core::Vector<wat::core::i64>
   i       <- :wat::core::i64
   acc     <- :wat::core::PersistentMap<wat::core::i64,wat::core::PersistentVector<wat::rete::Element>>]
  -> :wat::core::PersistentMap<wat::core::i64,wat::core::PersistentVector<wat::rete::Element>>
  (:wat::core::if (:wat::core::i64::>= i (:wat::core::length ids))
    acc
    (:wat::core::let [node-id (:wat::core::Option/expect
                                 (:wat::core::get ids i)
                                 "walk-alpha-ids: id")
                      acc1    (:wat::rete::activate-alpha facts network acc node-id)]
      (:wat::rete::walk-alpha-ids facts network ids
        (:wat::core::i64::+ i 1) acc1))))

;; walk-beta-ids — root-join-pass over sorted node-ids. Reads amem; writes beta.
;; acc: node-id → PV<Token>.
(:wat::core::defn :wat::rete::walk-beta-ids
  [network <- :wat::core::PersistentMap
   amem    <- :wat::core::PersistentMap<wat::core::i64,wat::core::PersistentVector<wat::rete::Element>>
   ids     <- :wat::core::Vector<wat::core::i64>
   i       <- :wat::core::i64
   acc     <- :wat::core::PersistentMap<wat::core::i64,wat::core::PersistentVector<wat::rete::Token>>]
  -> :wat::core::PersistentMap<wat::core::i64,wat::core::PersistentVector<wat::rete::Token>>
  (:wat::core::if (:wat::core::i64::>= i (:wat::core::length ids))
    acc
    (:wat::core::let [node-id (:wat::core::Option/expect
                                 (:wat::core::get ids i)
                                 "walk-beta-ids: id")
                      acc1    (:wat::rete::root-join-pass amem network acc node-id)]
      (:wat::rete::walk-beta-ids network amem ids
        (:wat::core::i64::+ i 1) acc1))))

;; walk-filter-ids — populate-then-emit: accumulate-pass, then filter-pass, then
;; hash-join-pass. Reads facts+amem; threads beta. acc: node-id → PV<Token>.
(:wat::core::defn :wat::rete::walk-filter-ids
  [facts   <- :wat::core::PersistentVector
   network <- :wat::core::PersistentMap
   amem    <- :wat::core::PersistentMap<wat::core::i64,wat::core::PersistentVector<wat::rete::Element>>
   ids     <- :wat::core::Vector<wat::core::i64>
   i       <- :wat::core::i64
   acc     <- :wat::core::PersistentMap<wat::core::i64,wat::core::PersistentVector<wat::rete::Token>>]
  -> :wat::core::PersistentMap<wat::core::i64,wat::core::PersistentVector<wat::rete::Token>>
  (:wat::core::if (:wat::core::i64::>= i (:wat::core::length ids))
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
        (:wat::core::i64::+ i 1) acc1))))

;; walk-prod-ids — production-pass over sorted node-ids. Reads bmem+rules; writes
;; production. acc: node-id → PV<Record>.
(:wat::core::defn :wat::rete::walk-prod-ids
  [network <- :wat::core::PersistentMap
   bmem    <- :wat::core::PersistentMap<wat::core::i64,wat::core::PersistentVector<wat::rete::Token>>
   rules   <- :wat::core::PersistentVector<wat::rete::Rule>
   ids     <- :wat::core::Vector<wat::core::i64>
   i       <- :wat::core::i64
   acc     <- :wat::core::PersistentMap<wat::core::i64,wat::core::PersistentVector<wat::core::Record>>]
  -> :wat::core::PersistentMap<wat::core::i64,wat::core::PersistentVector<wat::core::Record>>
  (:wat::core::if (:wat::core::i64::>= i (:wat::core::length ids))
    acc
    (:wat::core::let [node-id (:wat::core::Option/expect
                                 (:wat::core::get ids i)
                                 "walk-prod-ids: id")
                      acc1    (:wat::rete::production-pass network bmem rules acc node-id)]
      (:wat::rete::walk-prod-ids network bmem rules ids
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
;; WHY this location: immediately before fire-rules$oracle which it replaces.
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

