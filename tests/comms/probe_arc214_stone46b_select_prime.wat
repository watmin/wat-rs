;; tests/comms/probe_arc214_stone46b_select_prime.wat — co-located fixture slurped via startup_beside(file!()).
;; Arc 214 Stone 4.6b — select' (FM-2-bis disconfirming probe).
;; Probe 1 (LOAD-BEARING, RUNTIME): select' picks the ready peer.
;; Two thread echo peers; send 7 to peer B ONLY; select' [a b] must return ServiceEvent::Message{idx=1, msg=7}.

(:wat::core::defn :user::mk [] -> (:wat::kernel::Thread :- [:wat::core::i64 :wat::core::i64])
  (:wat::test::spawn-peer (:wat::spawn::thread)
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
              (:wat::kernel::assertion-failed! "recv': self closed unexpectedly" :wat::core::None :wat::core::None)) (:wat::kernel::RecvOutcome::TimedOut (:wat::kernel::assertion-failed! "recv: timed out — the peer is alive and silent" :wat::core::None :wat::core::None))))
        (:wat::kernel::SendOutcome::Sent nil)
        (:wat::kernel::SendOutcome::Closed nil)
        ((:wat::kernel::SendOutcome::Lost _c) nil)
        (:wat::kernel::SendOutcome::Stopped nil))))) ;; arc 278 #73 — fire-and-forget echo; outcome ignored uniformly regardless of cause

(:wat::core::defn :user::compute [] -> (:wat::spawn::ServiceEvent :- [:wat::core::i64 :wat::core::i64 :wat::core::nil])
  (:wat::core::let [a (:user::mk)
                    b (:user::mk)
                    _ (:wat::core::match (:wat::kernel::send b 7)
                        (:wat::kernel::SendOutcome::Sent nil)
                        (:wat::kernel::SendOutcome::Closed nil)
                        ((:wat::kernel::SendOutcome::Lost _c) nil)
                        (:wat::kernel::SendOutcome::Stopped nil)) ;; arc 278 #73 — fire-and-forget request; outcome ignored uniformly regardless of cause
                    picked (:wat::kernel::select [a b])]
    picked))

