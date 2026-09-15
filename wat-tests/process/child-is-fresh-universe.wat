;; A process child is a fresh universe — it does not inherit parent defns.
;; BRIEF-3: pins probe-child-inherits-defns.wat's answer as a gate rune target.
(:wat::test::deftest :wat-tests::process::child-is-fresh-universe
  (:wat::core::let
    [p (:wat::test::spawn-peer (:wat::spawn::process)
         ;; rune:lint(nested-program, expected) — test(deftest_wat_tests_process_child_is_fresh_universe)
         (:wat::core::forms
           (:wat::core::defn :user::main [] -> :wat::core::nil
             (:probe::parent-only 1))))]
    (:wat::core::match (:wat::kernel::recv p)
      [:wat::kernel::RecvOutcome.Message {:msg _m}
        (:wat::kernel::assertion-failed! :message "child inherited a parent-only name")]
      [:wat::kernel::RecvOutcome.Lost {:cause _cause} nil]
      [:wat::kernel::RecvOutcome.Stopped {}
        (:wat::kernel::assertion-failed! :message "recv': stopped")]
      [:wat::kernel::RecvOutcome.Closed {}
        (:wat::kernel::assertion-failed! :message "child closed; expected Lost on unknown parent defn")])))
