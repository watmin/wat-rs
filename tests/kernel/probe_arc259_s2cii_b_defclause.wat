;; Co-located fixture for probe_arc259_s2cii_b_defclause.rs — s2cii_b_two_arg_host_dispatch.
;; 2-arg (spawn-program' (thread) <self-peer-prog>) dispatches on ThreadOpts; echoes 42.

(:wat::core::defn :user::compute [] -> :wat::core::i64
  (:wat::core::let [peer (:wat::kernel::spawn-program' (:wat::spawn::thread)
                           (:wat::core::fn [self <- :wat::kernel::Peer'<wat::core::i64,wat::core::i64>] -> :wat::core::nil
                             (:wat::kernel::send' self (:wat::kernel::recv' self))))
                    _ (:wat::kernel::send' peer 42)
                    got (:wat::kernel::recv' peer)]
    got))

