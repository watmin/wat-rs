;; probe-derive-decomposition.wat — arc 278. WHERE does the derive time actually go?
;;
;; WHY THIS EXISTS: at fanout [40000] the whole program is ~5.3 s and the FIRE is ~0.046 s of it.
;; I twice attributed the remaining ~5 s to a component by READING the code — first to seeding
;; (disproved: ~13 ms), then to `vec->pvec` (disproved: deleting its interpreted pass moved the
;; wall clock by a median +0.06 s, i.e. nothing). Both were reasoned, neither measured.
;;
;; So this probe stops guessing. It fires the SAME workload fanout.wat fires, then times each
;; stage of the derive SEPARATELY:
;;
;;   query   — :wat::rete::query-by-type-string over the derived Pairs
;;   map     — the interpreted closure: 3 accessor calls + enc arithmetic, per derived fact
;;   sort    — :wat::core::sort over the encoded i64s
;;   pvec    — materialize into a PersistentVector (now ONE native into; was an N-step conj-fold)
;;
;; stdin  = [items]   (same shape as fanout.wat: keys = items / fanout^2, fanout = 20)
;; stdout = one #probe/DeriveSplit EDN line. It asserts nothing; the disk decides.

(:wat::core::defrecord :dd::Left  [key <- :wat::core::i64  lid <- :wat::core::i64])
(:wat::core::defrecord :dd::Right [key <- :wat::core::i64  rid <- :wat::core::i64])
(:wat::core::defrecord :dd::Pair  [key <- :wat::core::i64  lid <- :wat::core::i64  rid <- :wat::core::i64])

(:wat::core::defrecord :probe::DeriveSplit
  [derived-count <- :wat::core::i64
   fire-ns       <- :wat::core::i64
   query-ns      <- :wat::core::i64
   map-ns        <- :wat::core::i64
   sort-ns       <- :wat::core::i64
   pvec-ns       <- :wat::core::i64])

(:wat::rete::defquery :dd::q-Pair
  :params []
  :when [(?fact <- :dd::Pair)])


(:wat::core::defn :dd::ns-between [t0 <- :wat::time::Instant  t1 <- :wat::time::Instant] -> :wat::core::i64
  (:wat::core::i64::- (:wat::time::epoch-nanos t1) (:wat::time::epoch-nanos t0)))

(:wat::core::defn :dd::enc [key <- :wat::core::i64  lid <- :wat::core::i64  rid <- :wat::core::i64] -> :wat::core::i64
  (:wat::core::i64::+ (:wat::core::i64::+ (:wat::core::i64::* key 1000000) (:wat::core::i64::* lid 1000)) rid))

(:wat::core::defn :dd::seed-key [s <- :wat::rete::Session  k <- :wat::core::i64  fanout <- :wat::core::i64] -> :wat::rete::Session
  (:wat::core::foldl
    (:wat::core::fn [acc <- :wat::rete::Session  f <- :wat::core::i64] -> :wat::rete::Session
      (:wat::rete::insert (:wat::rete::insert acc (:dd::Left :key k :lid f)) (:dd::Right :key k :rid f)))
    s
    (:wat::core::range 0 fanout)))

(:wat::core::defn :dd::seed [s <- :wat::rete::Session  keys <- :wat::core::i64  fanout <- :wat::core::i64] -> :wat::rete::Session
  (:wat::core::foldl
    (:wat::core::fn [acc <- :wat::rete::Session  k <- :wat::core::i64] -> :wat::rete::Session
      (:dd::seed-key acc k fanout))
    s
    (:wat::core::range 0 keys)))

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::let
    [params (:wat::core::match (:wat::kernel::readln )
              ((:wat::kernel::ReadlnOutcome::Datum __d) __d)
              (:wat::kernel::ReadlnOutcome::Eof     (:wat::kernel::assertion-failed! "readln: eof"  :wat::core::None :wat::core::None))
              (:wat::kernel::ReadlnOutcome::Stopped (:wat::kernel::assertion-failed! "readln: stop" :wat::core::None :wat::core::None)))
     items   (:wat::core::Option/expect (:wat::core::get params 0) "stdin: [items]")
     fanout  20
     keys    (:wat::core::i64::/ items (:wat::core::i64::* fanout fanout))
     c1      (:wat::core::quote (:dd::Left  (?k <- :key) (?l <- :lid)))
     c2      (:wat::core::quote (:dd::Right (?k <- :key) (?r <- :rid)))
     rhs     (:wat::core::quote (:dd::Pair ?k ?l ?r))
     rule    (:wat::rete::Rule :name "dd" :lhs (:wat::core::PersistentVector c1 c2) :rhs (:wat::core::PersistentVector rhs))
     staged  (:dd::seed (:wat::rete::compile-all (:wat::core::PersistentVector rule) (:wat::core::PersistentVector (:dd::q-Pair))) keys fanout)

     f0      (:wat::time::now)
     fired   (:wat::rete::fire-rules staged)
     f1      (:wat::time::now)

     ;; ── the derive, stage by stage ──────────────────────────────────────────────────────
     ;; NOTE: `query-by-type-string` returns a PersistentVector, and `into` has a (PV,Vector)
     ;; clause but NOT its mirror (Vector,PV) — so this cannot be materialised into a Vector
     ;; without the very asymmetry DESIGN-STONE-into-pv-from-vector.md left owed. Map directly.
     q0      (:wat::time::now)
     pairs   (:wat::rete::query fired (:dd::q-Pair))
     q1      (:wat::time::now)

     m0      (:wat::time::now)
     codes   (:wat::core::into (:wat::core::Vector :wat::core::i64)
               (:wat::core::map
                 (:wat::core::fn [p <- :wat::core::PersistentMap] -> :wat::core::i64 (:wat::core::let [f (:wat::core::Option/expect (:wat::core::PersistentMap/get p "?fact") "query: ?fact")] (:dd::enc (:dd::Pair/key f) (:dd::Pair/lid f) (:dd::Pair/rid f))))
                 pairs))
     m1      (:wat::time::now)

     s0      (:wat::time::now)
     sorted  (:wat::core::sort codes)
     s1      (:wat::time::now)

     p0      (:wat::time::now)
     pv      (:wat::core::into (:wat::core::PersistentVector) sorted)
     p1      (:wat::time::now)]

    (:wat::kernel::println
      (:probe::DeriveSplit
        :derived-count (:wat::core::PersistentVector/length pv)   ; non-vacuity: a zero here means nothing was derived
        :fire-ns  (:dd::ns-between f0 f1)
        :query-ns (:dd::ns-between q0 q1)
        :map-ns   (:dd::ns-between m0 m1)
        :sort-ns  (:dd::ns-between s0 s1)
        :pvec-ns  (:dd::ns-between p0 p1)))))
