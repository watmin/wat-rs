;; d11-nested-then-rhs-is-not-type-checked.wat — work-list D11. ⛔ CURED 2026-09-03; this file is
;; now the RECORD of the defect, and it no longer reproduces it.
;;
;; THE DEFECT. D10 taught the rete `:then` RHS to type its field values — at the TOP LEVEL of a
;; fact form only. The identical flaw survived inside a NESTED constructor. Driven at HEAD
;; `f87bb070b`, the commit immediately after D10's cure, against the CURED binary:
;;
;;   :then [(:nh::Outer :i (:nh::Inner :n ?s))]     ?s : String, :nh::Inner.n : i64
;;     ->  "Outer count:" / 1
;;         #wat.core/PersistentVector [#wat.core/PersistentMap
;;           {"?f" #nh/Outer {:i #nh/Inner {:n "nested-string"}}}]
;;
;; A wrong-typed value still reached the FACT SET, one level down.
;;
;; THE CAUSE, and it was ONE MISSING PARAMETER. `walk_nested_constructors`
;; (`src/rete/validate/mod.rs`) took `(operand, rule_name, types, errors)` — no `binds`.
;; `resolve_operand_type` needs `binds` to type a `?var`, so that walker could only ever check
;; field NAMES, arity and missing fields: every wall it carried was structural.
;;
;; THE CURE. `binds` threaded through the walker's seven call sites (five recursive, two in
;; `validate_then_form`), plus `lookup_field_types` + `check_then_field_type` — D10's own
;; producer, called unchanged — in the aggregate branch's kwargs and positional arms. No new error
;; kind: `RhsFieldTypeMismatch` is the same claim at a different position. No `typing.rs` change,
;; no engine change.
;;
;; The SUBJECT rule that would live here is now a rule-compile refusal, so KEEPING IT HERE WOULD
;; RED the `every_wat_scripts_file_loads` gate. It lives, with its positional / depth-2 /
;; match-arm-body siblings and their `.edn` goldens, at
;; `tests/rete/probe_arc278_D11_nested_then_field_types.rs`:
;;
;;   (:wat::rete::defrule :nh::bad
;;     :when [(:nh::Box (?k <- :k) (?s <- :s))]
;;     :then [(:nh::Outer :i (:nh::Inner :n ?s))])
;;
;;   #wat.rete/RhsFieldTypeMismatch — "defrule `nh::bad`: `:then` insert of `:nh::Inner` fills
;;   field `:n`, declared `:wat::core::i64` (rete `i64`), with operand `?s`, whose type is
;;   `string`" — and the caret lands on `?s`, not on the outer fact form.
;;
;; ANTI-VACUITY: the well-typed nested rule below must derive, or this file proves nothing. What
;; it now proves is the other half of the cure — that a WELL-TYPED nested constructor still
;; compiles and fires.
(:wat::core::defrecord :d11r::Box   [k <- :wat::core::i64  s <- :wat::core::String])
(:wat::core::defrecord :d11r::Inner [n <- :wat::core::i64])
(:wat::core::defrecord :d11r::Outer [i <- :d11r::Inner])

(:wat::rete::defrule :d11r::ok
  :when [(:d11r::Box (?k <- :k))]
  :then [(:d11r::Outer :i (:d11r::Inner :n ?k))])          ;; i64 into i64, NESTED — the CONTROL

(:wat::rete::defquery :d11r::qo :params [] :when [(?f <- :d11r::Outer)])

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::let
    [s0 (:wat::core::match (:wat::rete::compile-all (:wat::rete::collect-rules :d11r)
                             (:wat::core::PersistentVector (:d11r::qo)))
          ((:wat::rete::CompileOutcome::Compiled __s) __s)
          ((:wat::rete::CompileOutcome::MayNotTerminate __r __f) (:wat::kernel::assertion-failed! "mnt" :wat::core::None :wat::core::None)))
     s1 (:wat::core::match (:wat::rete::insert s0 (:d11r::Box :k 7 :s "nested-string"))
          ((:wat::rete::InsertOutcome::Inserted __x) __x)
          ((:wat::rete::InsertOutcome::MemoryCeilingExceeded __a __b __c) (:wat::kernel::assertion-failed! "ceiling" :wat::core::None :wat::core::None)))]
    (:wat::core::match (:wat::rete::fire-rules s1)
      ((:wat::rete::FireOutcome::Fired __f)
        (:wat::core::do
          (:wat::kernel::println "CONTROL nested Outer.i.n:")
          (:wat::kernel::println (:wat::core::format "{v}" :v
            (:d11r::Inner/n (:d11r::Outer/i
              (:wat::core::Option/expect
                (:wat::core::PersistentMap/get (:wat::core::first (:wat::rete::query __f (:d11r::qo))) "?f")
                "control")))))
          (:wat::kernel::println "SUBJECT (was `:nh::bad`) is now refused at rule-compile — see the header")))
      ((:wat::rete::FireOutcome::MemoryCeilingExceeded __a __b __c) (:wat::kernel::println "ceil"))
      ((:wat::rete::FireOutcome::RoundCapExceeded __a __b) (:wat::kernel::println "roundcap")))))
