;; Fixture BESIDE probe_arc278_D10_then_field_types.rs — THE CONTROL for D10.
;;
;; ⛔ LOAD-BEARING. D10's wall makes a `:then` REFUSE for the first time, and the failure that
;; scores full marks on every other row is a wall that refuses EVERYTHING. Three `.wat.bad`
;; siblings prove the refusals; this file is the half that proves the wall still lets correct
;; programs through — and it checks the derived values BY VALUE, not by count, because a count of
;; 1 cannot see a wrong value (which is the very defect D10 exists to stop).
;;
;; Every well-typed shape the wall now inspects is here:
;;   row 1  a bound `?var` into a field of its own declared type      (source 2 of the resolver)
;;   row 2  a literal into a field of the literal's type              (source 3)
;;   row 3  a computed rete op whose row declares `ret` i64           (source 4, KNOWABLE and RIGHT)
;;   row 4  the POSITIONAL spelling, every arg well-typed             (the second producer arm)

(:wat::core::defrecord :dok::In  [k <- :wat::core::i64  s <- :wat::core::String])
(:wat::core::defrecord :dok::Out [n <- :wat::core::i64  t <- :wat::core::String])

;; rows 1+2: a bound i64 `?var` into `:n`, a String literal into `:t`
(:wat::rete::defrule :dok::bound-and-literal
  :when [(:dok::In (?k <- :k))]
  :then [(:dok::Out :n ?k :t "lit")])

(:wat::core::defrecord :dok::Sum [n <- :wat::core::i64  t <- :wat::core::String])
;; row 3: a COMPUTED operand the resolver types as i64 from its row's declared `ret` — knowable
;; AND correct, so the wall must pass it. The negative twin of this row is the `.wat.bad` where
;; the same computed i64 goes into a String field.
(:wat::rete::defrule :dok::computed
  :when [(:dok::In (?k <- :k) (?s <- :s))]
  :then [(:dok::Sum :n (:wat::rete::core::i64::+ ?k 1 :undefined 0) :t ?s)])

(:wat::core::defrecord :dok::Pos [n <- :wat::core::i64  t <- :wat::core::String])
;; row 4: POSITIONAL — args are declaration order by definition, so this exercises the OTHER
;; producer arm of the same wall. Both args are well-typed.
(:wat::rete::defrule :dok::positional
  :when [(:dok::In (?k <- :k) (?s <- :s))]
  :then [(:dok::Pos ?k ?s)])

(:wat::rete::defquery :dok::q-out :params [] :when [(?f <- :dok::Out)])
(:wat::rete::defquery :dok::q-sum :params [] :when [(?f <- :dok::Sum)])
(:wat::rete::defquery :dok::q-pos :params [] :when [(?f <- :dok::Pos)])

(:wat::core::defn :dok::fired [] -> :wat::rete::Session
  (:wat::core::match (:wat::rete::fire-rules
    (:wat::core::match (:wat::rete::insert-all
      (:wat::core::match (:wat::rete::compile-all (:wat::rete::collect-rules :dok)
        (:wat::core::PersistentVector (:dok::q-out) (:dok::q-sum) (:dok::q-pos)))
        ((:wat::rete::CompileOutcome::Compiled __s) __s)
        ((:wat::rete::CompileOutcome::MayNotTerminate __r __f) (:wat::kernel::assertion-failed! "compile: may not terminate" :wat::core::None :wat::core::None)))
      (:wat::core::PersistentVector (:dok::In :k 7 :s "seed")))
      ((:wat::rete::InsertOutcome::Inserted __x) __x)
      ((:wat::rete::InsertOutcome::MemoryCeilingExceeded __a __b __c) (:wat::kernel::assertion-failed! "insert: ceiling" :wat::core::None :wat::core::None)))
    )
    ((:wat::rete::FireOutcome::Fired __f) __f)
    ((:wat::rete::FireOutcome::MemoryCeilingExceeded __a __b __c) (:wat::kernel::assertion-failed! "fire: ceiling" :wat::core::None :wat::core::None))
    ((:wat::rete::FireOutcome::RoundCapExceeded __a __b) (:wat::kernel::assertion-failed! "fire: round cap" :wat::core::None :wat::core::None))))

(:wat::core::defn :dok::one [s <- :wat::rete::Session  q <- :wat::rete::Query] -> :wat::core::PersistentMap
  (:wat::core::first (:wat::rete::query s q)))

;; VALUES, not counts — three lines: "7 lit", "8 seed", "7 seed".
(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::let
    [s (:dok::fired)
     out (:wat::core::Option/expect (:wat::core::PersistentMap/get (:dok::one s (:dok::q-out)) "?f") "out")
     sum (:wat::core::Option/expect (:wat::core::PersistentMap/get (:dok::one s (:dok::q-sum)) "?f") "sum")
     pos (:wat::core::Option/expect (:wat::core::PersistentMap/get (:dok::one s (:dok::q-pos)) "?f") "pos")]
    (:wat::core::do
      (:wat::kernel::println (:wat::core::format "{n} {t}" :n (:dok::Out/n out) :t (:dok::Out/t out)))
      (:wat::kernel::println (:wat::core::format "{n} {t}" :n (:dok::Sum/n sum) :t (:dok::Sum/t sum)))
      (:wat::kernel::println (:wat::core::format "{n} {t}" :n (:dok::Pos/n pos) :t (:dok::Pos/t pos))))))
