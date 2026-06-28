;; Co-located fixture for peer_verb_round_trip_process.rs — slurped via startup_beside(file!()).
;; #[ignore] process-tier probe (arc 214 Stone 4.6a-ii).

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

