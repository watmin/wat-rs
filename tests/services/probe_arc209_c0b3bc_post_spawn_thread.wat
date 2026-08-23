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
             ((:wat::kernel::ConnectOutcome::Connected p) p)
             ((:wat::kernel::ConnectOutcome::Refused _c)
               (:wat::kernel::assertion-failed! "connect': refused binding the hook channel" :wat::core::None :wat::core::None))
             ((:wat::kernel::ConnectOutcome::Rejected _c)
               (:wat::kernel::assertion-failed! "connect': rejected binding the hook channel" :wat::core::None :wat::core::None))
             ((:wat::kernel::ConnectOutcome::Failed _c)
               (:wat::kernel::assertion-failed! "connect': failed binding the hook channel" :wat::core::None :wat::core::None)))
     rx    (:wat::core::match (:wat::kernel::accept lis)
             ((:wat::kernel::AcceptOutcome::Accepted p) p)
             (:wat::kernel::AcceptOutcome::Closed
               (:wat::kernel::assertion-failed! "accept': listener closed before the hook channel was accepted" :wat::core::None :wat::core::None))
             ((:wat::kernel::AcceptOutcome::Failed _c)
               (:wat::kernel::assertion-failed! "accept': failed accepting the hook channel" :wat::core::None :wat::core::None)))
     _thr  (:wat::test::spawn-peer
             (:wat::spawn::thread/post-spawn
               (:wat::core::fn [launch <- :wat::spawn::ThreadLaunch] -> :wat::core::nil
                 (:wat::core::let [_ (:wat::core::match (:wat::kernel::send tx 777) (:wat::kernel::SendOutcome::Sent nil) (:wat::kernel::SendOutcome::Closed nil) (:wat::kernel::SendOutcome::Stopped nil) ((:wat::kernel::SendOutcome::Lost _c) nil))] nil)))
             (:wat::core::fn [self <- (:wat::kernel::ThreadSelfPeer :- [:wat::core::i64 :wat::core::i64])] -> :wat::core::nil
               nil))
     sentinel (:wat::core::match (:wat::kernel::recv rx)
                ((:wat::kernel::RecvOutcome::Message m) m)
                ((:wat::kernel::RecvOutcome::Lost cause)
                  (:wat::kernel::assertion-failed! (:wat::kernel::LociDiedError/message cause) :wat::core::None :wat::core::None))
                (:wat::kernel::RecvOutcome::Stopped
                  (:wat::kernel::assertion-failed! "recv': stopped before the post-spawn hook sent the sentinel — the peer was ALIVE" :wat::core::None :wat::core::None))
                (:wat::kernel::RecvOutcome::Closed
                  (:wat::kernel::assertion-failed! "recv': rx closed before the post-spawn hook sent the sentinel" :wat::core::None :wat::core::None)))]
    sentinel))
