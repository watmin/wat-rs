;; Co-located fixture for probe_arc259_s2ci_spawn_thread_prime.rs — s2ci_spawn_thread_prime_round_trip.
;; spawn-program' (thread) spawns a thread peer; self-peer prog echoes 42.

(:wat::core::defn :user::compute [] -> :wat::core::i64
  (:wat::core::let [peer (:wat::test::spawn-peer (:wat::spawn::thread)
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
                                 (:wat::kernel::assertion-failed! :message "recv': self closed unexpectedly")])))
                    _ (:wat::core::match (:wat::kernel::send peer 42)
                        [:wat::kernel::SendOutcome.Sent {} nil]
                        [:wat::kernel::SendOutcome.Closed {} nil]
                        [:wat::kernel::SendOutcome.Lost {:cause _c} nil]
                        [:wat::kernel::SendOutcome.Stopped {} nil]) ;; arc 278 #73 — fire-and-forget request; outcome ignored uniformly regardless of cause
                    got (:wat::core::match (:wat::kernel::recv peer)
                          [:wat::kernel::RecvOutcome.Message {:msg m} m]
                          [:wat::kernel::RecvOutcome.Lost {:cause cause}
                            (:wat::kernel::assertion-failed! :message (:wat::kernel::LociDiedError/message cause))]
                          [:wat::kernel::RecvOutcome.Stopped {}
                            (:wat::kernel::assertion-failed! :message "recv': stopped — the substrate was asked to stop; the peer was ALIVE and the channel open")]
                          [:wat::kernel::RecvOutcome.Closed {}
                            (:wat::kernel::assertion-failed! :message "recv': peer closed before echoing")])]
    got))

