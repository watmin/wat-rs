;; Co-located fixture for wat_hermetic_round_trip.rs — slurped via startup_beside(file!()).

(:wat::core::defn :my::compute-stdout-count [] -> :wat::core::i64
  (:wat::core::let
    [p (:wat::test::spawn-peer (:wat::spawn::process)
         (:wat::core::forms
           (:wat::core::defn :user::main [] -> :wat::core::nil
             (:wat::kernel::println "tada!"))))]
    (:wat::core::match (:wat::kernel::recv p)
      [:wat::kernel::RecvOutcome::Message {:msg _m} 1]
      [:wat::kernel::RecvOutcome::Lost {:cause cause}
        (:wat::kernel::assertion-failed! :message (:wat::kernel::LociDiedError/message cause))]
      [:wat::kernel::RecvOutcome::Stopped {}
        (:wat::kernel::assertion-failed! :message "recv': stopped — the substrate was asked to stop; the peer was ALIVE and the channel open")]
      [:wat::kernel::RecvOutcome::Closed {}
        (:wat::kernel::assertion-failed! :message "compute-stdout-count: child closed before sending its value")])))

(:wat::core::defn :my::compute-eval-in-outer [] -> :wat::core::i64
  (:wat::core::let
    [p (:wat::test::spawn-peer (:wat::spawn::process)
         (:wat::core::forms
           (:wat::core::defn :user::main [] -> :wat::core::nil
             (:wat::kernel::println 42))))]
    (:wat::core::match (:wat::kernel::recv p)
      [:wat::kernel::RecvOutcome::Message {:msg m} m]
      [:wat::kernel::RecvOutcome::Lost {:cause cause}
        (:wat::kernel::assertion-failed! :message (:wat::kernel::LociDiedError/message cause))]
      [:wat::kernel::RecvOutcome::Stopped {}
        (:wat::kernel::assertion-failed! :message "recv': stopped — the substrate was asked to stop; the peer was ALIVE and the channel open")]
      [:wat::kernel::RecvOutcome::Closed {}
        (:wat::kernel::assertion-failed! :message "compute-eval-in-outer: child closed before sending its value")])))
