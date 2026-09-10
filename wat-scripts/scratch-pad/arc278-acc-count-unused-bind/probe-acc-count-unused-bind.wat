;; probe-acc-count-unused-bind.wat — ⛔ A NATIVE ENGINE DEFECT: acc::count returns a WRONG COUNT
;; when the `:from` inner carries a bind nothing consumes.
;;
;; Driven 2026-09-10 at `dfd883886`. THREE readings are inserted in both arms. The ONLY difference
;; between the two rules is an extra `(?v <- :value)` bind in the `:from` inner that NOTHING reads —
;; not the acc-form, not a constraint, not the `:then`.
;;
;;   plain            (?loc <- :location)                  ->  n = 3   ✅ correct
;;   with-unused-bind (?loc <- :location) (?v <- :value)   ->  n = 1   ⛔ WRONG
;;
;; ⛔ BOTH ARMS ARE IN THIS ONE FILE ON PURPOSE. A single wrong number proves nothing on its own —
;; it could be a miscounted insert, a bad query, a typo in the record name. Printed side by side,
;; the control interprets the defect: same facts, same session, same query shape, one extra
;; inert binding.
;;
;; NOT a type-checking question and NOT the arc-278 `:from` validator strike's subject (that strike
;; explicitly excludes the engine/reducer body). `acc::sum` over the SAME two binds is unaffected —
;; 10+20+30 sums correctly — so this is specific to the count fold's handling of the inner's
;; binding set. Suspected home: `src/rete/kernel/fire/acc.rs` / `fire/pass/accumulate.rs`.
;;
;; Finding: docs/arc/2026/06/278-rules-engine/the-position-axis-was-chosen-not-derived/
;;          FINDING-acc-count-miscounts-with-an-unused-bind.md

(:wat::core::defrecord :cnb::Station [location <- :wat::core::String])
(:wat::core::defrecord :cnb::Reading [location <- :wat::core::String  value <- :wat::core::i64])
(:wat::core::defrecord :cnb::Plain   [n <- :wat::core::i64])
(:wat::core::defrecord :cnb::Extra   [n <- :wat::core::i64])

(:wat::rete::defrule :cnb::plain
  :when [(:cnb::Station (?loc <- :location))
         (?n <- (:wat::rete::acc::count) :from (:cnb::Reading (?loc <- :location)))]
  :then [(:cnb::Plain :n ?n)])

;; identical, plus ONE bind that nothing consumes
(:wat::rete::defrule :cnb::with-unused-bind
  :when [(:cnb::Station (?loc <- :location))
         (?n <- (:wat::rete::acc::count) :from (:cnb::Reading (?loc <- :location) (?v <- :value)))]
  :then [(:cnb::Extra :n ?n)])

(:wat::rete::defquery :cnb::q-plain :params [] :when [(?f <- :cnb::Plain)])
(:wat::rete::defquery :cnb::q-extra :params [] :when [(?f <- :cnb::Extra)])

(:wat::core::defn :cnb::ins [s <- :wat::rete::Session  f <- :cnb::Reading] -> :wat::rete::Session
  (:wat::core::match (:wat::rete::insert s f) ((:wat::rete::InsertOutcome::Inserted __x) __x)
    ((:wat::rete::InsertOutcome::MemoryCeilingExceeded __a __b __c) (:wat::kernel::assertion-failed! "insert" :wat::core::None :wat::core::None))))

(:wat::core::defn :cnb::n [s <- :wat::rete::Session  q <- :wat::rete::Query] -> :wat::core::i64
  (:cnb::Plain/n (:wat::core::Option/expect
    (:wat::core::PersistentMap/get (:wat::core::first (:wat::rete::query s q)) "?f") "row")))

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::let
    [s0 (:wat::core::match (:wat::rete::compile-all (:wat::rete::collect-rules :cnb)
          (:wat::core::PersistentVector (:cnb::q-plain) (:cnb::q-extra)))
          ((:wat::rete::CompileOutcome::Compiled __s) __s)
          ((:wat::rete::CompileOutcome::MayNotTerminate __r __f) (:wat::kernel::assertion-failed! "compile" :wat::core::None :wat::core::None)))
     s1 (:wat::core::match (:wat::rete::insert s0 (:cnb::Station :location "A")) ((:wat::rete::InsertOutcome::Inserted __x) __x)
          ((:wat::rete::InsertOutcome::MemoryCeilingExceeded __a __b __c) (:wat::kernel::assertion-failed! "insert" :wat::core::None :wat::core::None)))
     s2 (:cnb::ins s1 (:cnb::Reading :location "A" :value 10))
     s3 (:cnb::ins s2 (:cnb::Reading :location "A" :value 20))
     s4 (:cnb::ins s3 (:cnb::Reading :location "A" :value 30))
     fired (:wat::core::match (:wat::rete::fire-rules s4) ((:wat::rete::FireOutcome::Fired __f) __f)
             ((:wat::rete::FireOutcome::MemoryCeilingExceeded __a __b __c) (:wat::kernel::assertion-failed! "mem" :wat::core::None :wat::core::None))
             ((:wat::rete::FireOutcome::RoundCapExceeded __a __b) (:wat::kernel::assertion-failed! "cap" :wat::core::None :wat::core::None)))
     plain (:cnb::n fired (:cnb::q-plain))
     extra (:wat::core::i64::- (:cnb::Extra/n (:wat::core::Option/expect
             (:wat::core::PersistentMap/get (:wat::core::first (:wat::rete::query fired (:cnb::q-extra))) "?f") "row")) 0)]
    (:wat::kernel::println (:wat::core::format
      "3 readings inserted -> plain={p}  with-unused-bind={e}   (both must be 3)" :p plain :e extra))))
