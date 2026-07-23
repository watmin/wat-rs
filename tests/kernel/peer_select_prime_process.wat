;; Co-located fixture for peer_select_prime_process.rs — slurped via startup_beside(file!()).
;; #[ignore] process-tier probe (arc 214 Stone 4.6b).

(:wat::core::defn :user::compute [] -> :wat::spawn::ServiceEvent<wat::core::i64,wat::core::i64>
  (:wat::core::let
    [a (:wat::kernel::spawn-program' (:wat::spawn::process)
          (:wat::core::forms
            (:wat::core::defn :user::main [] -> :wat::core::nil
              (:wat::core::let
                [n (:wat::kernel::readln )
                 _ (:wat::kernel::println (:wat::core::i64::+ n 1))]
                nil))))
     b (:wat::kernel::spawn-program' (:wat::spawn::process)
          (:wat::core::forms
            (:wat::core::defn :user::main [] -> :wat::core::nil
              (:wat::core::let
                [n (:wat::kernel::readln )
                 _ (:wat::kernel::println (:wat::core::i64::+ n 1))]
                nil))))
     _ (:wat::core::match (:wat::kernel::send' b 98) (:wat::kernel::SendOutcome::Sent nil) (:wat::kernel::SendOutcome::Closed nil) ((:wat::kernel::SendOutcome::Lost _c) nil))
     picked (:wat::kernel::select' [a b])]
    picked))

