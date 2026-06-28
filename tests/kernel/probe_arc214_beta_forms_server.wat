;; Co-located fixture for probe_arc214_beta_forms_server.rs — slurped via startup_beside(file!()).
;; #[ignore] process-tier FM-2-bis probe (arc 214 1b-ii-β).

(:wat::core::defn :user::compute [] -> :wat::core::i64
  (:wat::core::let
    [peer (:wat::kernel::spawn-program' (:wat::spawn::process)
            (:wat::core::forms
              (:wat::core::defn :user::main [] -> :wat::core::nil
                (:wat::core::let
                  [n (:wat::kernel::readln -> :wat::core::i64)
                   _ (:wat::kernel::println (:wat::core::i64::+ n 1))]
                  nil))))
     _   (:wat::kernel::send' peer 41)
     got (:wat::kernel::recv' peer)]
    got))

