;; tests/comms/probe_arc294_crashprop_inband.wat — co-located fixture slurped via startup_beside(file!()).
;; Arc 294 crash-prop (in-band, thread tier): a service that crashes mid-request must surface
;; its REAL crash reason to a connect'd client, not a generic "channel disconnected".
;;
;; The service handler crashes (assertion-failed!) between recv' and send', while its accepted
;; connection peer (resp_tx to the client) is in scope. spawn_thread_peer's death path sends the
;; reserved crash-sentinel frame in-band on the reused resp_tx; the client's recv'/select'
;; recognize it and surface the reason.

;; ── recv' path: the client's recv' raises with the real crash reason ──
(:wat::core::defn :user::compute-recv [] -> :wat::core::i64
  (:wat::core::let
    [pair  (:wat::kernel::listener' (:wat::spawn::thread) :wat::core::i64 :wat::core::i64)
     l     (:wat::spawn::Bound/listener pair)
     addr  (:wat::spawn::Bound/address pair)
     svc   (:wat::kernel::spawn-program' (:wat::spawn::thread)
              (:wat::core::fn [_admin <- :wat::kernel::ThreadSelfPeer'<wat::core::i64,wat::core::i64>] -> :wat::core::nil
                (:wat::core::let
                  [conn  (:wat::kernel::accept' l)
                   n     (:wat::kernel::recv' conn)
                   _boom (:wat::kernel::assertion-failed! "RECV-CRASH-REASON-42"
                            (:wat::core::Some "ACTUAL-1") (:wat::core::Some "EXPECTED-2"))
                   _     (:wat::kernel::send' conn (:wat::core::* n 2))]
                  nil)))
     conn  (:wat::kernel::connect' addr)
     _     (:wat::kernel::send' conn 5)
     reply (:wat::kernel::recv' conn)]
    reply))

;; ── poll' path: the REAL defservice serve loop (service.wat uses poll'). A poll'-driven
;;    service that crashes handling a Message must propagate its reason to the connect'd
;;    client's recv' — the client is accepted via wrap_connect_request, not plain accept'. ──
(:wat::core::defn :user::serve-crash
  [self    <- :wat::kernel::ThreadSelfPeer'<wat::core::i64,wat::core::i64>
   l       <- :wat::kernel::Listener'<wat::core::i64,wat::core::i64>
   clients <- :wat::core::Vector<wat::kernel::Peer'<wat::core::i64,wat::core::i64>>]
  -> :wat::core::nil
  (:wat::core::match (:wat::kernel::poll' self l clients) -> :wat::core::nil
    (:wat::spawn::ServiceEvent::Shutdown nil)
    ((:wat::spawn::ServiceEvent::Connection peer)
      (:user::serve-crash self l (:wat::core::conj clients peer)))
    ((:wat::spawn::ServiceEvent::Message idx msg)
      (:wat::kernel::assertion-failed! "POLL-CRASH-REASON-88"
         (:wat::core::Some "ACTUAL-5") (:wat::core::Some "EXPECTED-6")))
    ((:wat::spawn::ServiceEvent::Closed idx)
      (:user::serve-crash self l (:wat::std::list::remove-at clients idx)))
    ((:wat::spawn::ServiceEvent::Lost idx _cause)
      (:user::serve-crash self l (:wat::std::list::remove-at clients idx)))
    (_ nil)))

(:wat::core::defn :user::compute-poll [] -> :wat::core::i64
  (:wat::core::let
    [pair (:wat::kernel::listener' (:wat::spawn::thread) :wat::core::i64 :wat::core::i64)
     l    (:wat::spawn::Bound/listener pair)
     addr (:wat::spawn::Bound/address pair)
     svc  (:wat::kernel::spawn-program' (:wat::spawn::thread)
            (:wat::core::fn [self <- :wat::kernel::ThreadSelfPeer'<wat::core::i64,wat::core::i64>] -> :wat::core::nil
              (:user::serve-crash self l (:wat::core::Vector :wat::kernel::Peer'<wat::core::i64,wat::core::i64>))))
     conn (:wat::kernel::connect' addr)
     _    (:wat::kernel::send' conn 5)
     reply (:wat::kernel::recv' conn)]
    reply))

;; ── select' path: the client's select' yields :Lost (with the reason), not :Closed ──
;; Markers: :Lost -> 777, :Message -> 222, :Closed -> 111, other -> 333.
(:wat::core::defn :user::compute-select [] -> :wat::core::i64
  (:wat::core::let
    [pair  (:wat::kernel::listener' (:wat::spawn::thread) :wat::core::i64 :wat::core::i64)
     l     (:wat::spawn::Bound/listener pair)
     addr  (:wat::spawn::Bound/address pair)
     svc   (:wat::kernel::spawn-program' (:wat::spawn::thread)
              (:wat::core::fn [_admin <- :wat::kernel::ThreadSelfPeer'<wat::core::i64,wat::core::i64>] -> :wat::core::nil
                (:wat::core::let
                  [conn  (:wat::kernel::accept' l)
                   n     (:wat::kernel::recv' conn)
                   _boom (:wat::kernel::assertion-failed! "SELECT-CRASH-REASON-99"
                            (:wat::core::Some "ACTUAL-3") (:wat::core::Some "EXPECTED-4"))
                   _     (:wat::kernel::send' conn (:wat::core::* n 2))]
                  nil)))
     conn  (:wat::kernel::connect' addr)
     _     (:wat::kernel::send' conn 7)
     ev    (:wat::kernel::select' [conn])]
    (:wat::core::match ev -> :wat::core::i64
      ((:wat::spawn::ServiceEvent::Lost idx _cause) 777)
      ((:wat::spawn::ServiceEvent::Message idx msg) 222)
      ((:wat::spawn::ServiceEvent::Closed idx) 111)
      (_ 333))))
