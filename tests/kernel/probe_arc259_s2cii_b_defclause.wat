;; Co-located fixture for probe_arc259_s2cii_b_defclause.rs — s2cii_b_two_arg_host_dispatch.
;; 2-arg (spawn-program' (thread) <self-peer-prog>) dispatches on ThreadOpts; echoes 42.

(:wat::core::defn :user::compute [] -> :wat::core::i64
  (:wat::core::let [peer (:wat::test::spawn-peer (:wat::spawn::thread)
                           (:wat::core::fn [self <- (:wat::kernel::ThreadSelfPeer :- [:wat::core::i64 :wat::core::i64])] -> :wat::core::nil
                             (:wat::core::match (:wat::kernel::recv self)
                               ((:wat::kernel::RecvOutcome::Message m)
                                 (:wat::core::match (:wat::kernel::send self m)
                                   (:wat::kernel::SendOutcome::Sent nil)
                                   (:wat::kernel::SendOutcome::Closed nil)
                                   ((:wat::kernel::SendOutcome::Lost _c) nil)
                                   (:wat::kernel::SendOutcome::Stopped nil)))  ;; arc 278 #73 — fire-and-forget echo; outcome ignored uniformly regardless of cause
                               ((:wat::kernel::RecvOutcome::Lost cause)
                                 (:wat::kernel::assertion-failed! (:wat::kernel::LociDiedError/message cause) :wat::core::None :wat::core::None))
                               (:wat::kernel::RecvOutcome::Stopped
                                 (:wat::kernel::assertion-failed! "recv': stopped — the substrate was asked to stop; the peer was ALIVE and the channel open" :wat::core::None :wat::core::None))
                               (:wat::kernel::RecvOutcome::Closed
                                 (:wat::kernel::assertion-failed! "recv': self closed unexpectedly" :wat::core::None :wat::core::None)) (:wat::kernel::RecvOutcome::TimedOut (:wat::kernel::assertion-failed! "recv: timed out — the peer is alive and silent" :wat::core::None :wat::core::None)))))
                    _ (:wat::core::match (:wat::kernel::send peer 42)
                        (:wat::kernel::SendOutcome::Sent nil)
                        (:wat::kernel::SendOutcome::Closed nil)
                        ((:wat::kernel::SendOutcome::Lost _c) nil)
                        (:wat::kernel::SendOutcome::Stopped nil)) ;; arc 278 #73 — fire-and-forget request; outcome ignored uniformly regardless of cause
                    got (:wat::core::match (:wat::kernel::recv peer)
                          ((:wat::kernel::RecvOutcome::Message m) m)
                          ((:wat::kernel::RecvOutcome::Lost cause)
                            (:wat::kernel::assertion-failed! (:wat::kernel::LociDiedError/message cause) :wat::core::None :wat::core::None))
                          (:wat::kernel::RecvOutcome::Stopped
                            (:wat::kernel::assertion-failed! "recv': stopped — the substrate was asked to stop; the peer was ALIVE and the channel open" :wat::core::None :wat::core::None))
                          (:wat::kernel::RecvOutcome::Closed
                            (:wat::kernel::assertion-failed! "recv': peer closed before echoing" :wat::core::None :wat::core::None)) (:wat::kernel::RecvOutcome::TimedOut (:wat::kernel::assertion-failed! "recv: timed out — the peer is alive and silent" :wat::core::None :wat::core::None)))]
    got))

