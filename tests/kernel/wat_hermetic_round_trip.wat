;; Co-located fixture for wat_hermetic_round_trip.rs — slurped via startup_beside(file!()).

(:wat::core::defn :my::compute-stdout-count [] -> :wat::core::i64
  (:wat::core::let
    [p (:wat::kernel::spawn-program' (:wat::spawn::process)
         (:wat::core::forms
           (:wat::core::defn :user::main [] -> :wat::core::nil
             (:wat::kernel::println "tada!"))))]
    (:wat::core::match (:wat::kernel::recv' p)
      ((:wat::kernel::RecvOutcome::Message _m) 1)
      ((:wat::kernel::RecvOutcome::Lost cause)
        (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message cause) :wat::core::None :wat::core::None))
      (:wat::kernel::RecvOutcome::Closed
        (:wat::kernel::assertion-failed! "compute-stdout-count: child closed before sending its value" :wat::core::None :wat::core::None)))))

(:wat::core::defn :my::compute-eval-in-outer [] -> :wat::core::i64
  (:wat::core::let
    [p (:wat::kernel::spawn-program' (:wat::spawn::process)
         (:wat::core::forms
           (:wat::core::defn :user::main [] -> :wat::core::nil
             (:wat::kernel::println 42))))]
    (:wat::core::match (:wat::kernel::recv' p)
      ((:wat::kernel::RecvOutcome::Message m) m)
      ((:wat::kernel::RecvOutcome::Lost cause)
        (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message cause) :wat::core::None :wat::core::None))
      (:wat::kernel::RecvOutcome::Closed
        (:wat::kernel::assertion-failed! "compute-eval-in-outer: child closed before sending its value" :wat::core::None :wat::core::None)))))
