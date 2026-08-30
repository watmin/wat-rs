;; wat-scripts/perf/grid/accum.wat — GRID AXIS A5: accumulate / exists, IN WAT.
;;
;; The built-in fold library (wat/rete.wat:1989-2213) driven through an AccumulateNode over
;; matched facts, at scale — count / sum / min / max (the scalar-valued built-ins) plus :exists
;; (ExistsNode, wat/rete.wat:114-126). This is the R18 accumulate capability
;; (docs/arc/2026/06/278-rules-engine/DESIGN.md: "8 accumulators … 1:1 with Clara's set"),
;; stressed G groups wide x W readings deep.
;;
;; WHY count/sum/min/max + :exists and NOT distinct/group-by: an accuracy witness for this grid is
;; a SORTED VECTOR OF i64 compared byte-for-byte against Clara's identical workload
;; (gen-accum.sh). count/sum/min/max each bind a SCALAR i64 aggregate — it drops straight into a
;; derived record's i64 field and encodes to one canonical integer. distinct/group-by bind a
;; COLLECTION ((PV :- [i64]) / PM) whose reduction to a scalar (e.g. its cardinality) would require the
;; rule RHS to COMPUTE over the bound var — the rete action layer only inserts records from bound
;; vars + literals (no fold in the RHS), so neither engine can canonicalise a collection-valued
;; fold to an i64 witness without contortion. Both folds are exercised elsewhere (the oracle/native
;; differential tests/rete/probe_arc278_8*.rs); this axis benches the four scalar folds + :exists,
;; which is a faithful, byte-comparable slice of the same AccumulateNode machinery.
;;
;; Shape (mirrors tests/rete/probe_arc278_8a_accumulate_oracle.rs's Station/Reading/acc rule):
;;   Group(g)      — G anchor facts, g in [0, G).  The accumulate's LEFT token.
;;   Reading(g,v)  — W readings per group; v = val(g,j) is deterministic (SAME fn on both sides).
;;   CountF(g,n) :- Group(g) AND [?n <- (acc::count)   :from Reading(g)]        n = W
;;   SumF(g,n)   :- Group(g) AND [?n <- (acc::sum ?v)  :from Reading(g,?v)]     n = Σ v
;;   MinF(g,n)   :- Group(g) AND [?n <- (acc::min ?v)  :from Reading(g,?v)]     n = min v
;;   MaxF(g,n)   :- Group(g) AND [?n <- (acc::max ?v)  :from Reading(g,?v)]     n = max v
;;   ExistsF(g)  :- Group(g) AND (exists Reading(g))                            fires (W>=1)
;; W>=1 always ⇒ min/max (Option folds) are always Some ⇒ every group emits all five derived facts.
;;
;; Fires the NATIVE production verb `:wat::rete::fire-rules` (the differential-tested fast path;
;; NOT the wat oracle `fire-rules-spec`). :derived is the FULL SORTED derived-fact set, each fact
;; canonicalised to one i64 (kind*1e15 + g*1e9 + val), so it compares byte-for-byte against Clara's
;; rendering of the identical workload — no record/keyword shape to reconcile.
;;
;; Usage (stdin = an i64 vector [groups readings]; stdout = one #grid/Result EDN line):
;;   echo '[100 200]' | cargo wat ./wat-scripts/perf/grid/accum.wat
;;   => #grid/Result {:axis "accum" :size [100 200] :derived [...] :native-ns N}

(:wat::core::defrecord :acc::Group   [g <- :wat::core::i64])
(:wat::core::defrecord :acc::Reading [g <- :wat::core::i64  v <- :wat::core::i64])
(:wat::core::defrecord :acc::CountF  [g <- :wat::core::i64  n <- :wat::core::i64])
(:wat::core::defrecord :acc::SumF    [g <- :wat::core::i64  n <- :wat::core::i64])
(:wat::core::defrecord :acc::MinF    [g <- :wat::core::i64  n <- :wat::core::i64])
(:wat::core::defrecord :acc::MaxF    [g <- :wat::core::i64  n <- :wat::core::i64])
(:wat::core::defrecord :acc::ExistsF [g <- :wat::core::i64])

(:wat::core::defrecord :grid::Result
  [axis      <- :wat::core::String
   size      <- (:wat::core::PersistentVector :- [:wat::core::i64])
   derived   <- (:wat::core::PersistentVector :- [:wat::core::i64])
   native-ns      <- :wat::core::i64
   ;; THREE-WAY: the wat SPEC's own answer, so the runner can render :oracle-accuracy
   ;; (spec vs Clara) and :port-accuracy (spec vs native) instead of one verdict.
   oracle-derived <- (:wat::core::PersistentVector :- [:wat::core::i64])
   oracle-ns      <- :wat::core::i64
   insert-ns      <- :wat::core::i64
   fire-ns        <- :wat::core::i64
   query-ns       <- :wat::core::i64
   protocol-ns    <- :wat::core::i64])

;; ─── the five accumulate/exists rules (fixed structure; only the FACTS scale) ───
;; Structure mirrors the 8a/8b probe rule exactly: [anchor] [?n <- (acc) :from …] => insert.
(:wat::rete::defrule :acc::count-rule
  :when
  [(:acc::Group (?g <- :g))
   (?n <- (:wat::rete::acc::count) :from (:acc::Reading (?g <- :g)))]
  :then
  [(:acc::CountF ?g ?n)])

(:wat::rete::defrule :acc::sum-rule
  :when
  [(:acc::Group (?g <- :g))
   (?n <- (:wat::rete::acc::sum ?v) :from (:acc::Reading (?g <- :g) (?v <- :v)))]
  :then
  [(:acc::SumF ?g ?n)])

(:wat::rete::defrule :acc::min-rule
  :when
  [(:acc::Group (?g <- :g))
   (?n <- (:wat::rete::acc::min ?v) :from (:acc::Reading (?g <- :g) (?v <- :v)))]
  :then
  [(:acc::MinF ?g ?n)])

(:wat::rete::defrule :acc::max-rule
  :when
  [(:acc::Group (?g <- :g))
   (?n <- (:wat::rete::acc::max ?v) :from (:acc::Reading (?g <- :g) (?v <- :v)))]
  :then
  [(:acc::MaxF ?g ?n)])

(:wat::rete::defrule :acc::exists-rule
  :when
  [(:acc::Group (?g <- :g))
   (:wat::rete::exists (:acc::Reading (?g <- :g)))]
  :then
  [(:acc::ExistsF ?g)])

(:wat::rete::defquery :acc::q-CountF
  :params []
  :when [(?fact <- :acc::CountF)])


(:wat::rete::defquery :acc::q-SumF
  :params []
  :when [(?fact <- :acc::SumF)])


(:wat::rete::defquery :acc::q-MinF
  :params []
  :when [(?fact <- :acc::MinF)])


(:wat::rete::defquery :acc::q-MaxF
  :params []
  :when [(?fact <- :acc::MaxF)])


(:wat::rete::defquery :acc::q-ExistsF
  :params []
  :when [(?fact <- :acc::ExistsF)])


;; val g j — the deterministic reading value at (group g, index j): (g*31 + j*17) mod 1000.
;; No i64::mod op exists (only +,-,*,/), so mod is manual: x - (x/1000)*1000 (x>=0, truncating /).
;; The IDENTICAL fn runs on the Clara side (gen-accum.sh uses (mod (+ (* g 31) (* j 17)) 1000)),
;; so both engines fold byte-identical Reading facts.
(:wat::core::defn :acc::val [g <- :wat::core::i64  j <- :wat::core::i64] -> :wat::core::i64
  (:wat::core::let [x (:wat::core::i64::+ (:wat::core::i64::* g 31) (:wat::core::i64::* j 17))]
    (:wat::core::i64::- x (:wat::core::i64::* (:wat::core::i64::/ x 1000) 1000))))

;; enc kind g val — canonical single-i64 witness for one derived fact.
;; kind*1e15 + g*1e9 + val. g < 1e6 and val < ~2e6 at grid scale ⇒ injective, no i64 overflow.
(:wat::core::defn :acc::enc [kind <- :wat::core::i64  g <- :wat::core::i64  val <- :wat::core::i64]
  -> :wat::core::i64
  (:wat::core::i64::+
    (:wat::core::i64::+ (:wat::core::i64::* kind 1000000000000000) (:wat::core::i64::* g 1000000000))
    val))

;; vec->pvec v — materialize a (Vector :- [i64]) into a (PersistentVector :- [i64]). DESIGN-STONE-into-pv-
;; from-vector.md: `into` now has a native ((PersistentVector :- [T]), (Vector :- [T])) clause backed by one
;; `PersistentVector/concat` call — retiring the N-interpreted-closure-invocation conj-fold.
(:wat::core::defn :acc::vec->pvec [v <- (:wat::core::Vector :- [:wat::core::i64])] -> (:wat::core::PersistentVector :- [:wat::core::i64])
  (:wat::core::into (:wat::core::PersistentVector) v))

;; seed-readings session g W — stage Reading(g, val(g,j)) for j in [0, W), threading the session.
;; reading-facts acc g W — group g's W Readings, appended to a FACT VECTOR. No longer threads a
;; Session: staging is one BATCH `insert-all` at the end of `seed`.
(:wat::core::defn :acc::reading-facts
  [acc <- (:wat::core::PersistentVector :- [:wat::core::Record])  g <- :wat::core::i64  W <- :wat::core::i64]
  -> (:wat::core::PersistentVector :- [:wat::core::Record])
  (:wat::core::foldl
    (:wat::core::fn [a <- (:wat::core::PersistentVector :- [:wat::core::Record])  j <- :wat::core::i64]
                    -> (:wat::core::PersistentVector :- [:wat::core::Record])
      (:wat::core::PersistentVector/conj a (:acc::Reading :g g :v (:acc::val g j))))
    acc
    (:wat::core::range 0 W)))

;; all-facts G W — Group(g) + its W Readings for every g. Construct only; insert is timed
;; separately so protocol-ns is load+fire+query, not record allocation.
(:wat::core::defn :acc::all-facts [G <- :wat::core::i64  W <- :wat::core::i64] -> (:wat::core::PersistentVector :- [:wat::core::Record])
  (:wat::core::foldl
    (:wat::core::fn [acc <- (:wat::core::PersistentVector :- [:wat::core::Record])  g <- :wat::core::i64]
                    -> (:wat::core::PersistentVector :- [:wat::core::Record])
      (:acc::reading-facts (:wat::core::PersistentVector/conj acc (:acc::Group g)) g W))
    (:wat::core::PersistentVector)
    (:wat::core::range 0 G)))

;; seed session G W — stage Group(g) + its W Readings for every g in [0, G).
(:wat::core::defn :acc::seed [session <- :wat::rete::Session  G <- :wat::core::i64  W <- :wat::core::i64] -> :wat::rete::Session
  (:wat::rete::insert-all session (:acc::all-facts G W)))

;; codes fired — every derived fact across all five types, canonically encoded, into a (Vector :- [i64]).
;; Only five fixed types ⇒ no dispatch: five direct query+map+encode blocks folded into one Vector.
(:wat::core::defn :acc::codes [fired <- :wat::rete::Session] -> (:wat::core::Vector :- [:wat::core::i64])
  (:wat::core::let
    [c0 (:wat::core::into (:wat::core::Vector :wat::core::i64)
          (:wat::core::map (:wat::core::fn [p <- :wat::core::PersistentMap] -> :wat::core::i64 (:wat::core::let [f (:wat::core::Option/expect (:wat::core::PersistentMap/get p "?fact") "query: ?fact")] (:acc::enc 0 (:acc::CountF/g f) (:acc::CountF/n f))))
            (:wat::rete::query fired (:acc::q-CountF))))
     c1 (:wat::core::into c0
          (:wat::core::map (:wat::core::fn [p <- :wat::core::PersistentMap] -> :wat::core::i64 (:wat::core::let [f (:wat::core::Option/expect (:wat::core::PersistentMap/get p "?fact") "query: ?fact")] (:acc::enc 1 (:acc::SumF/g f) (:acc::SumF/n f))))
            (:wat::rete::query fired (:acc::q-SumF))))
     c2 (:wat::core::into c1
          (:wat::core::map (:wat::core::fn [p <- :wat::core::PersistentMap] -> :wat::core::i64 (:wat::core::let [f (:wat::core::Option/expect (:wat::core::PersistentMap/get p "?fact") "query: ?fact")] (:acc::enc 2 (:acc::MinF/g f) (:acc::MinF/n f))))
            (:wat::rete::query fired (:acc::q-MinF))))
     c3 (:wat::core::into c2
          (:wat::core::map (:wat::core::fn [p <- :wat::core::PersistentMap] -> :wat::core::i64 (:wat::core::let [f (:wat::core::Option/expect (:wat::core::PersistentMap/get p "?fact") "query: ?fact")] (:acc::enc 3 (:acc::MaxF/g f) (:acc::MaxF/n f))))
            (:wat::rete::query fired (:acc::q-MaxF))))
     c4 (:wat::core::into c3
          (:wat::core::map (:wat::core::fn [p <- :wat::core::PersistentMap] -> :wat::core::i64 (:wat::core::let [f (:wat::core::Option/expect (:wat::core::PersistentMap/get p "?fact") "query: ?fact")] (:acc::enc 4 (:acc::ExistsF/g f) 0)))
            (:wat::rete::query fired (:acc::q-ExistsF))))]
    c4))

;; derived-vector fired — the sorted i64 accuracy witness (the full set, not a count).
(:wat::core::defn :acc::derived-vector [fired <- :wat::rete::Session] -> (:wat::core::PersistentVector :- [:wat::core::i64])
  (:acc::vec->pvec (:wat::core::sort (:acc::codes fired))))

;; ns-between t0 t1 — nanoseconds between two Instants (mirrors strat-neg.wat's ns-between).
(:wat::core::defn :acc::ns-between [t0 <- :wat::time::Instant  t1 <- :wat::time::Instant] -> :wat::core::i64
  (:wat::core::i64::- (:wat::time::epoch-nanos t1) (:wat::time::epoch-nanos t0)))

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::let [params  (:wat::core::match (:wat::kernel::readln ) ((:wat::kernel::ReadlnOutcome::Datum __datum) __datum) (:wat::kernel::ReadlnOutcome::Eof (:wat::kernel::assertion-failed! "readln: end of input" :wat::core::None :wat::core::None)) (:wat::kernel::ReadlnOutcome::Stopped (:wat::kernel::assertion-failed! "readln: stop requested" :wat::core::None :wat::core::None)))
                    groups  (:wat::core::Option/expect  (:wat::core::get params 0) "stdin: [groups readings]")
                    reads   (:wat::core::Option/expect  (:wat::core::get params 1) "stdin: [groups readings]")
                    rules   (:wat::rete::collect-rules :acc)
                    session (:wat::rete::compile-all rules (:wat::core::PersistentVector (:acc::q-CountF) (:acc::q-SumF) (:acc::q-MinF) (:acc::q-MaxF) (:acc::q-ExistsF)))
                    facts   (:acc::all-facts groups reads)
                    ;; protocol: insert + fire + query. Compile and fact-construct are setup.
                    p0      (:wat::time::now)
                    staged  (:wat::rete::insert-all session facts)
                    i1      (:wat::time::now)
                    fired   (:wat::core::match (:wat::rete::fire-rules staged) ((:wat::rete::FireOutcome::Fired __fired) __fired) ((:wat::rete::FireOutcome::MemoryCeilingExceeded __limit __used __rounds) (:wat::kernel::assertion-failed! "fire-rules: session memory ceiling exceeded" :wat::core::None :wat::core::None)) ((:wat::rete::FireOutcome::RoundCapExceeded __cap __still) (:wat::kernel::assertion-failed! "fire-rules: fixpoint round cap exceeded" :wat::core::None :wat::core::None)))
                    f1      (:wat::time::now)
                    derived (:acc::derived-vector fired)
                    q1      (:wat::time::now)
                    ins-ns  (:acc::ns-between p0 i1)
                    fir-ns  (:acc::ns-between i1 f1)
                    qry-ns  (:acc::ns-between f1 q1)
                    proto-ns (:acc::ns-between p0 q1)
                    ;; ORACLE — fired on the SAME staged session. Value semantics make the
                    ;; two fires independent: `staged` is unchanged by either.
                    o0      (:wat::time::now)
                    ofired  (:wat::core::match (:wat::rete::fire-rules$oracle staged) ((:wat::rete::FireOutcome::Fired __fired) __fired) ((:wat::rete::FireOutcome::MemoryCeilingExceeded __limit __used __rounds) (:wat::kernel::assertion-failed! "fire-rules: session memory ceiling exceeded" :wat::core::None :wat::core::None)) ((:wat::rete::FireOutcome::RoundCapExceeded __cap __still) (:wat::kernel::assertion-failed! "fire-rules: fixpoint round cap exceeded" :wat::core::None :wat::core::None)))
                    o1      (:wat::time::now)]
    (:wat::kernel::println
      (:grid::Result :axis "accum" :size (:wat::core::PersistentVector groups reads) :derived derived :native-ns fir-ns :oracle-derived (:acc::derived-vector ofired) :oracle-ns (:acc::ns-between o0 o1) :insert-ns ins-ns :fire-ns fir-ns :query-ns qry-ns :protocol-ns proto-ns))))
