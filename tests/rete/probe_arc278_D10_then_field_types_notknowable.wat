;; Fixture BESIDE probe_arc278_D10_then_field_types.rs — ★ THE NOT-KNOWABLE SET.
;;
;; ⛔ THIS IS THE ROW WITHOUT WHICH A CURE THAT REFUSES EVERYTHING SCORES FULL MARKS.
;; D10's invariant is "a value whose type is KNOWABLE and does not match is refused" — and the
;; named failure of the strike is refusing what is merely not-knowable, which goes green on every
;; refusal probe and silently stops a corpus of legal rules from compiling.
;;
;; Each rule below is a DIFFERENT reason the wall must stand down, constructed rather than
;; asserted, and each derives a fact whose value is checked:
;;
;;   nk1  a computed operand under a `Form` head (`cond`) — `OperandType::ComputedNotDerivableHere`.
;;        The row's `ret` is not a type it states; the answer comes later, from `check.rs`.
;;   nk2  a destination field whose declared type is a RECORD — `rete_type_segment_of` -> None,
;;        so the wall returns before it even asks the resolver. There is no rete segment to
;;        compare at; a wall that guessed one would refuse every record-valued field in the tree.
;;   nk3  a nested CONSTRUCTOR as a value — a non-rete head, so `rete_op_for` declines and the
;;        answer is again `ComputedNotDerivableHere`, not "unbound".
;;   nk4  a `?var` bound from a DERIVED fact (nk4b matches what nk4a inserted). Knowable, and
;;        RIGHT — included because the temptation is to treat "came from a derived fact" as
;;        unknowable and skip it, which would make the wall miss the case D10 was found in.

(:wat::core::defrecord :dnk::In   [k <- :wat::core::i64  s <- :wat::core::String])
(:wat::core::defrecord :dnk::Pair [a <- :wat::core::String  b <- :wat::core::String])

;; nk1 — `cond` in value position, filling an i64 field.
(:wat::core::defrecord :dnk::Nk1 [n <- :wat::core::i64])
(:wat::rete::defrule :dnk::nk1
  :when [(:dnk::In (?s <- :s))]
  :then [(:dnk::Nk1 :n (:wat::rete::core::cond
                         ((:wat::rete::core::string::= ?s "seed") 11)
                         (:else 99)))])

;; nk2 — a RECORD-typed destination field, filled from a `?var` bound to one.
(:wat::core::defrecord :dnk::Holder [p <- :dnk::Pair]) 
(:wat::core::defrecord :dnk::Nk2    [p <- :dnk::Pair])
(:wat::rete::defrule :dnk::nk2
  :when [(:dnk::Holder (?p <- :p))]
  :then [(:dnk::Nk2 :p ?p)])

;; nk3 — a nested constructor as the value of a record-typed field.
(:wat::core::defrecord :dnk::Nk3 [p <- :dnk::Pair])
(:wat::rete::defrule :dnk::nk3
  :when [(:dnk::In (?s <- :s))]
  :then [(:dnk::Nk3 :p (:dnk::Pair :a ?s :b "nested"))])

;; nk4 — a two-stage derivation: nk4b's `?m` is bound from the fact nk4a derived.
(:wat::core::defrecord :dnk::Nk4a [m <- :wat::core::i64])
(:wat::core::defrecord :dnk::Nk4b [m <- :wat::core::i64])
(:wat::rete::defrule :dnk::nk4a
  :when [(:dnk::In (?k <- :k))]
  :then [(:dnk::Nk4a :m ?k)])
(:wat::rete::defrule :dnk::nk4b
  :when [(:dnk::Nk4a (?m <- :m))]
  :then [(:dnk::Nk4b :m ?m)])

(:wat::rete::defquery :dnk::q1 :params [] :when [(?f <- :dnk::Nk1)])
(:wat::rete::defquery :dnk::q2 :params [] :when [(?f <- :dnk::Nk2)])
(:wat::rete::defquery :dnk::q3 :params [] :when [(?f <- :dnk::Nk3)])
(:wat::rete::defquery :dnk::q4 :params [] :when [(?f <- :dnk::Nk4b)])

(:wat::core::defn :dnk::fired [] -> :wat::rete::Session
  (:wat::core::let
    [s0 (:wat::core::match (:wat::rete::compile-all (:wat::rete::collect-rules :dnk)
          (:wat::core::PersistentVector (:dnk::q1) (:dnk::q2) (:dnk::q3) (:dnk::q4)))
          ((:wat::rete::CompileOutcome::Compiled __s) __s)
          ((:wat::rete::CompileOutcome::MayNotTerminate __r __f) (:wat::kernel::assertion-failed! "compile: may not terminate" :wat::core::None :wat::core::None)))
     s1 (:wat::core::match (:wat::rete::insert s0 (:dnk::In :k 7 :s "seed"))
          ((:wat::rete::InsertOutcome::Inserted __x) __x)
          ((:wat::rete::InsertOutcome::MemoryCeilingExceeded __a __b __c) (:wat::kernel::assertion-failed! "insert: ceiling" :wat::core::None :wat::core::None)))
     s2 (:wat::core::match (:wat::rete::insert s1 (:dnk::Holder :p (:dnk::Pair :a "held" :b "pair")))
          ((:wat::rete::InsertOutcome::Inserted __x) __x)
          ((:wat::rete::InsertOutcome::MemoryCeilingExceeded __a __b __c) (:wat::kernel::assertion-failed! "insert: ceiling" :wat::core::None :wat::core::None)))]
    (:wat::core::match (:wat::rete::fire-rules s2)
      ((:wat::rete::FireOutcome::Fired __f) __f)
      ((:wat::rete::FireOutcome::MemoryCeilingExceeded __a __b __c) (:wat::kernel::assertion-failed! "fire: ceiling" :wat::core::None :wat::core::None))
      ((:wat::rete::FireOutcome::RoundCapExceeded __a __b) (:wat::kernel::assertion-failed! "fire: round cap" :wat::core::None :wat::core::None)))))

(:wat::core::defn :dnk::one [s <- :wat::rete::Session  q <- :wat::rete::Query] -> :wat::core::PersistentMap
  (:wat::core::first (:wat::rete::query s q)))

;; VALUES, one line per not-knowable arm: "11", "held", "seed", "7".
(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::let
    [s  (:dnk::fired)
     f1 (:wat::core::Option/expect (:wat::core::PersistentMap/get (:dnk::one s (:dnk::q1)) "?f") "nk1")
     f2 (:wat::core::Option/expect (:wat::core::PersistentMap/get (:dnk::one s (:dnk::q2)) "?f") "nk2")
     f3 (:wat::core::Option/expect (:wat::core::PersistentMap/get (:dnk::one s (:dnk::q3)) "?f") "nk3")
     f4 (:wat::core::Option/expect (:wat::core::PersistentMap/get (:dnk::one s (:dnk::q4)) "?f") "nk4")]
    (:wat::core::do
      (:wat::kernel::println (:wat::core::format "{v}" :v (:dnk::Nk1/n f1)))
      (:wat::kernel::println (:wat::core::format "{v}" :v (:dnk::Pair/a (:dnk::Nk2/p f2))))
      (:wat::kernel::println (:wat::core::format "{v}" :v (:dnk::Pair/a (:dnk::Nk3/p f3))))
      (:wat::kernel::println (:wat::core::format "{v}" :v (:dnk::Nk4b/m f4))))))
