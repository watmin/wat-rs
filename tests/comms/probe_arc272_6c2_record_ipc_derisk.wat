;; tests/comms/probe_arc272_6c2_record_ipc_derisk.wat — co-located fixture for the record IPC
;; de-risk probe, slurped via startup_beside(file!()). No placeholder main — startup_beside loads
;; defns only. The inner :user::main inside the forms block is the CHILD's entrypoint, not a
;; placeholder; it is kept.

(:wat::core::defrecord :user::Pt [x <- :wat::core::i64  y <- :wat::core::i64])

(:wat::core::defn :user::compute [] -> :wat::core::i64
  (:wat::core::let
    [svc (:wat::test::spawn-peer (:wat::spawn::process)
            (:wat::core::forms
              ;; The forked child runs a FRESH startup (stdlib prelude + these forms only) — it does
              ;; NOT inherit the parent's top-level defs. So the record must be defined HERE too (D1's
              ;; SocketAddressWire avoids this by living in spawn.wat/stdlib, loaded in every universe).
              (:wat::core::defrecord :user::Pt [x <- :wat::core::i64  y <- :wat::core::i64])
              (:wat::core::defn :user::main [] -> :wat::core::nil
                (:wat::core::let
                  ;; the child mints a plain base record and hands it to the parent over the self-peer.
                  [self (:wat::program::self-peer :user::Pt :wat::core::i64)
                   _    (:wat::core::match (:wat::kernel::send self (:user::Pt :x 7 :y 35)) (:wat::kernel::SendOutcome::Sent nil) (:wat::kernel::SendOutcome::Closed nil) ((:wat::kernel::SendOutcome::Lost _c) nil) (:wat::kernel::SendOutcome::Stopped nil))] ;; arc 278 #73 — fire-and-forget record handoff; outcome ignored uniformly regardless of cause
                  nil))))
     ;; the parent recv's the record off the lineage channel; reconstruct via the EDN wire.
     pt  (:wat::core::match (:wat::kernel::recv svc)
           ((:wat::kernel::RecvOutcome::Message m) m)
           ((:wat::kernel::RecvOutcome::Lost cause)
             (:wat::kernel::assertion-failed! (:wat::kernel::LociDiedError/message cause) :wat::core::None :wat::core::None))
           (:wat::kernel::RecvOutcome::Stopped
             (:wat::kernel::assertion-failed! "recv': stopped — the substrate was asked to stop; the peer was ALIVE and the channel open" :wat::core::None :wat::core::None))
           (:wat::kernel::RecvOutcome::Closed
             (:wat::kernel::assertion-failed! "recv': svc closed unexpectedly" :wat::core::None :wat::core::None)) (:wat::kernel::RecvOutcome::TimedOut (:wat::kernel::assertion-failed! "recv: timed out — the peer is alive and silent" :wat::core::None :wat::core::None)))]
    (:wat::core::+ (:user::Pt/x pt) (:user::Pt/y pt))))
