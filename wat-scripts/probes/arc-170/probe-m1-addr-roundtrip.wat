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
            (:wat::service::Outcome::Continue s
              (:wat::core::Some (:probe::Echo::Reply::Echo (:probe::Echo::EchoResponse::Ok (:wat::string::concat "echo:" (:probe::Echo::EchoRequest/msg req))))) (:wat::core::Vector :- [(:wat::service::Directed :- [:probe::Echo::Reply])]) (:wat::core::Vector :- [(:wat::service::Alarm :- [:probe::echo::Op])])))])

;; a typed helper: the param pins the reconstructed addr's S,R (unify ? = Echo::Op/Reply).
(:wat::core::defn :probe::dial-and-echo
  [a <- (:wat::kernel::Address :- [:probe::Echo::Op :probe::Echo::Reply])] -> :wat::core::String
  (:wat::core::let
    [c  (:wat::core::match (:wat::kernel::connect a) ((:wat::kernel::ConnectOutcome::Connected p) p) ((:wat::kernel::ConnectOutcome::Refused c) (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message c) :wat::core::None :wat::core::None)) ((:wat::kernel::ConnectOutcome::Rejected c) (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message c) :wat::core::None :wat::core::None)) ((:wat::kernel::ConnectOutcome::Failed c) (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message c) :wat::core::None :wat::core::None)))
     er (:probe::Echo/echo c (:probe::Echo::EchoRequest :msg "roundtrip"))]
    (:wat::core::match er ((:wat::kernel::RecvOutcome::Message __recv) (:wat::core::match __recv
  ((:probe::Echo::EchoResponse::Ok reply) reply)
  ((:probe::Echo::EchoResponse::RequestTooLarge bytes cap)
    (:wat::kernel::assertion-failed! "unexpected RequestTooLarge" :wat::core::None :wat::core::None))
  ((:probe::Echo::EchoResponse::RequestMalformed mpath mexpected mgot)
    (:wat::kernel::assertion-failed! "unexpected RequestMalformed" :wat::core::None :wat::core::None)))) ((:wat::kernel::RecvOutcome::Lost __cause) (:wat::kernel::assertion-failed! (:wat::kernel::LociDiedError/message __cause) :wat::core::None :wat::core::None)) (:wat::kernel::RecvOutcome::Stopped (:wat::kernel::assertion-failed! "recv': stopped — the substrate was asked to stop; the peer was ALIVE and the channel open" :wat::core::None :wat::core::None)) (:wat::kernel::RecvOutcome::Closed (:wat::kernel::assertion-failed! "recv': peer closed" :wat::core::None :wat::core::None)))))

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::let
    [eh    (:probe::echo/start :locus (:wat::spawn::process) :record (:probe::echo::Record))
     ea    (:probe::echo::Handle/addr eh)
     s     (:wat::edn::write ea)
     _     (:wat::kernel::println (:wat::string::concat "wire: " s))
     ;; reconstruct from the wire form, dial through the typed helper (unifies the addr type)
     out   (:probe::dial-and-echo (:wat::edn::read s))]
    (:wat::kernel::println (:wat::string::concat "result: " out))))
