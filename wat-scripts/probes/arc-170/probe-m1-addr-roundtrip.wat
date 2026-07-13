;; probe-m1-addr-roundtrip.wat — DISCONFIRMING PROBE for the closure_extract Address' fix.
;;
;; The fix rests on ONE assumption: a process Address' can be reconstructed from its pure
;; SocketAddressWire EDN form, and the reconstruction is DIALABLE. If edn/write → edn/read
;; round-trips a live process Address' AND connect' on the read-back addr reaches A, then
;; closure_extract can encode a captured Address' by emitting that same form (no new codec).
;;
;; EXPECT (green):  a "wire:" line showing #wat-edn.cap/address #wat.kernel/SocketAddressWire {...}
;;                  then  "result: echo:roundtrip"

(:wat::core::defsurface :probe::Echo :nature :wat::kernel::Peer'
  :messages
  [(:wat::core::defrecord :probe::Echo::EchoRequest  [msg   <- :wat::core::String])
   (:wat::core::defrecord :probe::Echo::EchoResponse [reply <- :wat::core::String])]
  :features
  [(echo [self <- :probe::Echo  req <- :probe::Echo::EchoRequest] -> :probe::Echo::EchoResponse)])

(:wat::service::defservice :probe::echo'
  :satisfies :probe::Echo  :durable [] :ephemeral []
  :impls [(echo [s req]
            (:wat::service::Outcome::Reply s
              (:probe::Echo::EchoResponse :reply
                (:wat::core::string::concat "echo:" (:probe::Echo::EchoRequest/msg req)))))])

;; a typed helper: the param pins the reconstructed addr's S,R (unify ? = Echo::Op/Reply).
(:wat::core::defn :probe::dial-and-echo
  [a <- :wat::kernel::Address'<probe::Echo::Op,probe::Echo::Reply>] -> :wat::core::String
  (:wat::core::let
    [c  (:wat::kernel::connect' a)
     er (:probe::Echo/echo c (:probe::Echo::EchoRequest :msg "roundtrip"))]
    (:probe::Echo::EchoResponse/reply er)))

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::let
    [eh    (:probe::echo'/start :locus (:wat::spawn::process) :record (:probe::echo'::Record))
     ea    (:probe::echo'::Handle/addr eh)
     s     (:wat::edn::write ea)
     _     (:wat::kernel::println (:wat::core::string::concat "wire: " s))
     ;; reconstruct from the wire form, dial through the typed helper (unifies the addr type)
     out   (:probe::dial-and-echo (:wat::edn::read s))]
    (:wat::kernel::println (:wat::core::string::concat "result: " out))))
