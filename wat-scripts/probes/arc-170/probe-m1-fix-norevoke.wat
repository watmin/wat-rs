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
            (:wat::service::Outcome::Reply {:state s
              :reply (:probe::Echo::EchoResponse::Ok {:reply (:wat::string::concat "echo:" (:probe::Echo::EchoRequest/msg req))})}))])

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
                     c1   (:wat::core::match (:wat::kernel::connect addr) [:wat::kernel::ConnectOutcome::Connected {:peer p} p] [:wat::kernel::ConnectOutcome::Refused {:cause c} (:wat::kernel::assertion-failed! :message (:wat::kernel::Failure/message c))] [:wat::kernel::ConnectOutcome::Rejected {:cause c} (:wat::kernel::assertion-failed! :message (:wat::kernel::Failure/message c))] [:wat::kernel::ConnectOutcome::Failed {:cause c} (:wat::kernel::assertion-failed! :message (:wat::kernel::Failure/message c))])
                     er1  (:probe::Echo/echo c1 (:probe::Echo::EchoRequest :msg "hi"))
                     _    (:wat::core::match (:wat::kernel::send self (:wat::core::match er1 [:probe::Echo::EchoResponse::Ok {:reply reply} reply]
  [:probe::Echo::EchoResponse::RequestTooLarge {:bytes bytes :cap cap}
    (:wat::kernel::assertion-failed! :message "unexpected RequestTooLarge")]
  [:probe::Echo::EchoResponse::RequestMalformed {:path mpath :expected mexpected :got mgot}
    (:wat::kernel::assertion-failed! :message "unexpected RequestMalformed")])) [:wat::kernel::SendOutcome::Sent {} nil] [:wat::kernel::SendOutcome::Closed {} nil] [:wat::kernel::SendOutcome::Lost {:cause _c} nil])
                     _sig (:wat::kernel::recv self)
                     c2   (:wat::core::match (:wat::kernel::connect addr) [:wat::kernel::ConnectOutcome::Connected {:peer p} p] [:wat::kernel::ConnectOutcome::Refused {:cause c} (:wat::kernel::assertion-failed! :message (:wat::kernel::Failure/message c))] [:wat::kernel::ConnectOutcome::Rejected {:cause c} (:wat::kernel::assertion-failed! :message (:wat::kernel::Failure/message c))] [:wat::kernel::ConnectOutcome::Failed {:cause c} (:wat::kernel::assertion-failed! :message (:wat::kernel::Failure/message c))])
                     er2  (:probe::Echo/echo c2 (:probe::Echo::EchoRequest :msg "hi"))
                     _2   (:wat::kernel::send self (:wat::core::match er2 [:probe::Echo::EchoResponse::Ok {:reply reply} reply]
  [:probe::Echo::EchoResponse::RequestTooLarge {:bytes bytes :cap cap}
    (:wat::kernel::assertion-failed! :message "unexpected RequestTooLarge")]
  [:probe::Echo::EchoResponse::RequestMalformed {:path mpath :expected mexpected :got mgot}
    (:wat::kernel::assertion-failed! :message "unexpected RequestMalformed")]))]
                    nil))))
     _   (:wat::core::match (:wat::kernel::peer-pid prober) 
           [:wat::core::Option::Some {:value p}
             (:wat::core::let
               [_  (:probe::echo/grant  eh (:wat::core::Vector :- [:wat::core::i64] p))
                _  (:wat::core::match (:wat::kernel::send prober ea) [:wat::kernel::SendOutcome::Sent {} nil] [:wat::kernel::SendOutcome::Closed {} nil] [:wat::kernel::SendOutcome::Stopped {} nil] [:wat::kernel::SendOutcome::Lost {:cause _c} nil])
                r1 (:wat::kernel::recv prober)
                ;; <<< the echo'/revoke line is REMOVED here (the counterfactual) >>>
                _  (:wat::core::match (:wat::kernel::send prober ea) [:wat::kernel::SendOutcome::Sent {} nil] [:wat::kernel::SendOutcome::Closed {} nil] [:wat::kernel::SendOutcome::Stopped {} nil] [:wat::kernel::SendOutcome::Lost {:cause _c} nil])
                rr2 (:wat::kernel::recv prober)
                r2 (:wat::core::match rr2
                     [:wat::kernel::RecvOutcome::Message {:msg m} m]
                     [:wat::kernel::RecvOutcome::Lost {:cause cause}
                       (:wat::kernel::assertion-failed! :message (:wat::kernel::LociDiedError/message cause))]
                     [:wat::kernel::RecvOutcome::Stopped {}
                       (:wat::kernel::assertion-failed! :message "recv': stopped — the substrate was asked to stop; the peer was ALIVE and the channel open")]
                     [:wat::kernel::RecvOutcome::Closed {}
                       (:wat::kernel::assertion-failed! :message "recv': prober closed unexpectedly")])]
               (:wat::kernel::println (:wat::string::concat "NOREVOKE-REACHED-END: " r2)))]
           [:wat::core::Option::None {}
             (:wat::kernel::assertion-failed! :message "peer-pid None on process prober")])]
    nil))
