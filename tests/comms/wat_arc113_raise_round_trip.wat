;; tests/comms/wat_arc113_raise_round_trip.wat — co-located fixture for the raise round-trip probe,
;; slurped via startup_beside(file!()). No placeholder main — startup_beside loads defns only.

(:wat::core::defn :my::compute [] -> :wat::core::Option<wat::holon::HolonAST>
  (:wat::core::let
    [r
      (:wat::test::run-thread
        (:wat::kernel::raise!
          (:wat::holon::leaf 42)))
     fail
      (:wat::kernel::RunResult/failure r)
     recovered
      (:wat::core::match fail -> :wat::core::Option<wat::holon::HolonAST>
        ((:wat::core::Some f)
         (:wat::core::Some (:wat::edn::read (:wat::kernel::Failure/message f))))
        (:wat::core::None :wat::core::None))]
    recovered))
