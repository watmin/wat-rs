;; Co-located fixture for probe_arc214_alpha_crash_autoraise.rs — slurped via startup_beside(file!()).
;; #[ignore] process-tier probe (arc 214 1b-ii-α).

(:wat::core::defn :user::compute [] -> :wat::core::i64
  (:wat::core::let
    [peer (:wat::kernel::spawn-program' (:wat::spawn::process)
            (:wat::core::forms
              (:wat::core::defn :user::main [] -> :wat::core::nil
                (:wat::core::let
                  [n (:wat::kernel::readln )
                   _ (:wat::kernel::println (:wat::core::i64::/ 100 n))]
                  nil))))
     _   (:wat::kernel::send' peer 0)
     got (:wat::core::match (:wat::kernel::recv' peer)
           ((:wat::kernel::RecvOutcome::Message m) m)
           ((:wat::kernel::RecvOutcome::Lost cause)
             (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message cause) :wat::core::None :wat::core::None))
           (:wat::kernel::RecvOutcome::Closed
             (:wat::kernel::assertion-failed! "recv': peer closed" :wat::core::None :wat::core::None)))]
    got))

