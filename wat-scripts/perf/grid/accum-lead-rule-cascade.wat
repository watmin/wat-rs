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

(:wat::core::defrecord :alrc::Reading [v <- :wat::core::i64])
(:wat::core::defrecord :alrc::Anchor  [k <- :wat::core::i64])
(:wat::core::defrecord :alrc::Link    [level <- :wat::core::i64])
(:wat::core::defrecord :alrc::Busy    [k <- :wat::core::i64  n <- :wat::core::i64])

(:wat::core::defrecord :grid::Result
  [axis      <- :wat::core::String
   size      <- (:wat::core::PersistentVector :- [:wat::core::i64])
   derived   <- (:wat::core::PersistentVector :- [:wat::core::i64])
   native-ns      <- :wat::core::i64
   oracle-derived <- (:wat::core::PersistentVector :- [:wat::core::i64])
   oracle-ns      <- :wat::core::i64])

(:wat::rete::defquery :alrc::q-Busy
  :params []
  :when [(?fact <- :alrc::Busy)])


;; Link(k) :- Link(k-1). Generated per depth; Busy never mentions Link.
(:wat::core::defn :alrc::build-link [k <- :wat::core::i64] -> :wat::rete::Rule
  (:wat::core::let [prev (:wat::core::i64::- k 1)
                    c (:wat::core::quasiquote (:alrc::Link (?l <- :level) (:wat::rete::core::i64::= ?l (:wat::core::unquote prev))))
                    t (:wat::core::quasiquote (:alrc::Link (:wat::core::unquote k)))]
    (:wat::rete::Rule :name (:wat::core::i64::to-string k)
      :lhs (:wat::core::PersistentVector c)
      :rhs (:wat::core::PersistentVector t))))

;; ★ leading accumulate, then a join. The accumulate has no parent condition.
(:wat::core::defn :alrc::busy-rule [] -> :wat::rete::Rule
  (:wat::rete::Rule :name "busy"
    :lhs (:wat::core::PersistentVector
      (:wat::core::quote (?n <- (:wat::rete::acc::count) :from (:alrc::Reading)))
      (:wat::core::quote (:alrc::Anchor (?k <- :k))))
    :rhs (:wat::core::PersistentVector
      (:wat::core::quote (:alrc::Busy ?k ?n)))))

(:wat::core::defn :alrc::build-rules [depth <- :wat::core::i64] -> (:wat::core::PersistentVector :- [:wat::rete::Rule])
  (:wat::core::foldl
    (:wat::core::fn [acc <- (:wat::core::PersistentVector :- [:wat::rete::Rule])  k <- :wat::core::i64]
                    -> (:wat::core::PersistentVector :- [:wat::rete::Rule])
      (:wat::core::PersistentVector/conj acc (:alrc::build-link k)))
    (:wat::core::PersistentVector (:alrc::busy-rule))
    (:wat::core::range 1 (:wat::core::i64::+ depth 1))))

(:wat::core::defn :alrc::empty-records [] -> (:wat::core::PersistentVector :- [:wat::core::Record])
  (:wat::core::PersistentVector))

(:wat::core::defn :alrc::seed-facts [items <- :wat::core::i64  anchors <- :wat::core::i64]
  -> (:wat::core::PersistentVector :- [:wat::core::Record])
  (:wat::core::let
    [with-read (:wat::core::foldl
                  (:wat::core::fn [acc <- (:wat::core::PersistentVector :- [:wat::core::Record])  v <- :wat::core::i64]
                                  -> (:wat::core::PersistentVector :- [:wat::core::Record])
                    (:wat::core::PersistentVector/conj acc (:alrc::Reading :v v)))
                  (:alrc::empty-records)
                  (:wat::core::range 0 items))
     with-anch (:wat::core::foldl
                  (:wat::core::fn [acc <- (:wat::core::PersistentVector :- [:wat::core::Record])  k <- :wat::core::i64]
                                  -> (:wat::core::PersistentVector :- [:wat::core::Record])
                    (:wat::core::PersistentVector/conj acc (:alrc::Anchor :k k)))
                  with-read
                  (:wat::core::range 0 anchors))]
    (:wat::core::PersistentVector/conj with-anch (:alrc::Link :level 0))))

(:wat::core::defn :alrc::seed [session <- :wat::rete::Session  items <- :wat::core::i64  anchors <- :wat::core::i64] -> :wat::rete::Session
  (:wat::core::match (:wat::rete::insert-all session (:alrc::seed-facts items anchors))
    ((:wat::rete::InsertOutcome::Inserted __staged) __staged)
    ((:wat::rete::InsertOutcome::MemoryCeilingExceeded __limit __used __count)
     (:wat::kernel::assertion-failed! "insert: session memory ceiling exceeded while staging" :wat::core::None :wat::core::None))))

(:wat::core::defn :alrc::fire [s <- :wat::rete::Session] -> :wat::rete::Session
  (:wat::core::match (:wat::rete::fire-rules s)
    ((:wat::rete::FireOutcome::Fired __fired) __fired)
    ((:wat::rete::FireOutcome::MemoryCeilingExceeded __limit __used __rounds)
     (:wat::kernel::assertion-failed! "fire-rules: session memory ceiling exceeded" :wat::core::None :wat::core::None))
    ((:wat::rete::FireOutcome::RoundCapExceeded __cap __still)
     (:wat::kernel::assertion-failed! "fire-rules: fixpoint round cap exceeded" :wat::core::None :wat::core::None))))

(:wat::core::defn :alrc::enc [k <- :wat::core::i64  n <- :wat::core::i64] -> :wat::core::i64
  (:wat::core::i64::+ (:wat::core::i64::* k 1000000000000000) n))

(:wat::core::defn :alrc::vec->pvec [v <- (:wat::core::Vector :- [:wat::core::i64])] -> (:wat::core::PersistentVector :- [:wat::core::i64])
  (:wat::core::into (:wat::core::PersistentVector) v))

;; Busy only. Link is derived and unqueried — if it leaked in, the count would move with depth.
(:wat::core::defn :alrc::derived-vector [fired <- :wat::rete::Session] -> (:wat::core::PersistentVector :- [:wat::core::i64])
  (:wat::core::let [codes (:wat::core::into (:wat::core::Vector :wat::core::i64)
                            (:wat::core::map
                              (:wat::core::fn [p <- :wat::core::PersistentMap] -> :wat::core::i64
                                (:wat::core::let [f (:wat::core::Option/expect (:wat::core::PersistentMap/get p "?fact") "query: ?fact")]
                                  (:alrc::enc (:alrc::Busy/k f) (:alrc::Busy/n f))))
                              (:wat::rete::query fired (:alrc::q-Busy))))]
    (:alrc::vec->pvec (:wat::core::sort codes))))

(:wat::core::defn :alrc::ns-between [t0 <- :wat::time::Instant  t1 <- :wat::time::Instant] -> :wat::core::i64
  (:wat::core::i64::- (:wat::time::epoch-nanos t1) (:wat::time::epoch-nanos t0)))

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::let [params  (:wat::core::match (:wat::kernel::readln ) ((:wat::kernel::ReadlnOutcome::Datum __datum) __datum) (:wat::kernel::ReadlnOutcome::Eof (:wat::kernel::assertion-failed! "readln: end of input" :wat::core::None :wat::core::None)) (:wat::kernel::ReadlnOutcome::Stopped (:wat::kernel::assertion-failed! "readln: stop requested" :wat::core::None :wat::core::None)))
                    items   (:wat::core::Option/expect (:wat::core::get params 0) "stdin: [items anchors depth]")
                    anchors (:wat::core::Option/expect (:wat::core::get params 1) "stdin: [items anchors depth]")
                    depth   (:wat::core::Option/expect (:wat::core::get params 2) "stdin: [items anchors depth]")
                    rules   (:alrc::build-rules depth)
                    session (:wat::core::match (:wat::rete::compile-all rules (:wat::core::PersistentVector (:alrc::q-Busy))) ((:wat::rete::CompileOutcome::Compiled __session) __session) ((:wat::rete::CompileOutcome::MayNotTerminate __rule __fact-type) (:wat::kernel::assertion-failed! "compile: the rule set may not terminate" :wat::core::None :wat::core::None)))
                    ;; Variable MUST be named `staged` so GRID_SKIP_ORACLE / axes_live rewrite
                    ;; `fire-rules$oracle staged`.
                    staged  (:alrc::seed session items anchors)
                    n0      (:wat::time::now)
                    fired   (:alrc::fire staged)
                    n1      (:wat::time::now)
                    derived (:alrc::derived-vector fired)
                    nat-ns  (:alrc::ns-between n0 n1)
                    o0      (:wat::time::now)
                    ofired  (:wat::core::match (:wat::rete::fire-rules$oracle staged) ((:wat::rete::FireOutcome::Fired __fired) __fired) ((:wat::rete::FireOutcome::MemoryCeilingExceeded __limit __used __rounds) (:wat::kernel::assertion-failed! "fire-rules: session memory ceiling exceeded" :wat::core::None :wat::core::None)) ((:wat::rete::FireOutcome::RoundCapExceeded __cap __still) (:wat::kernel::assertion-failed! "fire-rules: fixpoint round cap exceeded" :wat::core::None :wat::core::None)))
                    o1      (:wat::time::now)]
    (:wat::kernel::println
      (:grid::Result :axis "accum-lead-rule-cascade" :size (:wat::core::PersistentVector items anchors depth) :derived derived :native-ns nat-ns :oracle-derived (:alrc::derived-vector ofired) :oracle-ns (:alrc::ns-between o0 o1)))))
