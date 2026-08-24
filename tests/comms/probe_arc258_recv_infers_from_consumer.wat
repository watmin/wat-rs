;; tests/comms/probe_arc258_recv_infers_from_consumer.wat — co-located fixture slurped via startup_beside(file!()).
;; Arc 258.5 — recv' infers its type from the constraining consumer (connect').
;; CHECK-LEVEL probe: (connect' (recv' svc)) with NO -> :T must type-check.
;; GREEN once connect' unifies its arg against (Address' :- [fresh fresh]) so the fresh O binds.

(:wat::core::defn :user::compute [] -> :wat::core::nil
  (:wat::core::let
    [svc  (:wat::test::spawn-peer (:wat::spawn::process)
            (:wat::core::forms
              (:wat::core::defn :user::main [] -> :wat::core::nil
                (:wat::core::let
                  [b    (:wat::kernel::listener (:wat::spawn::process) :wat::core::i64 :wat::core::i64)
                   addr (:wat::spawn::Bound/address b)
                   self (:wat::program::self-peer
                          (:wat::kernel::Address :- [:wat::core::i64 :wat::core::i64]) :wat::core::i64)
                   _    (:wat::core::match (:wat::kernel::send self addr) (:wat::kernel::SendOutcome::Sent nil) (:wat::kernel::SendOutcome::Closed nil) ((:wat::kernel::SendOutcome::Lost _c) nil) (:wat::kernel::SendOutcome::Stopped nil))] ;; arc 278 #73 — fire-and-forget address handoff; outcome ignored uniformly regardless of cause
                  nil))))
     r    (:wat::kernel::recv svc)
     ;; arc 278 the recv'-outcome wall — recv' returns a matchable (RecvOutcome :- [Address']),
     ;; never a raise. The consumer (connect' addr) still pins O through the ::Message
     ;; binding. OWNER role (the test is the final caller): on ::Lost surface the cause
     ;; loudly (eprintln — the dying declaration, divergent-return); ::Closed likewise.
     addr (:wat::core::match r
            ((:wat::kernel::RecvOutcome::Message m) m)
            ((:wat::kernel::RecvOutcome::Lost cause)
              (:wat::kernel::assertion-failed! (:wat::kernel::LociDiedError/message cause) :wat::core::None :wat::core::None))
            (:wat::kernel::RecvOutcome::Stopped
              (:wat::kernel::assertion-failed! "recv': stopped — the substrate was asked to stop; the peer was ALIVE and the channel open" :wat::core::None :wat::core::None))
            (:wat::kernel::RecvOutcome::Closed
              (:wat::kernel::assertion-failed! "recv': svc closed before sending the address" :wat::core::None :wat::core::None)))
     c    (:wat::core::match (:wat::kernel::connect addr) ((:wat::kernel::ConnectOutcome::Connected p) p) ((:wat::kernel::ConnectOutcome::Refused c) (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message c) :wat::core::None :wat::core::None)) ((:wat::kernel::ConnectOutcome::Rejected c) (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message c) :wat::core::None :wat::core::None)) ((:wat::kernel::ConnectOutcome::Failed c) (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message c) :wat::core::None :wat::core::None)))]
    nil))

