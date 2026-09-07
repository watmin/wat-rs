;; tests/comms/probe_arc272_6a_capability_handoff.wat — co-located fixture for the capability
;; handoff probe, slurped via startup_beside(file!()). No placeholder main at the top level.
;; The inner :user::main inside the forms block is the CHILD's entrypoint — it is kept.

(:wat::core::defn :user::compute [] -> :wat::core::i64
  (:wat::core::let
    [svc  (:wat::test::spawn-peer (:wat::spawn::process)
            (:wat::core::forms
              (:wat::core::defn :user::main [] -> :wat::core::nil
                (:wat::core::let
                  ;; the child mints its OWN rendezvous: autobind, no name (step 2b).
                  [b    (:wat::kernel::listener (:wat::spawn::process) :wat::core::i64 :wat::core::i64)
                   addr (:wat::spawn::Bound/address b)
                   ;; the self-peer carries the Address' capability child->parent (S = Address').
                   self (:wat::program::self-peer
                          (:wat::kernel::Address :- [:wat::core::i64 :wat::core::i64]) :wat::core::i64)
                   ;; hand the parent the capability — the lock-step handoff (it now has perfect knowledge).
                   _    (:wat::core::match (:wat::kernel::send self addr) [:wat::kernel::SendOutcome::Sent {} nil] [:wat::kernel::SendOutcome::Closed {} nil] [:wat::kernel::SendOutcome::Lost {:cause _c} nil] [:wat::kernel::SendOutcome::Stopped {} nil]) ;; arc 278 #73 — fire-and-forget capability handoff; outcome ignored uniformly regardless of cause
                   ;; accept the parent's dial on our own listener; round-trip n -> n+100.
                   c    (:wat::core::match (:wat::kernel::accept (:wat::spawn::Bound/listener b))
                          [:wat::kernel::AcceptOutcome::Accepted {:peer p} p]
                          [:wat::kernel::AcceptOutcome::Closed {}
                            (:wat::kernel::assertion-failed! :message "accept': listener closed before the parent dialed")]
                          [:wat::kernel::AcceptOutcome::Failed {:cause cause}
                            (:wat::kernel::assertion-failed! :message (:wat::kernel::Failure/message cause))])
                   n    (:wat::core::match (:wat::kernel::recv c)
                          [:wat::kernel::RecvOutcome::Message {:msg m} m]
                          [:wat::kernel::RecvOutcome::Lost {:cause cause}
                            (:wat::kernel::assertion-failed! :message (:wat::kernel::LociDiedError/message cause))]
                          [:wat::kernel::RecvOutcome::Stopped {}
                            (:wat::kernel::assertion-failed! :message "recv': stopped — the substrate was asked to stop; the peer was ALIVE and the channel open")]
                          [:wat::kernel::RecvOutcome::Closed {}
                            (:wat::kernel::assertion-failed! :message "recv': c closed unexpectedly")])
                   _    (:wat::core::match (:wat::kernel::send c (:wat::core::+ n 100)) [:wat::kernel::SendOutcome::Sent {} nil] [:wat::kernel::SendOutcome::Closed {} nil] [:wat::kernel::SendOutcome::Lost {:cause _c} nil] [:wat::kernel::SendOutcome::Stopped {} nil])] ;; arc 278 #73 — fire-and-forget reply; outcome ignored uniformly regardless of cause
                  nil))))
     ;; recv' the child's minted capability over the lineage channel (blocks until the child sends it).
     addr (:wat::core::match (:wat::kernel::recv svc)
            [:wat::kernel::RecvOutcome::Message {:msg m} m]
            [:wat::kernel::RecvOutcome::Lost {:cause cause}
              (:wat::kernel::assertion-failed! :message (:wat::kernel::LociDiedError/message cause))]
            [:wat::kernel::RecvOutcome::Stopped {}
              (:wat::kernel::assertion-failed! :message "recv': stopped — the substrate was asked to stop; the peer was ALIVE and the channel open")]
            [:wat::kernel::RecvOutcome::Closed {}
              (:wat::kernel::assertion-failed! :message "recv': svc closed unexpectedly")])
     ;; dial the capability — the child is guaranteed listening (it sent AFTER listen()).
     c    (:wat::core::match (:wat::kernel::connect addr) [:wat::kernel::ConnectOutcome::Connected {:peer p} p] [:wat::kernel::ConnectOutcome::Refused {:cause c} (:wat::kernel::assertion-failed! :message (:wat::kernel::Failure/message c))] [:wat::kernel::ConnectOutcome::Rejected {:cause c} (:wat::kernel::assertion-failed! :message (:wat::kernel::Failure/message c))] [:wat::kernel::ConnectOutcome::Failed {:cause c} (:wat::kernel::assertion-failed! :message (:wat::kernel::Failure/message c))])
     _    (:wat::core::match (:wat::kernel::send c 5) [:wat::kernel::SendOutcome::Sent {} nil] [:wat::kernel::SendOutcome::Closed {} nil] [:wat::kernel::SendOutcome::Lost {:cause _c} nil] [:wat::kernel::SendOutcome::Stopped {} nil]) ;; arc 278 #73 — fire-and-forget request; outcome ignored uniformly regardless of cause
     got  (:wat::core::match (:wat::kernel::recv c)
            [:wat::kernel::RecvOutcome::Message {:msg m} m]
            [:wat::kernel::RecvOutcome::Lost {:cause cause}
              (:wat::kernel::assertion-failed! :message (:wat::kernel::LociDiedError/message cause))]
            [:wat::kernel::RecvOutcome::Stopped {}
              (:wat::kernel::assertion-failed! :message "recv': stopped — the substrate was asked to stop; the peer was ALIVE and the channel open")]
            [:wat::kernel::RecvOutcome::Closed {}
              (:wat::kernel::assertion-failed! :message "recv': c closed unexpectedly")])]
    got))
