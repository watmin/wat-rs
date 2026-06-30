;; Co-located fixture for probe_arc259_s2d_raii_hinge.rs — blocked_peer_dropped_without_close_does_not_hang.
;; THE HINGE: peer blocked on recv' dropped at scope-exit; RAII drains before join -> no hang. Returns 7.

(:wat::core::defn :user::compute [] -> :wat::core::i64
  (:wat::core::do
    (:wat::core::let
      [peer (:wat::kernel::spawn-program' (:wat::spawn::thread)
              (:wat::core::fn [self <- :wat::kernel::ThreadSelfPeer'<wat::core::i64,wat::core::i64>] -> :wat::core::nil
                (:wat::kernel::send' self (:wat::kernel::recv' self))))]
      nil)
    7))

