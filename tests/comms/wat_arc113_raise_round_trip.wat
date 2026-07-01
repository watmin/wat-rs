;; tests/comms/wat_arc113_raise_round_trip.wat — co-located fixture for the raise round-trip probe,
;; slurped via startup_beside(file!()). No placeholder main — startup_beside loads defns only.
;;
;; Arc 296 re-gate: raise! now requires :wat::core::Error. The round-trip is:
;;   (raise! (Fault/of "arc113-raise-data")) → EDN string in Failure/message →
;;   recovered String that contains "arc113-raise-data".
;; Returns Option<String> (the raw Failure/message EDN) instead of Option<HolonAST>
;; because edn::read now returns Value::Aggregate for known record types, not HolonAST.

(:wat::core::defn :my::compute [] -> :wat::core::Option<wat::core::String>
  (:wat::core::let
    [r
      (:wat::test::run-thread
        (:wat::kernel::raise!
          (:wat::core::Fault/of "arc113-raise-data")))
     fail
      (:wat::kernel::RunResult/failure r)]
    (:wat::core::match fail -> :wat::core::Option<wat::core::String>
      ((:wat::core::Some f)
       (:wat::core::Some (:wat::kernel::Failure/message f)))
      (:wat::core::None :wat::core::None))))
