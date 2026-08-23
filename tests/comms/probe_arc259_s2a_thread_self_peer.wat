;; tests/comms/probe_arc259_s2a_thread_self_peer.wat — co-located fixture slurped via startup_beside(file!()).
;; Arc 259 Stone S2a — ThreadProg self-peer model on the unified pipes-only Peer.
;; The thread prog drives its OWN pipes-only self-peer: recv the parent's 42, echo it back.

(:wat::core::defn :user::compute [] -> :wat::core::i64
  (:wat::core::let [peer (:wat::test::spawn-peer (:wat::spawn::thread)
                           (:wat::core::fn [self <- (:wat::kernel::ThreadSelfPeer :- [:wat::core::i64 :wat::core::i64])] -> :wat::core::nil
                             (:wat::core::match
                               (:wat::kernel::send self
                                 (:wat::core::match (:wat::kernel::recv self)
                                   ((:wat::kernel::RecvOutcome::Message m) m)
                                   ((:wat::kernel::RecvOutcome::Lost cause)
                                     (:wat::kernel::assertion-failed! (:wat::kernel::LociDiedError/message cause) :wat::core::None :wat::core::None))
                                   (:wat::kernel::RecvOutcome::Stopped
                                     (:wat::kernel::assertion-failed! "recv': stopped — the substrate was asked to stop; the peer was ALIVE and the channel open" :wat::core::None :wat::core::None))
                                   (:wat::kernel::RecvOutcome::Closed
                                     (:wat::kernel::assertion-failed! "recv': self closed unexpectedly" :wat::core::None :wat::core::None))))
                               (:wat::kernel::SendOutcome::Sent nil)
                               (:wat::kernel::SendOutcome::Closed nil)
                               ((:wat::kernel::SendOutcome::Lost _c) nil)
                               (:wat::kernel::SendOutcome::Stopped nil)))) ;; arc 278 #73 — fire-and-forget echo; outcome ignored uniformly regardless of cause
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
                           (:wat::kernel::assertion-failed! "recv': peer closed unexpectedly" :wat::core::None :wat::core::None)))]
    got))

