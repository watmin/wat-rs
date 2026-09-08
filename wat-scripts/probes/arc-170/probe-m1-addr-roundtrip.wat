;; probe-m1-addr-roundtrip.wat — DISCONFIRMING PROBE for the closure_extract Address' fix.
;;
;; The fix rests on ONE assumption: a process Address' can be reconstructed from its pure
;; SocketAddressWire EDN form, and the reconstruction is DIALABLE. If edn/write → edn/read
;; round-trips a live process Address' AND connect' on the read-back addr reaches A, then
;; closure_extract can encode a captured Address' by emitting that same form (no new codec).
;;
;; EXPECT (green):  a "wire:" line showing #wat.kernel/Address #wat.kernel/SocketAddressWire {...}
;;                  then  "result: echo:roundtrip"

(:wat::core::defsurface :probe::Echo :nature :wat::kernel::Peer
  :messages
  [(:wat::core::defrecord :probe::Echo::EchoRequest  [msg   <- :wat::core::String])
   (:wat::core::defenum :probe::Echo::EchoResponse :wat::enum::Pure :Ok [reply <- :wat::core::String] :RequestTooLarge [bytes <- :wat::core::i64  cap <- :wat::core::i64]
                                                                                                      :RequestMalformed [path <- (:wat::core::Vector :- [:wat::core::String])  expected <- :wat::core::String  got <- :wat::core::String])]
  :features
  [(echo [self <- :probe::Echo  req <- :probe::Echo::EchoRequest] -> :probe::Echo::EchoResponse :max-request-bytes 524288)])

(:wat::service::defservice :probe::echo
  :satisfies :probe::Echo  :durable [] :ephemeral []
  :impls [(echo [s ctx req]
            (:wat::service::Outcome::Reply {:state s
              :reply (:probe::Echo::EchoResponse::Ok (:wat::string::concat "echo:" (:probe::Echo::EchoRequest/msg req)))}))])

;; a typed helper: the param pins the reconstructed addr's S,R (unify ? = Echo::Op/Reply).
(:wat::core::defn :probe::dial-and-echo
  [a <- (:wat::kernel::Address :- [:probe::Echo::Op :probe::Echo::Reply])] -> :wat::core::String
  (:wat::core::let
    [c  (:wat::core::match (:wat::kernel::connect a) [:wat::kernel::ConnectOutcome::Connected {:peer p} p] [:wat::kernel::ConnectOutcome::Refused {:cause c} (:wat::kernel::assertion-failed! :message (:wat::kernel::Failure/message c))] [:wat::kernel::ConnectOutcome::Rejected {:cause c} (:wat::kernel::assertion-failed! :message (:wat::kernel::Failure/message c))] [:wat::kernel::ConnectOutcome::Failed {:cause c} (:wat::kernel::assertion-failed! :message (:wat::kernel::Failure/message c))])
     er (:probe::Echo/echo c (:probe::Echo::EchoRequest :msg "roundtrip"))]
    (:wat::core::match er [:wat::kernel::RecvOutcome::Message {:msg __recv} (:wat::core::match __recv
  [:probe::Echo::EchoResponse::Ok {:reply reply} reply]
  [:probe::Echo::EchoResponse::RequestTooLarge {:bytes bytes :cap cap}
    (:wat::kernel::assertion-failed! :message "unexpected RequestTooLarge")]
  [:probe::Echo::EchoResponse::RequestMalformed {:path mpath :expected mexpected :got mgot}
    (:wat::kernel::assertion-failed! :message "unexpected RequestMalformed")])] [:wat::kernel::RecvOutcome::Lost {:cause __cause} (:wat::kernel::assertion-failed! :message (:wat::kernel::LociDiedError/message __cause))] [:wat::kernel::RecvOutcome::Stopped {} (:wat::kernel::assertion-failed! :message "recv': stopped — the substrate was asked to stop; the peer was ALIVE and the channel open")] [:wat::kernel::RecvOutcome::Closed {} (:wat::kernel::assertion-failed! :message "recv': peer closed")])))

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::let
    [eh    (:probe::echo/start :locus (:wat::spawn::process) :record (:probe::echo::Record))
     ea    (:probe::echo::Handle/addr eh)
     s     (:wat::edn::write ea)
     _     (:wat::kernel::println (:wat::string::concat "wire: " s))
     ;; reconstruct from the wire form, dial through the typed helper (unifies the addr type)
     out   (:probe::dial-and-echo (:wat::edn::read s))]
    (:wat::kernel::println (:wat::string::concat "result: " out))))
