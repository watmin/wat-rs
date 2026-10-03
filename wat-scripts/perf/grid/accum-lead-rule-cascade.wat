;; wat-scripts/perf/grid/accum-lead-rule-cascade.wat — CORRECTNESS AXIS: the matrix's fourth cell.
;;
;;   Reading(v) for v in [0,items);  Anchor(k) for k in [0,anchors);
;;   Link(0) seeded; Link(k) :- Link(k-1) for k in [1,depth] — INERT cascade, read by NOTHING;
;;   Busy(k,n) :- [?n <- (acc::count) :from Reading] AND Anchor(k)
;;
;; where-accum-lead   leading accumulate, in a RULE,  no cascade   — covered, agrees
;; leading-exists     leading :exists,    in a RULE,  WITH cascade — covered, agrees
;; where-accum-lead-cascade  leading accumulate, in a QUERY, WITH cascade — covered, agrees
;; THIS AXIS          leading accumulate, in a RULE,  WITH cascade
;;
;; Found by the rete differential fuzzer (2026-08-25, family A): the row count tracked the
;; FIXPOINT ROUND COUNT exactly. A fix for one cell did not reach the other.
;; Measured 2026-09-07: 2 grid axes have a leading accumulate in a RULE; 0 of them have a cascade.
;;
;; ⛔ THE CASCADE IS INERT AND THAT IS THE WHOLE DESIGN. Link facts are derived and never
;; read by Busy or the query. The cascade's only job is more fixpoint rounds. So :derived
;; must be IDENTICAL at every depth. Any spread is the engine leaking its round count into
;; an answer — the signature the fuzzer caught in the sibling cells.
;;
;; ⛔ THE COUNT IS `anchors` AT EVERY CASCADE DEPTH. Constancy IS the assertion. Every other
;; axis's count tracks its dial; do not "fix" this constant into a formula of depth.
;; Non-vacuity is anchors > 0 plus the Clara | oracle | native three-way, not a moving count.
;;
;; ⛔ ONE CORRECTNESS SIZE, not a sweep. No `gen-accum-lead-rule-cascade.sh`. Static twin.
;;
;; :derived = sorted, NOT deduped: enc(k, n) per Busy. Expected count = anchors.
;; Size vector is [items anchors depth]. Stated choice: [3 2 3] → count 2.
;;
;; Variable MUST be named `staged` so GRID_SKIP_ORACLE / axes_live rewrite
;; `fire-rules$oracle staged`.
;;
;; Usage (stdin = [items anchors depth]; stdout = one #grid/Result EDN line):
;;   echo '[3 2 3]' | cargo run --release --bin wat -- ./wat-scripts/perf/grid/accum-lead-rule-cascade.wat

(:wat::core::defrecord :alrc::Reading [v <- wat.type/i64])
(:wat::core::defrecord :alrc::Anchor  [k <- wat.type/i64])
(:wat::core::defrecord :alrc::Link    [level <- wat.type/i64])
(:wat::core::defrecord :alrc::Busy    [k <- wat.type/i64  n <- wat.type/i64])

(:wat::core::defrecord :grid::Result
  [axis      <- wat.type/String
   size      <- (wat.type/PersistentVector :- [wat.type/i64])
   derived   <- (wat.type/PersistentVector :- [wat.type/i64])
   native-ns      <- wat.type/i64
   oracle-derived <- (wat.type/PersistentVector :- [wat.type/i64])
   oracle-ns      <- wat.type/i64])

(:wat::rete::defquery :alrc::q-Busy
  :params []
  :when [(?fact :- :alrc::Busy)])


;; Link(k) :- Link(k-1). Generated per depth; Busy never mentions Link.
(:wat::core::defn :alrc::build-link [k <- wat.type/i64] -> :wat::rete::Rule
  (:wat::core::let [prev (:wat::i64::- k 1)
                    c (:wat::core::quasiquote (:alrc::Link (?l :- :level) (:wat::rete::i64::= ?l (:wat::core::unquote prev))))
                    t (:wat::core::quasiquote (:alrc::Link (:wat::core::unquote k)))]
    (:wat::rete::Rule :name (:wat::i64::to-string k)
      :lhs (wat.type/PersistentVector :- [wat.type/AST] c)
      :rhs (wat.type/PersistentVector :- [wat.type/AST] t))))

;; ★ leading accumulate, then a join. The accumulate has no parent condition.
(:wat::core::defn :alrc::busy-rule [] -> :wat::rete::Rule
  (:wat::rete::Rule :name "busy"
    :lhs (wat.type/PersistentVector :- [wat.type/AST]
      (:wat::core::quote (?n :- (:wat::rete::acc::count) :from (:alrc::Reading)))
      (:wat::core::quote (:alrc::Anchor (?k :- :k))))
    :rhs (wat.type/PersistentVector :- [wat.type/AST]
      (:wat::core::quote (:alrc::Busy ?k ?n)))))

(:wat::core::defn :alrc::build-rules [depth <- wat.type/i64] -> (wat.type/PersistentVector :- [:wat::rete::Rule])
  (:wat::core::foldl
    (:wat::core::fn [acc <- (wat.type/PersistentVector :- [:wat::rete::Rule])  k <- wat.type/i64]
                    -> (wat.type/PersistentVector :- [:wat::rete::Rule])
      (:wat::core::conj acc (:alrc::build-link k)))
    (wat.type/PersistentVector :- [:wat::rete::Rule] (:alrc::busy-rule))
    (:wat::core::range 1 (:wat::i64::+ depth 1))))

(:wat::core::defn :alrc::empty-records [] -> (wat.type/PersistentVector :- [wat.type/Record])
  (wat.type/PersistentVector :- [wat.type/Record]))

(:wat::core::defn :alrc::seed-facts [items <- wat.type/i64  anchors <- wat.type/i64]
  -> (wat.type/PersistentVector :- [wat.type/Record])
  (:wat::core::let
    [with-read (:wat::core::foldl
                  (:wat::core::fn [acc <- (wat.type/PersistentVector :- [wat.type/Record])  v <- wat.type/i64]
                                  -> (wat.type/PersistentVector :- [wat.type/Record])
                    (:wat::core::conj acc (:alrc::Reading :v v)))
                  (:alrc::empty-records)
                  (:wat::core::range 0 items))
     with-anch (:wat::core::foldl
                  (:wat::core::fn [acc <- (wat.type/PersistentVector :- [wat.type/Record])  k <- wat.type/i64]
                                  -> (wat.type/PersistentVector :- [wat.type/Record])
                    (:wat::core::conj acc (:alrc::Anchor :k k)))
                  with-read
                  (:wat::core::range 0 anchors))]
    (:wat::core::conj with-anch (:alrc::Link :level 0))))

(:wat::core::defn :alrc::seed [session <- :wat::rete::Session  items <- wat.type/i64  anchors <- wat.type/i64] -> :wat::rete::Session
  (:wat::core::match (:wat::rete::insert-all session (:alrc::seed-facts items anchors))
    [:wat::rete::InsertOutcome.Inserted {:session __staged} __staged]
    [:wat::rete::InsertOutcome.MemoryCeilingExceeded {:limit __limit :used __used :staged __count}
     (:wat::kernel::assertion-failed! :message "insert: session memory ceiling exceeded while staging")]))

(:wat::core::defn :alrc::fire [s <- :wat::rete::Session] -> :wat::rete::Session
  (:wat::core::match (:wat::rete::fire-rules s)
    [:wat::rete::FireOutcome.Fired {:value __fired} __fired]
    [:wat::rete::FireOutcome.MemoryCeilingExceeded {:limit __limit :used __used :rounds __rounds}
     (:wat::kernel::assertion-failed! :message "fire-rules: session memory ceiling exceeded")]
    [:wat::rete::FireOutcome.RoundCapExceeded {:cap __cap :still-deriving __still}
     (:wat::kernel::assertion-failed! :message "fire-rules: fixpoint round cap exceeded")]))

(:wat::core::defn :alrc::enc [k <- wat.type/i64  n <- wat.type/i64] -> wat.type/i64
  (:wat::i64::+ (:wat::i64::* k 1000000000000000) n))

(:wat::core::defn :alrc::vec->pvec [v <- (wat.type/Vector :- [wat.type/i64])] -> (wat.type/PersistentVector :- [wat.type/i64])
  (:wat::core::into (wat.type/PersistentVector :- [wat.type/i64]) v))

;; Busy only. Link is derived and unqueried — if it leaked in, the count would move with depth.
(:wat::core::defn :alrc::derived-vector [fired <- :wat::rete::Session] -> (wat.type/PersistentVector :- [wat.type/i64])
  (:wat::core::let [codes (:wat::core::into (wat.type/Vector :- [wat.type/i64])
                            (:wat::core::map
                              (:wat::core::fn [p <- wat.type/PersistentMap] -> wat.type/i64
                                (:wat::core::let [f (:wat::core::Option/expect (:wat::core::get p "?fact") "query: ?fact")]
                                  (:alrc::enc (:alrc::Busy/k f) (:alrc::Busy/n f))))
                              (:wat::rete::query fired (:alrc::q-Busy))))]
    (:alrc::vec->pvec (:wat::core::sort codes))))

(:wat::core::defn :alrc::ns-between [t0 <- :wat::time::Instant  t1 <- :wat::time::Instant] -> wat.type/i64
  (:wat::i64::- (:wat::time::epoch-nanos t1) (:wat::time::epoch-nanos t0)))

(:wat::core::defn :user::main [] -> wat.type/nil
  (:wat::core::let [params  (:wat::core::match (:wat::kernel::readln ) [:wat::kernel::ReadlnOutcome.Datum {:v __datum} __datum] [:wat::kernel::ReadlnOutcome.Eof {} (:wat::kernel::assertion-failed! :message "readln: end of input")] [:wat::kernel::ReadlnOutcome.Stopped {} (:wat::kernel::assertion-failed! :message "readln: stop requested")])
                    items   (:wat::core::Option/expect (:wat::core::get params 0) "stdin: [items anchors depth]")
                    anchors (:wat::core::Option/expect (:wat::core::get params 1) "stdin: [items anchors depth]")
                    depth   (:wat::core::Option/expect (:wat::core::get params 2) "stdin: [items anchors depth]")
                    rules   (:alrc::build-rules depth)
                    session (:wat::core::match (:wat::rete::compile-all rules (wat.type/PersistentVector :- [:wat::rete::Query] (:alrc::q-Busy))) [:wat::rete::CompileOutcome.Compiled {:session __session} __session] [:wat::rete::CompileOutcome.MayNotTerminate {:rule __rule :fact-type __fact-type} (:wat::kernel::assertion-failed! :message "compile: the rule set may not terminate")])
                    ;; Variable MUST be named `staged` so GRID_SKIP_ORACLE / axes_live rewrite
                    ;; `fire-rules$oracle staged`.
                    staged  (:alrc::seed session items anchors)
                    n0      (:wat::time::now)
                    fired   (:alrc::fire staged)
                    n1      (:wat::time::now)
                    derived (:alrc::derived-vector fired)
                    nat-ns  (:alrc::ns-between n0 n1)
                    o0      (:wat::time::now)
                    ofired  (:wat::core::match (:wat::rete::fire-rules$oracle staged) [:wat::rete::FireOutcome.Fired {:value __fired} __fired] [:wat::rete::FireOutcome.MemoryCeilingExceeded {:limit __limit :used __used :rounds __rounds} (:wat::kernel::assertion-failed! :message "fire-rules: session memory ceiling exceeded")] [:wat::rete::FireOutcome.RoundCapExceeded {:cap __cap :still-deriving __still} (:wat::kernel::assertion-failed! :message "fire-rules: fixpoint round cap exceeded")])
                    o1      (:wat::time::now)]
    (:wat::kernel::println
      (:grid::Result :axis "accum-lead-rule-cascade" :size (wat.type/PersistentVector :- [wat.type/i64] items anchors depth) :derived derived :native-ns nat-ns :oracle-derived (:alrc::derived-vector ofired) :oracle-ns (:alrc::ns-between o0 o1)))))
