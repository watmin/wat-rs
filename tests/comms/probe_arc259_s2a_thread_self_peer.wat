;; tests/comms/probe_arc259_s2a_thread_self_peer.wat — co-located fixture slurped via startup_beside(file!()).
;; Arc 259 Stone S2a — ThreadProg self-peer model on the unified pipes-only Peer.
;; The thread prog drives its OWN pipes-only self-peer: recv the parent's 42, echo it back.

(:wat::core::defn :user::compute [] -> :wat::core::i64
  (:wat::core::let [peer (:wat::kernel::spawn-program' (:wat::spawn::thread)
                           (:wat::core::fn [self <- :wat::kernel::ThreadSelfPeer'<wat::core::i64,wat::core::i64>] -> :wat::core::nil
                             (:wat::kernel::send' self (:wat::kernel::recv' self))))
                   _ (:wat::kernel::send' peer 42)
                   got (:wat::kernel::recv' peer)]
    got))

