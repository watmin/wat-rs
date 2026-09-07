;; wat-scripts/perf/grid/userfn-head.wat — CORRECTNESS AXIS: the :then head is a USER FN,
;; not a record constructor.
;;
;;   Src(k) for k in [0,items);
;;   Bad :- Src, k = -1           (never fires — no key in [0,items) is -1; keeps the negation live)
;;   Rate :- Src(k), (not Bad(k)), :then [(mk-rate ?k)]   <- the user-fn head
;;   Out(n) :- Rate(n)
;;
;; Pre-cure the oracle assigned the stratum to `mk-rate`, left Rate at 0, and Out -- consuming
;; Rate -- sat below its producer, so Out was DROPPED. `21a5f8514` cured it.
;;
;; ⛔ NOT "nothing covers this shape." The scratch `arc278-produced-type-userfn-facts` drove it
;; at Src(1) and is GREEN post-cure. 0 of 46 prior grid axes have a user-fn :then head
;; (anchored on that scratch + probe_arc278_then_user_forms_userfn). The colon-strip was
;; CORRECT for all 46 — they construct with a record-type head, so the grid never saw the
;; flaw. This axis is that shape with a swept items dial and a LIVE Clara referee.
;;
;; ⛔ ONE CORRECTNESS SIZE, not a sweep. No `gen-userfn-head.sh` — a generator would drag a
;; correctness proof onto the perf ladder (`run-all.sh:81-87`, `check-grid-speed.sh` treats
;; :accuracy :MISMATCH as a gate failure). Static `userfn-head.clj`.
;;
;; Items is a stated choice: 5. Expected count = 2 * items = 10, re-derived from this
;; formula, not read off a run. :derived carries BOTH Rate and Out, sorted, NOT deduped:
;; enc(0,k) per Rate, enc(1,n) per Out. A witness carrying Out alone cannot tell
;; "Out dropped" from "nothing derived".
;;
;; mk-rate is `:wat::rete::core::defn` returning Rate, body exactly `(:ufh::Rate :count k)`
;; — a bound var, not a computed mint. A computed mint IS refused (rete_fn_body_mints).
;;
;; Variable MUST be named `staged` so GRID_SKIP_ORACLE / axes_live rewrite
;; `fire-rules$oracle staged`.
;;
;; Usage (stdin = an i64 vector [items]; stdout = one #grid/Result EDN line):
;;   echo '[5]' | cargo run --release --bin wat -- ./wat-scripts/perf/grid/userfn-head.wat

(:wat::core::defrecord :ufh::Src  [k <- :wat::core::i64])
(:wat::core::defrecord :ufh::Bad  [k <- :wat::core::i64])
(:wat::core::defrecord :ufh::Rate [count <- :wat::core::i64])
(:wat::core::defrecord :ufh::Out  [n <- :wat::core::i64])

(:wat::core::defrecord :grid::Result
  [axis      <- :wat::core::String
   size      <- (:wat::core::PersistentVector :- [:wat::core::i64])
   derived   <- (:wat::core::PersistentVector :- [:wat::core::i64])
   native-ns      <- :wat::core::i64
   oracle-derived <- (:wat::core::PersistentVector :- [:wat::core::i64])
   oracle-ns      <- :wat::core::i64])

(:wat::rete::core::defn :ufh::mk-rate
  [k <- :wat::core::i64]
  -> :ufh::Rate
  (:ufh::Rate :count k))

(:wat::rete::defrule :ufh::bad
  :when [(:ufh::Src (?k <- :k))
         (:wat::rete::where (:wat::rete::core::i64::= ?k -1))]
  :then [(:ufh::Bad :k ?k)])

(:wat::rete::defrule :ufh::via
  :when [(:ufh::Src (?k <- :k))
         (:wat::rete::not (:ufh::Bad (?k <- :k)))]
  :then [(:ufh::mk-rate ?k)])

(:wat::rete::defrule :ufh::out
  :when [(:ufh::Rate (?n <- :count))]
  :then [(:ufh::Out :n ?n)])

(:wat::rete::defquery :ufh::q-Rate
  :params []
  :when [(?fact <- :ufh::Rate)])


(:wat::rete::defquery :ufh::q-Out
  :params []
  :when [(?fact <- :ufh::Out)])


(:wat::core::defn :ufh::build-rules [] -> (:wat::core::PersistentVector :- [:wat::rete::Rule])
  (:wat::core::PersistentVector (:ufh::bad) (:ufh::via) (:ufh::out)))

(:wat::core::defn :ufh::empty-records [] -> (:wat::core::PersistentVector :- [:wat::core::Record])
  (:wat::core::PersistentVector))

(:wat::core::defn :ufh::seed-facts [items <- :wat::core::i64] -> (:wat::core::PersistentVector :- [:wat::core::Record])
  (:wat::core::foldl
    (:wat::core::fn [acc <- (:wat::core::PersistentVector :- [:wat::core::Record])  k <- :wat::core::i64]
                    -> (:wat::core::PersistentVector :- [:wat::core::Record])
      (:wat::core::PersistentVector/conj acc (:ufh::Src :k k)))
    (:ufh::empty-records)
    (:wat::core::range 0 items)))

(:wat::core::defn :ufh::seed [session <- :wat::rete::Session  items <- :wat::core::i64] -> :wat::rete::Session
  (:wat::core::match (:wat::rete::insert-all session (:ufh::seed-facts items))
    ((:wat::rete::InsertOutcome::Inserted __staged) __staged)
    ((:wat::rete::InsertOutcome::MemoryCeilingExceeded __limit __used __count)
     (:wat::kernel::assertion-failed! "insert: session memory ceiling exceeded while staging" :wat::core::None :wat::core::None))))

(:wat::core::defn :ufh::fire [s <- :wat::rete::Session] -> :wat::rete::Session
  (:wat::core::match (:wat::rete::fire-rules s)
    ((:wat::rete::FireOutcome::Fired __fired) __fired)
    ((:wat::rete::FireOutcome::MemoryCeilingExceeded __limit __used __rounds)
     (:wat::kernel::assertion-failed! "fire-rules: session memory ceiling exceeded" :wat::core::None :wat::core::None))
    ((:wat::rete::FireOutcome::RoundCapExceeded __cap __still)
     (:wat::kernel::assertion-failed! "fire-rules: fixpoint round cap exceeded" :wat::core::None :wat::core::None))))

(:wat::core::defn :ufh::enc [kind <- :wat::core::i64  id <- :wat::core::i64] -> :wat::core::i64
  (:wat::core::i64::+ (:wat::core::i64::* kind 1000000000000000) id))

(:wat::core::defn :ufh::vec->pvec [v <- (:wat::core::Vector :- [:wat::core::i64])] -> (:wat::core::PersistentVector :- [:wat::core::i64])
  (:wat::core::into (:wat::core::PersistentVector) v))

;; derived-vector — sorted, NOT deduped. Every Rate AND every Out.
;; Pre-cure oracle is Rate-only (Out dropped). Post-cure both. Empty is a third failure.
(:wat::core::defn :ufh::derived-vector [fired <- :wat::rete::Session] -> (:wat::core::PersistentVector :- [:wat::core::i64])
  (:wat::core::let
    [c0 (:wat::core::into (:wat::core::Vector :wat::core::i64)
          (:wat::core::map
            (:wat::core::fn [p <- :wat::core::PersistentMap] -> :wat::core::i64
              (:wat::core::let [f (:wat::core::Option/expect (:wat::core::PersistentMap/get p "?fact") "query: ?fact")]
                (:ufh::enc 0 (:ufh::Rate/count f))))
            (:wat::rete::query fired (:ufh::q-Rate))))
     c1 (:wat::core::into c0
          (:wat::core::map
            (:wat::core::fn [p <- :wat::core::PersistentMap] -> :wat::core::i64
              (:wat::core::let [f (:wat::core::Option/expect (:wat::core::PersistentMap/get p "?fact") "query: ?fact")]
                (:ufh::enc 1 (:ufh::Out/n f))))
            (:wat::rete::query fired (:ufh::q-Out))))]
    (:ufh::vec->pvec (:wat::core::sort c1))))

(:wat::core::defn :ufh::ns-between [t0 <- :wat::time::Instant  t1 <- :wat::time::Instant] -> :wat::core::i64
  (:wat::core::i64::- (:wat::time::epoch-nanos t1) (:wat::time::epoch-nanos t0)))

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::let [params  (:wat::core::match (:wat::kernel::readln ) ((:wat::kernel::ReadlnOutcome::Datum __datum) __datum) (:wat::kernel::ReadlnOutcome::Eof (:wat::kernel::assertion-failed! "readln: end of input" :wat::core::None :wat::core::None)) (:wat::kernel::ReadlnOutcome::Stopped (:wat::kernel::assertion-failed! "readln: stop requested" :wat::core::None :wat::core::None)))
                    items   (:wat::core::Option/expect (:wat::core::get params 0) "stdin: [items]")
                    rules   (:ufh::build-rules)
                    session (:wat::core::match (:wat::rete::compile-all rules (:wat::core::PersistentVector (:ufh::q-Rate) (:ufh::q-Out))) ((:wat::rete::CompileOutcome::Compiled __session) __session) ((:wat::rete::CompileOutcome::MayNotTerminate __rule __fact-type) (:wat::kernel::assertion-failed! "compile: the rule set may not terminate" :wat::core::None :wat::core::None)))
                    ;; Variable MUST be named `staged` so GRID_SKIP_ORACLE / axes_live rewrite
                    ;; `fire-rules$oracle staged` and not a first-fire leftover.
                    staged  (:ufh::seed session items)
                    n0      (:wat::time::now)
                    fired   (:ufh::fire staged)
                    n1      (:wat::time::now)
                    derived (:ufh::derived-vector fired)
                    nat-ns  (:ufh::ns-between n0 n1)
                    o0      (:wat::time::now)
                    ofired  (:wat::core::match (:wat::rete::fire-rules$oracle staged) ((:wat::rete::FireOutcome::Fired __fired) __fired) ((:wat::rete::FireOutcome::MemoryCeilingExceeded __limit __used __rounds) (:wat::kernel::assertion-failed! "fire-rules: session memory ceiling exceeded" :wat::core::None :wat::core::None)) ((:wat::rete::FireOutcome::RoundCapExceeded __cap __still) (:wat::kernel::assertion-failed! "fire-rules: fixpoint round cap exceeded" :wat::core::None :wat::core::None)))
                    o1      (:wat::time::now)]
    (:wat::kernel::println
      (:grid::Result :axis "userfn-head" :size (:wat::core::PersistentVector items) :derived derived :native-ns nat-ns :oracle-derived (:ufh::derived-vector ofired) :oracle-ns (:ufh::ns-between o0 o1)))))
