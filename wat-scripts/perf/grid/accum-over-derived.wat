;; wat-scripts/perf/grid/accum-over-derived.wat — CORRECTNESS AXIS: acc :from a type THIS SET derives.
;;
;;   Seed;  Step(0);  Step(k) :- Step(k-1) for k in [1,depth];
;;   Tally(n) :- Seed AND [?n <- (acc::count) :from Step]
;;
;; Native puts Tally at stratum 1 (Step is derived) and evaluates it ONCE, after Step closes.
;; The oracle puts it at stratum 0 and re-evaluates each round, so `depth` intermediate
;; tallies are created and must be SUPERSEDED away.
;;
;; ⛔ NOT "nothing covers this shape." `probe_arc278_oracle_accumulate_supersedes` bags a
;; derived type across empty / 0→1 / 0→1→2 and is GREEN. That probe pins n ≤ 2. This axis
;; is DEPTH and a LIVE referee (Clara on every check). 0 of 13 prior accumulate axes bag
;; a type the same rule set derives (anchored on derived_exists_acc + L2-3 scratch).
;;
;; ⛔ ONE CORRECTNESS SIZE, not a sweep. No `gen-accum-over-derived.sh` — a generator would
;; drag a correctness proof onto the perf ladder (`run-all.sh:81-87`, `check-grid-speed.sh`
;; treats :accuracy :MISMATCH as a gate failure). Static `accum-over-derived.clj`.
;;
;; Depth is a stated choice: 9. Past the probe's two intermediate states (4.5×). If the
;; oracle does not terminate, hits the round cap, or takes minutes here, that is STOP-3
;; — do not shrink until it is quiet.
;;
;; :derived = sorted, NOT deduped: enc(0,k,0) per derived Step level k in [1,depth]
;; plus enc(1,0,n) per Tally. A leaked intermediate tally is an EXTRA ELEMENT.
;; Expected count = depth + 1, re-derived from this formula, not read off a run.
;; Tally n = count of ALL Step facts including seeded Step(0) = depth + 1.
;;
;; The Seed condition is the ANCHOR, deliberately — a leading accumulate would braid
;; conferre L2-1 into this measurement.
;;
;; Variable MUST be named `staged` so GRID_SKIP_ORACLE / axes_live rewrite
;; `fire-rules$oracle staged`.
;;
;; Usage (stdin = an i64 vector [depth]; stdout = one #grid/Result EDN line):
;;   echo '[9]' | cargo wat ./wat-scripts/perf/grid/accum-over-derived.wat

(:wat::core::defrecord :aod::Seed  [id <- :wat::core::i64])
(:wat::core::defrecord :aod::Step  [level <- :wat::core::i64])
(:wat::core::defrecord :aod::Tally [n <- :wat::core::i64])

(:wat::core::defrecord :grid::Result
  [axis      <- :wat::core::String
   size      <- (:wat::core::PersistentVector :- [:wat::core::i64])
   derived   <- (:wat::core::PersistentVector :- [:wat::core::i64])
   native-ns      <- :wat::core::i64
   oracle-derived <- (:wat::core::PersistentVector :- [:wat::core::i64])
   oracle-ns      <- :wat::core::i64])

(:wat::rete::defquery :aod::q-Step
  :params []
  :when [(?fact <- :aod::Step)])


(:wat::rete::defquery :aod::q-Tally
  :params []
  :when [(?fact <- :aod::Tally)])


;; build-step k — Step(k) :- Step(k-1). Level literals spliced via quasiquote, same as
;; deep-cascade's per-level rule. One generated rule per level, not a self-recursive where.
(:wat::core::defn :aod::build-step [k <- :wat::core::i64] -> :wat::rete::Rule
  (:wat::core::let [prev (:wat::core::i64::- k 1)
                    c (:wat::core::quasiquote (:aod::Step (?l <- :level) (:wat::rete::core::i64::= ?l (:wat::core::unquote prev))))
                    t (:wat::core::quasiquote (:aod::Step (:wat::core::unquote k)))]
    (:wat::rete::Rule :name (:wat::core::i64::to-string k)
      :lhs (:wat::core::PersistentVector c)
      :rhs (:wat::core::PersistentVector t))))

;; tally — Seed AND count of every Step. Seed is the left token so this is not a leading accumulate.
(:wat::core::defn :aod::tally-rule [] -> :wat::rete::Rule
  (:wat::rete::Rule :name "tally"
    :lhs (:wat::core::PersistentVector
      (:wat::core::quote (:aod::Seed (?id <- :id)))
      (:wat::core::quote (?n <- (:wat::rete::acc::count) :from (:aod::Step))))
    :rhs (:wat::core::PersistentVector
      (:wat::core::quote (:aod::Tally ?n)))))

;; build-rules depth — tally plus one Step(k):-Step(k-1) per k in [1,depth].
(:wat::core::defn :aod::build-rules [depth <- :wat::core::i64] -> (:wat::core::PersistentVector :- [:wat::rete::Rule])
  (:wat::core::foldl
    (:wat::core::fn [acc <- (:wat::core::PersistentVector :- [:wat::rete::Rule])  k <- :wat::core::i64]
                    -> (:wat::core::PersistentVector :- [:wat::rete::Rule])
      (:wat::core::PersistentVector/conj acc (:aod::build-step k)))
    (:wat::core::PersistentVector (:aod::tally-rule))
    (:wat::core::range 1 (:wat::core::i64::+ depth 1))))

;; Empty (PersistentVector :- [Record]) so Seed and Step can share one batch.
;; A two-element literal infers from the first element and refuses the second
;; (homogeneous PV; same check-time refusal insert-all has for mixed Records).
(:wat::core::defn :aod::empty-records [] -> (:wat::core::PersistentVector :- [:wat::core::Record])
  (:wat::core::PersistentVector))

(:wat::core::defn :aod::seed-facts [] -> (:wat::core::PersistentVector :- [:wat::core::Record])
  (:wat::core::PersistentVector/conj
    (:wat::core::PersistentVector/conj
      (:aod::empty-records)
      (:aod::Seed :id 0))
    (:aod::Step :level 0)))

(:wat::core::defn :aod::seed [session <- :wat::rete::Session] -> :wat::rete::Session
  (:wat::core::match (:wat::rete::insert-all session (:aod::seed-facts))
    ((:wat::rete::InsertOutcome::Inserted __staged) __staged)
    ((:wat::rete::InsertOutcome::MemoryCeilingExceeded __limit __used __count)
     (:wat::kernel::assertion-failed! "insert: session memory ceiling exceeded while staging" :wat::core::None :wat::core::None))))

(:wat::core::defn :aod::fire [s <- :wat::rete::Session] -> :wat::rete::Session
  (:wat::core::match (:wat::rete::fire-rules s)
    ((:wat::rete::FireOutcome::Fired __fired) __fired)
    ((:wat::rete::FireOutcome::MemoryCeilingExceeded __limit __used __rounds)
     (:wat::kernel::assertion-failed! "fire-rules: session memory ceiling exceeded" :wat::core::None :wat::core::None))
    ((:wat::rete::FireOutcome::RoundCapExceeded __cap __still)
     (:wat::kernel::assertion-failed! "fire-rules: fixpoint round cap exceeded" :wat::core::None :wat::core::None))))

(:wat::core::defn :aod::enc [kind <- :wat::core::i64  level <- :wat::core::i64  id <- :wat::core::i64] -> :wat::core::i64
  (:wat::core::i64::+
    (:wat::core::i64::+ (:wat::core::i64::* kind 1000000000000000) (:wat::core::i64::* level 1000000000))
    id))

(:wat::core::defn :aod::vec->pvec [v <- (:wat::core::Vector :- [:wat::core::i64])] -> (:wat::core::PersistentVector :- [:wat::core::i64])
  (:wat::core::into (:wat::core::PersistentVector) v))

;; derived-vector — sorted, NOT deduped. Derived Step levels (level > 0) plus every Tally.
;; A leaked intermediate tally is an extra enc(1,0,k) next to the surviving one.
(:wat::core::defn :aod::derived-vector [fired <- :wat::rete::Session] -> (:wat::core::PersistentVector :- [:wat::core::i64])
  (:wat::core::let
    [c0 (:wat::core::into (:wat::core::Vector :wat::core::i64)
          (:wat::core::map
            (:wat::core::fn [p <- :wat::core::PersistentMap] -> :wat::core::i64
              (:wat::core::let [f (:wat::core::Option/expect (:wat::core::PersistentMap/get p "?fact") "query: ?fact")]
                (:aod::enc 0 (:aod::Step/level f) 0)))
            (:wat::core::filter
              (:wat::core::fn [p <- :wat::core::PersistentMap] -> :wat::core::bool
                (:wat::core::let [f (:wat::core::Option/expect (:wat::core::PersistentMap/get p "?fact") "query: ?fact")]
                  (:wat::core::i64::> (:aod::Step/level f) 0)))
              (:wat::rete::query fired (:aod::q-Step)))))
     c1 (:wat::core::into c0
          (:wat::core::map
            (:wat::core::fn [p <- :wat::core::PersistentMap] -> :wat::core::i64
              (:wat::core::let [f (:wat::core::Option/expect (:wat::core::PersistentMap/get p "?fact") "query: ?fact")]
                (:aod::enc 1 0 (:aod::Tally/n f))))
            (:wat::rete::query fired (:aod::q-Tally))))]
    (:aod::vec->pvec (:wat::core::sort c1))))

(:wat::core::defn :aod::ns-between [t0 <- :wat::time::Instant  t1 <- :wat::time::Instant] -> :wat::core::i64
  (:wat::core::i64::- (:wat::time::epoch-nanos t1) (:wat::time::epoch-nanos t0)))

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::let [params  (:wat::core::match (:wat::kernel::readln ) ((:wat::kernel::ReadlnOutcome::Datum __datum) __datum) (:wat::kernel::ReadlnOutcome::Eof (:wat::kernel::assertion-failed! "readln: end of input" :wat::core::None :wat::core::None)) (:wat::kernel::ReadlnOutcome::Stopped (:wat::kernel::assertion-failed! "readln: stop requested" :wat::core::None :wat::core::None)))
                    depth   (:wat::core::Option/expect (:wat::core::get params 0) "stdin: [depth]")
                    rules   (:aod::build-rules depth)
                    session (:wat::core::match (:wat::rete::compile-all rules (:wat::core::PersistentVector (:aod::q-Step) (:aod::q-Tally))) ((:wat::rete::CompileOutcome::Compiled __session) __session) ((:wat::rete::CompileOutcome::MayNotTerminate __rule __fact-type) (:wat::kernel::assertion-failed! "compile: the rule set may not terminate" :wat::core::None :wat::core::None)))
                    ;; Variable MUST be named `staged` so GRID_SKIP_ORACLE / axes_live rewrite
                    ;; `fire-rules$oracle staged` and not a first-fire leftover.
                    staged  (:aod::seed session)
                    n0      (:wat::time::now)
                    fired   (:aod::fire staged)
                    n1      (:wat::time::now)
                    derived (:aod::derived-vector fired)
                    nat-ns  (:aod::ns-between n0 n1)
                    o0      (:wat::time::now)
                    ofired  (:wat::core::match (:wat::rete::fire-rules$oracle staged) ((:wat::rete::FireOutcome::Fired __fired) __fired) ((:wat::rete::FireOutcome::MemoryCeilingExceeded __limit __used __rounds) (:wat::kernel::assertion-failed! "fire-rules: session memory ceiling exceeded" :wat::core::None :wat::core::None)) ((:wat::rete::FireOutcome::RoundCapExceeded __cap __still) (:wat::kernel::assertion-failed! "fire-rules: fixpoint round cap exceeded" :wat::core::None :wat::core::None)))
                    o1      (:wat::time::now)]
    (:wat::kernel::println
      (:grid::Result :axis "accum-over-derived" :size (:wat::core::PersistentVector depth) :derived derived :native-ns nat-ns :oracle-derived (:aod::derived-vector ofired) :oracle-ns (:aod::ns-between o0 o1)))))
