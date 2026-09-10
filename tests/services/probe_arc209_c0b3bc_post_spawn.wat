;; Proof 1: process post-spawn hook receives the child pid, owner-side.
;;
;; Arc 278 — the hook is an owner-side CALLBACK; it cannot return a value to its caller, so
;; the pid must cross a channel to reach the enclosing `let`. That host used to be
;; `peer-pair'`, a bare-pair primitive minting two connected ends without spawning; it is
;; annihilated with the rest of the hand-rolled IPC. The replacement is the substrate's own
;; connection path — `listener'` binds a rendezvous, `connect'` takes the client end,
;; `accept'` the server end. Still no spawn, and now the same ceremony every real consumer pays.
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
     _proc (:wat::test::spawn-peer
             (:wat::spawn::process/post-spawn
               (:wat::core::fn [launch <- :wat::spawn::ProcessLaunch] -> :wat::core::nil
                 (:wat::core::let [_ (:wat::core::match (:wat::kernel::send tx (:wat::spawn::ProcessLaunch/pid launch)) [:wat::kernel::SendOutcome.Sent {} nil] [:wat::kernel::SendOutcome.Closed {} nil] [:wat::kernel::SendOutcome.Stopped {} nil] [:wat::kernel::SendOutcome.Lost {:cause _c} nil])]
                   nil)))
             (:wat::core::forms
               (:wat::core::defn :user::main [] -> :wat::core::nil (:wat::kernel::println "spawned child"))))
     pid   (:wat::core::match (:wat::kernel::recv rx)
             [:wat::kernel::RecvOutcome.Message {:msg m} m]
             [:wat::kernel::RecvOutcome.Lost {:cause cause}
               (:wat::kernel::assertion-failed! :message (:wat::kernel::LociDiedError/message cause))]
             [:wat::kernel::RecvOutcome.Stopped {}
               (:wat::kernel::assertion-failed! :message "recv': stopped before the post-spawn hook sent the pid — the peer was ALIVE")]
             [:wat::kernel::RecvOutcome.Closed {}
               (:wat::kernel::assertion-failed! :message "recv': rx closed before the post-spawn hook sent the pid")])]
    pid))
