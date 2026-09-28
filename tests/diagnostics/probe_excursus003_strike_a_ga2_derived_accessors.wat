;; tests/diagnostics/probe_excursus003_strike_a_ga2_derived_accessors.wat — co-located
;; fixture for GA2 (excursus 003 strike A, BRIEF-shape-strike-A-one-death-shape.md):
;; `Failure/actual` / `Failure/expected` are DERIVED accessors now (F2) — `Some` when
;; the death is an assertion (`error` is `:wat::runtime::AssertionFailed`), `None`
;; for every other death (here, a runtime error — a division by zero has no
;; actual/expected value to attribute).

(:wat::core::defrecord :ga2::Result
  [actual   <- (:wat::core::Option :- [:wat::core::String])
   expected <- (:wat::core::Option :- [:wat::core::String])])

(:wat::core::defn :ga2::assertion-report [] -> :ga2::Result
  (:wat::core::let
    [p (:wat::test::spawn-peer (:wat::spawn::process)
         (:wat::core::forms
           (:wat::core::defn :user::main [] -> :wat::core::nil
             (:wat::test::assert-eq 1 2))))]
    (:wat::core::match (:wat::kernel::recv p)
      [:wat::kernel::RecvOutcome.Message {:msg _m}
        (:ga2::Result :actual :wat::core::Option.None :expected :wat::core::Option.None)]
      [:wat::kernel::RecvOutcome.Lost {:cause cause}
        (:wat::core::match cause
          [:wat::kernel::LociDiedError.Panic {:failure f}
            (:ga2::Result :actual (:wat::kernel::Failure/actual f) :expected (:wat::kernel::Failure/expected f))]
          [_ (:ga2::Result :actual :wat::core::Option.None :expected :wat::core::Option.None)])]
      [:wat::kernel::RecvOutcome.Stopped {}
        (:ga2::Result :actual :wat::core::Option.None :expected :wat::core::Option.None)]
      [:wat::kernel::RecvOutcome.Closed {}
        (:ga2::Result :actual :wat::core::Option.None :expected :wat::core::Option.None)])))

(:wat::core::defn :ga2::runtime-error-report [] -> :ga2::Result
  (:wat::core::let
    [p (:wat::test::spawn-peer (:wat::spawn::process)
         (:wat::core::forms
           (:wat::core::defn :user::main [] -> :wat::core::nil
             (:wat::core::let [_ (:wat::i64::/ 1 0)] nil))))]
    (:wat::core::match (:wat::kernel::recv p)
      [:wat::kernel::RecvOutcome.Message {:msg _m}
        (:ga2::Result :actual :wat::core::Option.None :expected :wat::core::Option.None)]
      [:wat::kernel::RecvOutcome.Lost {:cause cause}
        (:wat::core::match cause
          [:wat::kernel::LociDiedError.RuntimeError {:failure f}
            (:ga2::Result :actual (:wat::kernel::Failure/actual f) :expected (:wat::kernel::Failure/expected f))]
          [_ (:ga2::Result :actual :wat::core::Option.None :expected :wat::core::Option.None)])]
      [:wat::kernel::RecvOutcome.Stopped {}
        (:ga2::Result :actual :wat::core::Option.None :expected :wat::core::Option.None)]
      [:wat::kernel::RecvOutcome.Closed {}
        (:ga2::Result :actual :wat::core::Option.None :expected :wat::core::Option.None)])))
