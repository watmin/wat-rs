(:wat::core::defrecord :probe::In  [v <- :wat::core::i64])
(:wat::core::defrecord :probe::Out [n <- :wat::core::i64])

;; The ONLY RETE_OPS row whose declared signature is exactly (PersistentVector<T>) -> i64,
;; used DIRECTLY as the accumulator's acc-form head.
(:wat::rete::defrule :probe::acc
  :when  [(?n <- (:wat::rete::core::PersistentVector/length ?v) :from (:probe::In (?v <- :v)))]
  :then  [(:probe::Out :n ?n)])

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::let
    [rules   (:wat::rete::collect-rules :probe)
     session (:wat::core::match (:wat::rete::compile-all rules (:wat::core::PersistentVector))
               ((:wat::rete::CompileOutcome::Compiled s) s)
               ((:wat::rete::CompileOutcome::MayNotTerminate __r __f) (:wat::kernel::assertion-failed! "compile" :wat::core::None :wat::core::None)))
     session (:wat::core::match (:wat::rete::insert session (:probe::In :v 1))
               ((:wat::rete::InsertOutcome::Inserted s) s)
               ((:wat::rete::InsertOutcome::MemoryCeilingExceeded __a __b __c) (:wat::kernel::assertion-failed! "ins" :wat::core::None :wat::core::None)))
     fired   (:wat::core::match (:wat::rete::fire-rules session)
               ((:wat::rete::FireOutcome::Fired f) f)
               ((:wat::rete::FireOutcome::MemoryCeilingExceeded __l __u __r) (:wat::kernel::assertion-failed! "mem" :wat::core::None :wat::core::None))
               ((:wat::rete::FireOutcome::RoundCapExceeded __c __s) (:wat::kernel::assertion-failed! "cap" :wat::core::None :wat::core::None)))]
    (:wat::kernel::println "fired")))
