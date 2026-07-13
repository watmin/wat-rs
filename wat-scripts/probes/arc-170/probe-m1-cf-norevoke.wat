;; probe-m1-cf-norevoke.wat — COUNTERFACTUAL for the M1-teeth test soundness.
;;
;; The EXACT committed revoked circuit (probe_arc170_m1_teeth_revoked.wat), with ONE change:
;; the `echo'/revoke` line is REMOVED. If the revoke is load-bearing, dial #2 (by a still-granted
;; pid) is ADMITTED, and compute REACHES THE END → prints "NOREVOKE-REACHED-END".
;; If instead this ALSO raises (the prober's clean exit closing the channel makes recv' EOF), then
;; the committed test is VACUOUS — its Err doesn't discriminate the bounce from the exit.
;;
;; EXPECT (if the test is sound): "NOREVOKE-REACHED-END: <r2>"
;; If it raises with NO print → the committed test is vacuous → the fixture needs the prober to
;; send dial #2's reply UP so success is observable.

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

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::let
    [eh  (:probe::echo'/start :locus (:wat::spawn::process) :record (:probe::echo'::Record))
     ea  (:probe::echo'::Handle/addr eh)
     prober (:wat::kernel::spawn-program' (:wat::spawn::process)
              (:wat::core::forms
                (:wat::core::defsurface :probe::Echo :nature :wat::kernel::Peer'
                  :messages
                  [(:wat::core::defrecord :probe::Echo::EchoRequest  [msg   <- :wat::core::String])
                   (:wat::core::defrecord :probe::Echo::EchoResponse [reply <- :wat::core::String])]
                  :features
                  [(echo [self <- :probe::Echo  req <- :probe::Echo::EchoRequest] -> :probe::Echo::EchoResponse)])
                (:wat::core::defn :user::main [] -> :wat::core::nil
                  (:wat::core::let
                    [self (:wat::program::self-peer :wat::core::String
                             :wat::kernel::Address'<probe::Echo::Op,probe::Echo::Reply>)
                     addr (:wat::kernel::recv' self)
                     c1   (:wat::kernel::connect' addr)
                     er1  (:probe::Echo/echo c1 (:probe::Echo::EchoRequest :msg "hi"))
                     _    (:wat::kernel::send' self (:probe::Echo::EchoResponse/reply er1))
                     _sig (:wat::kernel::recv' self)
                     c2   (:wat::kernel::connect' addr)
                     er2  (:probe::Echo/echo c2 (:probe::Echo::EchoRequest :msg "hi"))]
                    nil))))
     _   (:wat::core::match (:wat::kernel::peer-pid prober) -> :wat::core::nil
           ((:wat::core::Some p)
             (:wat::core::let
               [_  (:probe::echo'/grant  eh (:wat::core::Vector :wat::core::i64 p))
                _  (:wat::kernel::send' prober ea)
                r1 (:wat::kernel::recv' prober)
                ;; <<< the echo'/revoke line is REMOVED here (the counterfactual) >>>
                _  (:wat::kernel::send' prober ea)
                r2 (:wat::kernel::recv' prober)]
               (:wat::kernel::println (:wat::core::string::concat "NOREVOKE-REACHED-END: " r2))))
           (:wat::core::None
             (:wat::kernel::assertion-failed! "peer-pid None on process prober"
               :wat::core::None :wat::core::None)))]
    nil))
