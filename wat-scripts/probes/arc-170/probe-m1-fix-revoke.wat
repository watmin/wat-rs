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

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::let
    [eh  (:probe::echo/start :locus (:wat::spawn::process) :record (:probe::echo::Record))
     ea  (:probe::echo::Handle/addr eh)
     prober (:wat::test::spawn-peer (:wat::spawn::process)
              (:wat::core::forms
                (:wat::core::defsurface :probe::Echo :nature :wat::kernel::Peer
                  :messages
                  [(:wat::core::defrecord :probe::Echo::EchoRequest  [msg   <- :wat::core::String])
                   (:wat::core::defenum :probe::Echo::EchoResponse :wat::enum::Pure :Ok [reply <- :wat::core::String] :RequestTooLarge [bytes <- :wat::core::i64  cap <- :wat::core::i64]
                                                                                                                      :RequestMalformed [path <- (:wat::core::Vector :- [:wat::core::String])  expected <- :wat::core::String  got <- :wat::core::String])]
                  :features
                  [(echo [self <- :probe::Echo  req <- :probe::Echo::EchoRequest] -> :probe::Echo::EchoResponse :max-request-bytes 524288)])
                (:wat::core::defn :user::main [] -> :wat::core::nil
                  (:wat::core::let
                    [self (:wat::program::self-peer :wat::core::String
                             (:wat::kernel::Address :- [:probe::Echo::Op :probe::Echo::Reply]))
                     addr (:wat::kernel::recv self)
                     c1   (:wat::core::match (:wat::kernel::connect addr) ((:wat::kernel::ConnectOutcome::Connected p) p) ((:wat::kernel::ConnectOutcome::Refused c) (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message c) :wat::core::None :wat::core::None)) ((:wat::kernel::ConnectOutcome::Rejected c) (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message c) :wat::core::None :wat::core::None)) ((:wat::kernel::ConnectOutcome::Failed c) (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message c) :wat::core::None :wat::core::None)))
                     er1  (:probe::Echo/echo c1 (:probe::Echo::EchoRequest :msg "hi"))
                     _    (:wat::core::match (:wat::kernel::send self (:wat::core::match er1 ((:probe::Echo::EchoResponse::Ok reply) reply)
  ((:probe::Echo::EchoResponse::RequestTooLarge bytes cap)
    (:wat::kernel::assertion-failed! "unexpected RequestTooLarge" :wat::core::None :wat::core::None))
  ((:probe::Echo::EchoResponse::RequestMalformed mpath mexpected mgot)
    (:wat::kernel::assertion-failed! "unexpected RequestMalformed" :wat::core::None :wat::core::None)))) (:wat::kernel::SendOutcome::Sent nil) (:wat::kernel::SendOutcome::Closed nil) ((:wat::kernel::SendOutcome::Lost _c) nil))
                     _sig (:wat::kernel::recv self)
                     c2   (:wat::core::match (:wat::kernel::connect addr) ((:wat::kernel::ConnectOutcome::Connected p) p) ((:wat::kernel::ConnectOutcome::Refused c) (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message c) :wat::core::None :wat::core::None)) ((:wat::kernel::ConnectOutcome::Rejected c) (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message c) :wat::core::None :wat::core::None)) ((:wat::kernel::ConnectOutcome::Failed c) (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message c) :wat::core::None :wat::core::None)))
                     er2  (:probe::Echo/echo c2 (:probe::Echo::EchoRequest :msg "hi"))
                     _2   (:wat::kernel::send self (:wat::core::match er2 ((:probe::Echo::EchoResponse::Ok reply) reply)
  ((:probe::Echo::EchoResponse::RequestTooLarge bytes cap)
    (:wat::kernel::assertion-failed! "unexpected RequestTooLarge" :wat::core::None :wat::core::None))
  ((:probe::Echo::EchoResponse::RequestMalformed mpath mexpected mgot)
    (:wat::kernel::assertion-failed! "unexpected RequestMalformed" :wat::core::None :wat::core::None))))]
                    nil))))
     _   (:wat::core::match (:wat::kernel::peer-pid prober) 
           ((:wat::core::Some p)
             (:wat::core::let
               [_  (:probe::echo/grant  eh (:wat::core::Vector :- [:wat::core::i64] p))
                _  (:wat::core::match (:wat::kernel::send prober ea) (:wat::kernel::SendOutcome::Sent nil) (:wat::kernel::SendOutcome::Closed nil) (:wat::kernel::SendOutcome::Stopped nil) ((:wat::kernel::SendOutcome::Lost _c) nil))
                r1 (:wat::kernel::recv prober)
                _r (:probe::echo/revoke eh (:wat::core::Vector :- [:wat::core::i64] p))
                ;; <<< the echo'/revoke line is REMOVED here (the counterfactual) >>>
                _  (:wat::core::match (:wat::kernel::send prober ea) (:wat::kernel::SendOutcome::Sent nil) (:wat::kernel::SendOutcome::Closed nil) (:wat::kernel::SendOutcome::Stopped nil) ((:wat::kernel::SendOutcome::Lost _c) nil))
                rr2 (:wat::kernel::recv prober)
                r2 (:wat::core::match rr2
                     ((:wat::kernel::RecvOutcome::Message m) m)
                     ((:wat::kernel::RecvOutcome::Lost cause)
                       (:wat::kernel::assertion-failed! (:wat::kernel::LociDiedError/message cause) :wat::core::None :wat::core::None))
                     (:wat::kernel::RecvOutcome::Stopped
                       (:wat::kernel::assertion-failed! "recv': stopped — the substrate was asked to stop; the peer was ALIVE and the channel open" :wat::core::None :wat::core::None))
                     (:wat::kernel::RecvOutcome::Closed
                       (:wat::kernel::assertion-failed! "recv': prober closed unexpectedly" :wat::core::None :wat::core::None)))]
               (:wat::kernel::println (:wat::string::concat "NOREVOKE-REACHED-END: " r2))))
           (:wat::core::None
             (:wat::kernel::assertion-failed! "peer-pid None on process prober"
               :wat::core::None :wat::core::None)))]
    nil))
