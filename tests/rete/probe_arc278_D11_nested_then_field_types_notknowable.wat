;; Fixture BESIDE probe_arc278_D11_nested_then_field_types.rs — ★ THE NOT-KNOWABLE SET, AT DEPTH.
;;
;; ⛔ THIS IS THE ROW WITHOUT WHICH A CURE THAT REFUSES EVERYTHING SCORES FULL MARKS.
;; D11's invariant is "a NESTED value whose type is KNOWABLE and does not match is refused" — and
;; the named failure of the strike is refusing what is merely not-knowable. That failure goes
;; GREEN on all four refusal fixtures and on the control, and silently stops a corpus of legal
;; rules compiling. D10 proved it is not theoretical: making `ComputedNotDerivableHere` a refusal
;; took four pre-existing corpus tests down with it.
;;
;; Each rule is a DIFFERENT reason the wall must stand down INSIDE a nested constructor,
;; constructed rather than asserted, and each derives a fact whose value is checked:
;;
;;   nk1  a computed operand under a `Form` head (`cond`), nested — `ComputedNotDerivableHere`.
;;        The row's `ret` is not a type it states; the answer comes later, from `check.rs`.
;;   nk2  a NESTED field whose declared type is a RECORD — `rete_type_segment_of` -> None, so the
;;        wall returns before it even asks the resolver.
;;   nk3  a constructor as the value of a nested constructor's field — DEPTH 2 on the passing
;;        side, so the recursion that arm 3's `.wat.bad` exercises on the refusing side is shown
;;        not to over-refuse.
;;   nk4  a `?var` bound from a DERIVED fact, used inside a nested constructor. Knowable, and
;;        RIGHT — the temptation is to treat "came from a derived fact" as unknowable and skip it,
;;        which would make the wall miss the very case D11 was found in.
;;   nk5  an enum-variant KEYWORD as a nested value. ⚠ This one pins a DELIBERATE gap rather than
;;        a channel of `OperandType`: `rhs_operand_can_never_resolve` skips a `Keyword` operand
;;        before the resolver is asked, so no nested keyword is typed at all. It is pinned here so
;;        that a later strike which starts typing keywords has to come through this file and say
;;        so, instead of discovering it as a corpus break.

(:wat::core::defenum :d11n::E :wat::enum::Pure :A :B)

(:wat::core::defrecord :d11n::In   [k <- :wat::core::i64  s <- :wat::core::String])
(:wat::core::defrecord :d11n::Pair [a <- :wat::core::String  b <- :wat::core::String])

;; nk1 — `cond` in a NESTED value position, filling an i64 field.
(:wat::core::defrecord :d11n::In1  [n <- :wat::core::i64])
(:wat::core::defrecord :d11n::Nk1  [i <- :d11n::In1])
(:wat::rete::defrule :d11n::nk1
  :when [(:d11n::In (?s <- :s))]
  :then [(:d11n::Nk1 :i (:d11n::In1 :n (:wat::rete::core::cond
                                         ((:wat::rete::core::string::= ?s "seed") 11)
                                         (:else 99))))])

;; nk2 — a RECORD-typed field of a NESTED constructor, filled from a `?var` bound to one.
(:wat::core::defrecord :d11n::Holder [p <- :d11n::Pair])
(:wat::core::defrecord :d11n::In2    [p <- :d11n::Pair])
(:wat::core::defrecord :d11n::Nk2    [i <- :d11n::In2])
(:wat::rete::defrule :d11n::nk2
  :when [(:d11n::Holder (?p <- :p))]
  :then [(:d11n::Nk2 :i (:d11n::In2 :p ?p))])

;; nk3 — a constructor as the value of a NESTED constructor's field (depth 2, passing side).
(:wat::core::defrecord :d11n::In3 [p <- :d11n::Pair])
(:wat::core::defrecord :d11n::Nk3 [i <- :d11n::In3])
(:wat::rete::defrule :d11n::nk3
  :when [(:d11n::In (?s <- :s))]
  :then [(:d11n::Nk3 :i (:d11n::In3 :p (:d11n::Pair :a ?s :b "nested")))])

;; nk4 — a two-stage derivation: nk4b's `?m` is bound from the fact nk4a derived, and it is the
;; NESTED constructor that consumes it.
(:wat::core::defrecord :d11n::In4  [n <- :wat::core::i64])
(:wat::core::defrecord :d11n::Nk4a [m <- :wat::core::i64])
(:wat::core::defrecord :d11n::Nk4b [i <- :d11n::In4])
(:wat::rete::defrule :d11n::nk4a
  :when [(:d11n::In (?k <- :k))]
  :then [(:d11n::Nk4a :m ?k)])
(:wat::rete::defrule :d11n::nk4b
  :when [(:d11n::Nk4a (?m <- :m))]
  :then [(:d11n::Nk4b :i (:d11n::In4 :n ?m))])

;; nk5 — an enum-variant keyword as a NESTED value: the class the operand skip excludes.
(:wat::core::defrecord :d11n::In5 [e <- :d11n::E])
(:wat::core::defrecord :d11n::Nk5 [i <- :d11n::In5])
(:wat::rete::defrule :d11n::nk5
  :when [(:d11n::In (?k <- :k))]
  :then [(:d11n::Nk5 :i (:d11n::In5 :e :d11n::E::B))])

(:wat::rete::defquery :d11n::q1 :params [] :when [(?f <- :d11n::Nk1)])
(:wat::rete::defquery :d11n::q2 :params [] :when [(?f <- :d11n::Nk2)])
(:wat::rete::defquery :d11n::q3 :params [] :when [(?f <- :d11n::Nk3)])
(:wat::rete::defquery :d11n::q4 :params [] :when [(?f <- :d11n::Nk4b)])
(:wat::rete::defquery :d11n::q5 :params [] :when [(?f <- :d11n::Nk5)])

(:wat::core::defn :d11n::fired [] -> :wat::rete::Session
  (:wat::core::let
    [s0 (:wat::core::match (:wat::rete::compile-all (:wat::rete::collect-rules :d11n)
          (:wat::core::PersistentVector (:d11n::q1) (:d11n::q2) (:d11n::q3) (:d11n::q4) (:d11n::q5)))
          ((:wat::rete::CompileOutcome::Compiled __s) __s)
          ((:wat::rete::CompileOutcome::MayNotTerminate __r __f) (:wat::kernel::assertion-failed! "compile: may not terminate" :wat::core::None :wat::core::None)))
     s1 (:wat::core::match (:wat::rete::insert s0 (:d11n::In :k 7 :s "seed"))
          ((:wat::rete::InsertOutcome::Inserted __x) __x)
          ((:wat::rete::InsertOutcome::MemoryCeilingExceeded __a __b __c) (:wat::kernel::assertion-failed! "insert: ceiling" :wat::core::None :wat::core::None)))
     s2 (:wat::core::match (:wat::rete::insert s1 (:d11n::Holder :p (:d11n::Pair :a "held" :b "pair")))
          ((:wat::rete::InsertOutcome::Inserted __x) __x)
          ((:wat::rete::InsertOutcome::MemoryCeilingExceeded __a __b __c) (:wat::kernel::assertion-failed! "insert: ceiling" :wat::core::None :wat::core::None)))]
    (:wat::core::match (:wat::rete::fire-rules s2)
      ((:wat::rete::FireOutcome::Fired __f) __f)
      ((:wat::rete::FireOutcome::MemoryCeilingExceeded __a __b __c) (:wat::kernel::assertion-failed! "fire: ceiling" :wat::core::None :wat::core::None))
      ((:wat::rete::FireOutcome::RoundCapExceeded __a __b) (:wat::kernel::assertion-failed! "fire: round cap" :wat::core::None :wat::core::None)))))

(:wat::core::defn :d11n::one [s <- :wat::rete::Session  q <- :wat::rete::Query] -> :wat::core::PersistentMap
  (:wat::core::first (:wat::rete::query s q)))

;; VALUES, one line per not-knowable arm: "11", "held", "seed", "7", ":d11n::E::B".
(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::let
    [s  (:d11n::fired)
     f1 (:wat::core::Option/expect (:wat::core::PersistentMap/get (:d11n::one s (:d11n::q1)) "?f") "nk1")
     f2 (:wat::core::Option/expect (:wat::core::PersistentMap/get (:d11n::one s (:d11n::q2)) "?f") "nk2")
     f3 (:wat::core::Option/expect (:wat::core::PersistentMap/get (:d11n::one s (:d11n::q3)) "?f") "nk3")
     f4 (:wat::core::Option/expect (:wat::core::PersistentMap/get (:d11n::one s (:d11n::q4)) "?f") "nk4")
     f5 (:wat::core::Option/expect (:wat::core::PersistentMap/get (:d11n::one s (:d11n::q5)) "?f") "nk5")]
    (:wat::core::do
      (:wat::kernel::println (:wat::core::format "{v}" :v (:d11n::In1/n (:d11n::Nk1/i f1))))
      (:wat::kernel::println (:wat::core::format "{v}" :v (:d11n::Pair/a (:d11n::In2/p (:d11n::Nk2/i f2)))))
      (:wat::kernel::println (:wat::core::format "{v}" :v (:d11n::Pair/a (:d11n::In3/p (:d11n::Nk3/i f3)))))
      (:wat::kernel::println (:wat::core::format "{v}" :v (:d11n::In4/n (:d11n::Nk4b/i f4))))
      (:wat::kernel::println (:wat::core::format "{v}" :v (:d11n::In5/e (:d11n::Nk5/i f5)))))))
