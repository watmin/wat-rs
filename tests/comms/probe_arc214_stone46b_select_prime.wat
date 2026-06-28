;; tests/comms/probe_arc214_stone46b_select_prime.wat — co-located fixture slurped via startup_beside(file!()).
;; Arc 214 Stone 4.6b — select' (FM-2-bis disconfirming probe).
;; Probe 1 (LOAD-BEARING, RUNTIME): select' picks the ready peer.
;; Two thread echo peers; send 7 to peer B ONLY; select' [a b] must return ServiceEvent::Message{idx=1, msg=7}.

(:wat::core::defn :user::mk [] -> :wat::kernel::Thread'<wat::core::i64,wat::core::i64>
  (:wat::kernel::spawn-program' (:wat::spawn::thread)
    (:wat::core::fn [self <- :wat::kernel::Peer'<wat::core::i64,wat::core::i64>] -> :wat::core::nil
      (:wat::kernel::send' self (:wat::kernel::recv' self)))))

(:wat::core::defn :user::compute [] -> :wat::spawn::ServiceEvent<wat::core::i64,wat::core::i64,wat::core::nil>
  (:wat::core::let [a (:user::mk)
                    b (:user::mk)
                    _ (:wat::kernel::send' b 7)
                    picked (:wat::kernel::select' [a b])]
    picked))

