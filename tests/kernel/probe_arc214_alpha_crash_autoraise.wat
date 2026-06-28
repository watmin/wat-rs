;; Co-located fixture for probe_arc214_alpha_crash_autoraise.rs — slurped via startup_beside(file!()).
;; #[ignore] process-tier probe (arc 214 1b-ii-α).

(:wat::core::defn :user::compute [] -> :wat::core::i64
  (:wat::core::let
    [peer (:wat::kernel::spawn-program' (:wat::spawn::process)
            (:wat::core::forms
              (:wat::core::defn :user::main [] -> :wat::core::nil
                (:wat::core::let
                  [n (:wat::kernel::readln -> :wat::core::i64)
                   _ (:wat::kernel::println (:wat::core::i64::/ 100 n))]
                  nil))))
     _   (:wat::kernel::send' peer 0)
     got (:wat::kernel::recv' peer)]
    got))

