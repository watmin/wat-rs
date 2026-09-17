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
;;
;; ⛔ MERGE NOTE (replay #349) — nk5 DROPPED, not adapted; it pins a gap this tree does not have,
;; for a reason ORTHOGONAL to D10/D11. Grok's nk5 put a bare enum-variant KEYWORD
;; (`:d11n::E.B`) as a nested `:then` value, to pin that `rhs_operand_can_never_resolve` skips a
;; `Keyword` BEFORE the type resolver is asked — on grok's tree this is a genuine gap because
;; `check_rhs_operands` (the structural producer that predicate feeds) is NOT called at nested
;; depth there (D11's own commit body says so explicitly). On THIS tree it is not a gap: this same
;; file's `src/rete/validate/mod.rs` already carries a "MERGE NOTE (REPLAY #262)" recording that
;; main independently unified `check_rhs_operands` to run at nested depth BEFORE D10 or D11 ever
;; landed here — so the STRUCTURAL half of that check (`RhsUnresolvableOperand`, "a keyword is a
;; field reference in a RHS, not a value") already refuses a bare nested keyword, at STATIC
;; validation, regardless of D10/D11's typing work. Measured: running the pre-#349 file with nk5
;; present fails at `--check` (not merely at runtime) with exactly that message. Reported, not
;; routed around — there is no well-typed rephrasing of "a bare keyword as a value" that survives
;; a wall which refuses the SHAPE outright.

(:wat::core::defrecord :d11n::In   [k <- :wat::core::i64  s <- :wat::core::String])
(:wat::core::defrecord :d11n::Pair [a <- :wat::core::String  b <- :wat::core::String])

;; nk1 — `cond` in a NESTED value position, filling an i64 field.
(:wat::core::defrecord :d11n::In1  [n <- :wat::core::i64])
(:wat::core::defrecord :d11n::Nk1  [i <- :d11n::In1])
(:wat::rete::defrule :d11n::nk1
  :when [(:d11n::In (?s <- :s))]
  :then [(:d11n::Nk1 :i (:d11n::In1 :n (:wat::rete::core::cond
                                         ((:wat::rete::string::= ?s "seed") 11)
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

(:wat::rete::defquery :d11n::q1 :params [] :when [(?f <- :d11n::Nk1)])
(:wat::rete::defquery :d11n::q2 :params [] :when [(?f <- :d11n::Nk2)])
(:wat::rete::defquery :d11n::q3 :params [] :when [(?f <- :d11n::Nk3)])
(:wat::rete::defquery :d11n::q4 :params [] :when [(?f <- :d11n::Nk4b)])

(:wat::core::defn :d11n::fired [] -> :wat::rete::Session
  (:wat::core::let
    [s0 (:wat::core::match (:wat::rete::compile-all (:wat::rete::collect-rules :d11n)
          (:wat::core::PersistentVector (:d11n::q1) (:d11n::q2) (:d11n::q3) (:d11n::q4)))
          [:wat::rete::CompileOutcome.Compiled {:session __s} __s]
          [:wat::rete::CompileOutcome.MayNotTerminate {:rule __r :fact-type __f} (:wat::kernel::assertion-failed! :message "compile: may not terminate")])
     s1 (:wat::core::match (:wat::rete::insert s0 (:d11n::In :k 7 :s "seed"))
          [:wat::rete::InsertOutcome.Inserted {:session __x} __x]
          [:wat::rete::InsertOutcome.MemoryCeilingExceeded {:limit __a :used __b :staged __c} (:wat::kernel::assertion-failed! :message "insert: ceiling")])
     s2 (:wat::core::match (:wat::rete::insert s1 (:d11n::Holder :p (:d11n::Pair :a "held" :b "pair")))
          [:wat::rete::InsertOutcome.Inserted {:session __x} __x]
          [:wat::rete::InsertOutcome.MemoryCeilingExceeded {:limit __a :used __b :staged __c} (:wat::kernel::assertion-failed! :message "insert: ceiling")])]
    (:wat::core::match (:wat::rete::fire-rules s2)
      [:wat::rete::FireOutcome.Fired {:value __f} __f]
      [:wat::rete::FireOutcome.MemoryCeilingExceeded {:limit __a :used __b :rounds __c} (:wat::kernel::assertion-failed! :message "fire: ceiling")]
      [:wat::rete::FireOutcome.RoundCapExceeded {:cap __a :still-deriving __b} (:wat::kernel::assertion-failed! :message "fire: round cap")])))

(:wat::core::defn :d11n::one [s <- :wat::rete::Session  q <- :wat::rete::Query] -> :wat::core::PersistentMap
  (:wat::core::first (:wat::rete::query s q)))

;; VALUES, one line per not-knowable arm: "11", "held", "seed", "7".
(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::let
    [s  (:d11n::fired)
     f1 (:wat::core::Option/expect (:wat::map::get (:d11n::one s (:d11n::q1)) "?f") "nk1")
     f2 (:wat::core::Option/expect (:wat::map::get (:d11n::one s (:d11n::q2)) "?f") "nk2")
     f3 (:wat::core::Option/expect (:wat::map::get (:d11n::one s (:d11n::q3)) "?f") "nk3")
     f4 (:wat::core::Option/expect (:wat::map::get (:d11n::one s (:d11n::q4)) "?f") "nk4")]
    (:wat::core::do
      (:wat::kernel::println (:wat::core::format "{v}" :v (:d11n::In1/n (:d11n::Nk1/i f1))))
      (:wat::kernel::println (:wat::core::format "{v}" :v (:d11n::Pair/a (:d11n::In2/p (:d11n::Nk2/i f2)))))
      (:wat::kernel::println (:wat::core::format "{v}" :v (:d11n::Pair/a (:d11n::In3/p (:d11n::Nk3/i f3)))))
      (:wat::kernel::println (:wat::core::format "{v}" :v (:d11n::In4/n (:d11n::Nk4b/i f4)))))))
