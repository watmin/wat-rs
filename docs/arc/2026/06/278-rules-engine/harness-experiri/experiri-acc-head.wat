(:wat::core::defrecord :probe::In  [v <- :wat::core::i64])
(:wat::core::defrecord :probe::Out [n <- :wat::core::i64])

;; The ONLY RETE_OPS row whose declared signature is exactly (PersistentVector<T>) -> i64,
;; used DIRECTLY as the accumulator's acc-form head.
(:wat::rete::defrule :probe::acc
  :when  [(?n :- (:wat::rete::vector::length ?v) :from (:probe::In (?v :- :v)))]
  :then  [(:probe::Out :n ?n)])

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::let
    [rules   (:wat::rete::collect-rules :probe)
     session (:wat::core::match (:wat::rete::compile-all rules (:wat::core::PersistentVector))
               [:wat::rete::CompileOutcome.Compiled {:session s} s]
               [:wat::rete::CompileOutcome.MayNotTerminate {:rule __r :fact-type __f} (:wat::kernel::assertion-failed! :message "compile")])
     session (:wat::core::match (:wat::rete::insert session (:probe::In :v 1))
               [:wat::rete::InsertOutcome.Inserted {:session s} s]
               [:wat::rete::InsertOutcome.MemoryCeilingExceeded {:limit __a :used __b :staged __c} (:wat::kernel::assertion-failed! :message "ins")])
     fired   (:wat::core::match (:wat::rete::fire-rules session)
               [:wat::rete::FireOutcome.Fired {:value f} f]
               [:wat::rete::FireOutcome.MemoryCeilingExceeded {:limit __l :used __u :rounds __r} (:wat::kernel::assertion-failed! :message "mem")]
               [:wat::rete::FireOutcome.RoundCapExceeded {:cap __c :still-deriving __s} (:wat::kernel::assertion-failed! :message "cap")])]
    (:wat::kernel::println "fired")))
