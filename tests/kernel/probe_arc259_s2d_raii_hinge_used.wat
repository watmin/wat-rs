;; Co-located fixture for probe_arc259_s2d_raii_hinge.rs — peer_used_then_dropped_without_close.
;; Peer used (send -> echo -> recv) then DROPPED without close': RAII reaps; returns 99.

(:wat::core::defn :user::compute [] -> :wat::core::i64
  (:wat::core::let
    [peer (:wat::kernel::spawn-program' (:wat::spawn::thread)
            (:wat::core::fn [self <- :wat::kernel::Peer'<wat::core::i64,wat::core::i64>] -> :wat::core::nil
              (:wat::kernel::send' self (:wat::kernel::recv' self))))
     _ (:wat::kernel::send' peer 99)
     got (:wat::kernel::recv' peer)]
    got))

