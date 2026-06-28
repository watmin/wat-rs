;; tests/comms/wat_arc113_cross_fork_cascade.wat — co-located fixture for the cross-fork cascade probe,
;; slurped via startup_beside(file!()). No placeholder main — startup_beside loads defns only.

(:wat::core::defn :my::compute [] -> :wat::core::Vector<wat::core::String>
  (:wat::core::let
    [r
      (:wat::test::run-hermetic
        (:wat::test::assert-eq 1 2))
     fail
      (:wat::kernel::RunResult/failure r)
     rendered
      (:wat::core::match fail -> :wat::core::Vector<wat::core::String>
        ((:wat::core::Some f)
         (:wat::core::Vector :wat::core::String
           (:wat::kernel::Failure/message f)
           (:wat::core::match (:wat::kernel::Failure/actual f) -> :wat::core::String
             ((:wat::core::Some a) a)
             (:wat::core::None ":None"))
           (:wat::core::match (:wat::kernel::Failure/expected f) -> :wat::core::String
             ((:wat::core::Some e) e)
             (:wat::core::None ":None"))))
        (:wat::core::None
         (:wat::core::Vector :wat::core::String "NO-FAILURE")))]
    rendered))
