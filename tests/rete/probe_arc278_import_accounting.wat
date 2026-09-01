;; strike-import-accounting (arc 278, class A7) — the door's own fixture.
;;
;; The Rust probes beside this file tamper with an `Export`'s `nodes` field to drive the node cap
;; (`MAX_IMPORT_NODES`), which is checked on the DECLARED length before any node is unpacked. The
;; program itself is deliberately the smallest one that produces a real Export.

(:wat::core::defrecord :ia::Temp [c <- :wat::core::i64])
(:wat::core::defrecord :ia::Hit [c <- :wat::core::i64])

(:wat::rete::defquery :ia::q-Hit :params [] :when [(?fact <- :ia::Hit)])

(:wat::rete::defrule :ia::cool
  :when [(:ia::Temp (?c <- :c))
         (:wat::rete::where (:wat::rete::core::i64::< ?c 20))]
  :then [(:ia::Hit ?c)])

(:wat::core::defn :ia::compiled [] -> :wat::rete::Session
  (:wat::core::match (:wat::rete::compile-all
      (:wat::core::PersistentVector (:ia::cool))
      (:wat::core::PersistentVector (:ia::q-Hit)))
    ((:wat::rete::CompileOutcome::Compiled __session) __session)
    ((:wat::rete::CompileOutcome::MayNotTerminate __rule __fact-type)
      (:wat::kernel::assertion-failed! "compile: the rule set may not terminate" :wat::core::None :wat::core::None))))

(:wat::core::defn :user::an-export [] -> :wat::rete::Export
  (:wat::rete::export (:ia::compiled)))

(:wat::core::defn :user::import-one [e <- :wat::rete::Export] -> :wat::rete::Session
  (:wat::rete::import e))
