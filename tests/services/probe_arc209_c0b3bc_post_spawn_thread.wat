;; Proof 2: thread post-spawn hook fires owner-side with the empty ThreadLaunch.
;;
;; Arc 278 — the hook is an owner-side CALLBACK and cannot return a value, so the sentinel
;; crosses a channel. `peer-pair'` (the annihilated bare-pair primitive) is replaced by the
;; substrate's own connection path: listener' binds, connect' takes the client end, accept'
;; the server end. No spawn, and the ceremony every real consumer pays.
(:wat::core::defn :user::compute [] -> :wat::core::i64
  (:wat::core::let
    [bound (:wat::kernel::listener (:wat::spawn::thread) :wat::core::i64 :wat::core::i64)
     lis   (:wat::spawn::Bound/listener bound)
     addr  (:wat::spawn::Bound/address bound)
     tx    (:wat::core::match (:wat::kernel::connect addr)
             [:wat::kernel::ConnectOutcome.Connected {:peer p} p]
             [:wat::kernel::ConnectOutcome.Refused {:cause _c}
               (:wat::kernel::assertion-failed! :message "connect': refused binding the hook channel")]
             [:wat::kernel::ConnectOutcome.Rejected {:cause _c}
               (:wat::kernel::assertion-failed! :message "connect': rejected binding the hook channel")]
             [:wat::kernel::ConnectOutcome.Failed {:cause _c}
               (:wat::kernel::assertion-failed! :message "connect': failed binding the hook channel")])
     rx    (:wat::core::match (:wat::kernel::accept lis)
             [:wat::kernel::AcceptOutcome.Accepted {:peer p} p]
             [:wat::kernel::AcceptOutcome.Closed {}
               (:wat::kernel::assertion-failed! :message "accept': listener closed before the hook channel was accepted")]
             [:wat::kernel::AcceptOutcome.Failed {:cause _c}
               (:wat::kernel::assertion-failed! :message "accept': failed accepting the hook channel")])
     _thr  (:wat::test::spawn-peer
             (:wat::spawn::thread/post-spawn
               (:wat::core::fn [launch <- :wat::spawn::ThreadLaunch] -> :wat::core::nil
                 (:wat::core::let [_ (:wat::core::match (:wat::kernel::send tx 777) [:wat::kernel::SendOutcome.Sent {} nil] [:wat::kernel::SendOutcome.Closed {} nil] [:wat::kernel::SendOutcome.Stopped {} nil] [:wat::kernel::SendOutcome.Lost {:cause _c} nil])] nil)))
             (:wat::core::fn [self <- (:wat::kernel::ThreadSelfPeer :- [:wat::core::i64 :wat::core::i64])] -> :wat::core::nil
               nil))
     sentinel (:wat::core::match (:wat::kernel::recv rx)
                [:wat::kernel::RecvOutcome.Message {:msg m} m]
                [:wat::kernel::RecvOutcome.Lost {:cause cause}
                  (:wat::kernel::assertion-failed! :message (:wat::kernel::LociDiedError/message cause))]
                [:wat::kernel::RecvOutcome.Stopped {}
                  (:wat::kernel::assertion-failed! :message "recv': stopped before the post-spawn hook sent the sentinel — the peer was ALIVE")]
                [:wat::kernel::RecvOutcome.Closed {}
                  (:wat::kernel::assertion-failed! :message "recv': rx closed before the post-spawn hook sent the sentinel")])]
    sentinel))
