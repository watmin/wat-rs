;; arc 278 — ARM 2: can the produced-type divergence change derived FACTS?
;;
;; Shape (BRIEF sketch):
;;   Src(k)                         [input]
;;   Bad(k) :- Src(k), k=2          derived, so (not Bad) raises the next rule
;;   Rate  :- Src(k), (not Bad(k)), :then (mk-rate ?k)
;;                                  mk-rate is a rete defn whose body constructs Rate
;;                                  from the bound k — not a computed mint
;;   Out(n) :- Rate(n)
;;
;; Native assigns stratum to Rate; the oracle assigns it to mk-rate and leaves Rate at 0.
;; Out consuming Rate is then below its producer on the oracle. Stratified fire does not
;; re-fire lower strata, so oracle Out would never appear.
;;
;; Construction is a bound-var kwargs record, not i64::+. rete_fn_body_mints looks for a
;; constructor whose argument is a List; a Symbol is not a List.
;;
;; Run: cargo run --release --bin wat -- wat-scripts/scratch-pad/arc278-produced-type-userfn-facts.wat

(:wat::core::defrecord :a2::Src  [k <- :wat::core::i64])
(:wat::core::defrecord :a2::Bad  [k <- :wat::core::i64])
(:wat::core::defrecord :a2::Rate [count <- :wat::core::i64])
(:wat::core::defrecord :a2::Out  [n <- :wat::core::i64])

(:wat::rete::core::defn :a2::mk-rate
  [k <- :wat::core::i64]
  -> :a2::Rate
  (:a2::Rate :count k))

(:wat::rete::defrule :a2::bad
  :when [(:a2::Src (?k <- :k))
         (:wat::rete::where (:wat::rete::core::i64::= ?k 2))]
  :then [(:a2::Bad :k ?k)])

(:wat::rete::defrule :a2::via
  :when [(:a2::Src (?k <- :k))
         (:wat::rete::not (:a2::Bad (?k <- :k)))]
  :then [(:a2::mk-rate ?k)])

(:wat::rete::defrule :a2::out
  :when [(:a2::Rate (?n <- :count))]
  :then [(:a2::Out :n ?n)])

(:wat::rete::defquery :a2::q-Rate :params [] :when [(?f <- :a2::Rate)])
(:wat::rete::defquery :a2::q-Out  :params [] :when [(?f <- :a2::Out)])
(:wat::rete::defquery :a2::q-Bad  :params [] :when [(?f <- :a2::Bad)])

(:wat::core::defn :a2::readback [s <- :wat::rete::Session]
  -> (:wat::core::PersistentVector :- [:wat::core::i64])
  (:wat::core::PersistentVector
    (:wat::core::length (:wat::rete::query s (:a2::q-Bad)))
    (:wat::core::length (:wat::rete::query s (:a2::q-Rate)))
    (:wat::core::length (:wat::rete::query s (:a2::q-Out)))))

(:wat::core::defn :a2::empty-records [] -> (:wat::core::PersistentVector :- [:wat::core::Record])
  (:wat::core::PersistentVector))

(:wat::core::defn :a2::staged [] -> :wat::rete::Session
  (:wat::core::let [rules (:wat::core::PersistentVector (:a2::bad) (:a2::via) (:a2::out))
                    qs    (:wat::core::PersistentVector (:a2::q-Rate) (:a2::q-Out) (:a2::q-Bad))
                    session (:wat::core::match (:wat::rete::compile-all rules qs)
                              ((:wat::rete::CompileOutcome::Compiled __s) __s)
                              ((:wat::rete::CompileOutcome::MayNotTerminate __r __f)
                                (:wat::kernel::assertion-failed! "compile: MayNotTerminate" :wat::core::None :wat::core::None)))]
    (:wat::core::match
      (:wat::rete::insert-all session
        (:wat::core::PersistentVector/conj
          (:a2::empty-records)
          (:a2::Src :k 1)))
      ((:wat::rete::InsertOutcome::Inserted __s) __s)
      ((:wat::rete::InsertOutcome::MemoryCeilingExceeded __l __u __c)
        (:wat::kernel::assertion-failed! "insert: ceiling" :wat::core::None :wat::core::None)))))

(:wat::core::defn :user::native-facts [] -> (:wat::core::PersistentVector :- [:wat::core::i64])
  (:a2::readback
    (:wat::core::match (:wat::rete::fire-rules (:a2::staged))
      ((:wat::rete::FireOutcome::Fired __f) __f)
      ((:wat::rete::FireOutcome::MemoryCeilingExceeded __l __u __r)
        (:wat::kernel::assertion-failed! "native fire: ceiling" :wat::core::None :wat::core::None))
      ((:wat::rete::FireOutcome::RoundCapExceeded __c __s)
        (:wat::kernel::assertion-failed! "native fire: round cap" :wat::core::None :wat::core::None)))))

(:wat::core::defn :user::oracle-facts [] -> (:wat::core::PersistentVector :- [:wat::core::i64])
  (:a2::readback
    (:wat::core::match (:wat::rete::fire-rules$oracle (:a2::staged))
      ((:wat::rete::FireOutcome::Fired __f) __f)
      ((:wat::rete::FireOutcome::MemoryCeilingExceeded __l __u __r)
        (:wat::kernel::assertion-failed! "oracle fire: ceiling" :wat::core::None :wat::core::None))
      ((:wat::rete::FireOutcome::RoundCapExceeded __c __s)
        (:wat::kernel::assertion-failed! "oracle fire: round cap" :wat::core::None :wat::core::None)))))

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::do
    (:wat::kernel::println "COMPILE: Compiled")
    (:wat::kernel::println "ORACLE STRATA:")
    (:wat::kernel::println (:wat::rete::stratify (:wat::core::PersistentVector (:a2::bad) (:a2::via) (:a2::out))))
    (:wat::kernel::println "ORACLE rule-produces via:")
    (:wat::kernel::println (:wat::rete::rule-produces (:a2::via)))
    (:wat::kernel::println "NATIVE facts [Bad Rate Out]:")
    (:wat::kernel::println (:user::native-facts))
    (:wat::kernel::println "ORACLE facts [Bad Rate Out]:")
    (:wat::kernel::println (:user::oracle-facts))))
