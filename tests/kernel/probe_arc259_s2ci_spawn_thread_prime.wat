;; Co-located fixture for probe_arc259_s2ci_spawn_thread_prime.rs — s2ci_spawn_thread_prime_round_trip.
;; spawn-program' (thread) spawns a thread peer; self-peer prog echoes 42.

(:wat::core::defn :user::compute [] -> :wat::core::i64
  (:wat::core::let [peer (:wat::kernel::spawn-program' (:wat::spawn::thread)
                           (:wat::core::fn [self <- :wat::kernel::ThreadSelfPeer'<wat::core::i64,wat::core::i64>] -> :wat::core::nil
                             (:wat::kernel::send' self (:wat::kernel::recv' self))))
                    _ (:wat::kernel::send' peer 42)
                    got (:wat::kernel::recv' peer)]
    got))

