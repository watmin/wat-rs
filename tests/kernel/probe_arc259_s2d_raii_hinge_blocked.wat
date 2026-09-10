;; Co-located fixture for probe_arc259_s2d_raii_hinge.rs — blocked_peer_dropped_without_close_does_not_hang.
;; THE HINGE: peer blocked on recv' dropped at scope-exit; RAII drains before join -> no hang. Returns 7.

(:wat::core::defn :user::compute [] -> :wat::core::i64
  (:wat::core::do
    (:wat::core::let
      [peer (:wat::test::spawn-peer (:wat::spawn::thread)
              (:wat::core::fn [self <- (:wat::kernel::ThreadSelfPeer :- [:wat::core::i64 :wat::core::i64])] -> :wat::core::nil
                (:wat::core::match (:wat::kernel::recv self)
                  [:wat::kernel::RecvOutcome.Message {:msg m}
                    (:wat::core::match (:wat::kernel::send self m)
                      [:wat::kernel::SendOutcome.Sent {} nil]
                      [:wat::kernel::SendOutcome.Closed {} nil]
                      [:wat::kernel::SendOutcome.Lost {:cause _c} nil]
                      [:wat::kernel::SendOutcome.Stopped {} nil])]  ;; arc 278 #73 — fire-and-forget echo; outcome ignored uniformly regardless of cause
                  [:wat::kernel::RecvOutcome.Lost {:cause cause}
                    (:wat::kernel::assertion-failed! :message (:wat::kernel::LociDiedError/message cause))]
                  [:wat::kernel::RecvOutcome.Stopped {}
                    (:wat::kernel::assertion-failed! :message "recv': stopped — the substrate was asked to stop; the peer was ALIVE and the channel open")]
                  [:wat::kernel::RecvOutcome.Closed {}
                    (:wat::kernel::assertion-failed! :message "recv': self closed unexpectedly")])))]
      nil)
    7))

