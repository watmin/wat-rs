;; tests/diagnostics/probe_arc296_raise_gate.wat — co-located fixture for the raise gate probe.
;;
;; Arc 296 S3: :wat::kernel::raise! re-gated to require :wat::core::Error.
;;
;; GREEN after: startup boots and main runs. Proves:
;; (a) :wat::core::Fault/of "boom" type-checks as :wat::core::Error (satisfies the surface).
;; (b) The sandboxed raise is caught and the Failure/message contains "boom".
;; (c) Passing a Fault to [e <- :wat::core::Error] param type-checks.

(:wat::core::defn :probe::accept-error [e <- :wat::core::Error] -> :wat::core::String
  (:wat::core::Error/message e))

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::let
    [;; (c) Fault/of satisfies :wat::core::Error structurally.
     msg  (:probe::accept-error (:wat::core::Fault/of "boom"))
     ;; (b) Sandboxed raise is caught; RunResult/failure is Some.
     r    (:wat::test::run-thread
            (:wat::kernel::raise! (:wat::core::Fault/of "boom")))
     fail (:wat::kernel::RunResult/failure r)
     ;; got-failure: 1 if Some (caught), 0 if None (escaped).
     got-failure (:wat::core::match fail 
                   ((:wat::core::Some _) 1)
                   (:wat::core::None     0))]
    (:wat::core::do
      ;; Verify the error message round-trips through accept-error.
      (:wat::test::assert-eq msg "boom")
      ;; Verify the sandboxed raise produced a Failure.
      (:wat::test::assert-eq got-failure 1))))
