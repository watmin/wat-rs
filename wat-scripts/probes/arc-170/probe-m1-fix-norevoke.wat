;; probe-m1-fix-norevoke.wat — the FIXED counterfactual for the M1-teeth test soundness: the fix
;; probe-m1-cf-norevoke.wat's raise asked for.
;;
;; probe-m1-cf-norevoke.wat's circuit (no `echo'/revoke`), with ONE change: the prober SENDS dial #2's
;; reply UP (`_2 (send self …)`), so success is observable. Without the revoke, dial #2 (by a
;; still-granted pid) is ADMITTED and compute REACHES THE END.
;;
;; Its twin, probe-m1-fix-revoke.wat, restores the revoke; together they show the fixed circuit
;; discriminates — the shape the committed test (tests/services/probe_arc170_m1_teeth_revoked.wat,
;; `Outcome.Bounced` / `Outcome.Served`) now has.
;;
;; EXPECT: "NOREVOKE-REACHED-END: echo:hi"

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
            (:wat::service::Outcome.Reply {:state s
              :reply (:probe::Echo::EchoResponse.Ok {:reply (:wat::string::concat "echo:" (:probe::Echo::EchoRequest/msg req))})}))])

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
                     addr (:wat::core::match (:wat::kernel::recv self) [:wat::kernel::RecvOutcome.Message {:msg __d} __d] [:wat::kernel::RecvOutcome.Lost {:cause __c} (:wat::kernel::assertion-failed! :message (:wat::kernel::LociDiedError/message __c))] [:wat::kernel::RecvOutcome.Stopped {} (:wat::kernel::assertion-failed! :message "recv': stopped — the substrate was asked to stop; the peer was ALIVE and the channel open")] [:wat::kernel::RecvOutcome.Closed {} (:wat::kernel::assertion-failed! :message "recv': peer closed")])
                     c1   (:wat::core::match (:wat::kernel::connect addr) [:wat::kernel::ConnectOutcome.Connected {:peer p} p] [:wat::kernel::ConnectOutcome.Closed {:cause c} (:wat::kernel::assertion-failed! :message (:wat::kernel::Failure/message c))] [:wat::kernel::ConnectOutcome.Undialable {:cause c} (:wat::kernel::assertion-failed! :message (:wat::kernel::Failure/message c))] [:wat::kernel::ConnectOutcome.WrongPeer {:cause c} (:wat::kernel::assertion-failed! :message (:wat::kernel::Failure/message c))] [:wat::kernel::ConnectOutcome.Failed {:cause c} (:wat::kernel::assertion-failed! :message (:wat::kernel::Failure/message c))])
                     er1  (:probe::Echo/echo c1 (:probe::Echo::EchoRequest :msg "hi"))
                     _    (:wat::core::match (:wat::kernel::send self (:wat::core::match er1 [:wat::kernel::RecvOutcome.Message {:msg __recv} (:wat::core::match __recv [:probe::Echo::EchoResponse.Ok {:reply reply} reply]
  [:probe::Echo::EchoResponse.RequestTooLarge {:bytes bytes :cap cap}
    (:wat::kernel::assertion-failed! :message "unexpected RequestTooLarge")]
  [:probe::Echo::EchoResponse.RequestMalformed {:path mpath :expected mexpected :got mgot}
    (:wat::kernel::assertion-failed! :message "unexpected RequestMalformed")])] [:wat::kernel::RecvOutcome.Lost {:cause __cause} (:wat::kernel::assertion-failed! :message (:wat::kernel::LociDiedError/message __cause))] [:wat::kernel::RecvOutcome.Stopped {} (:wat::kernel::assertion-failed! :message "recv': stopped — the substrate was asked to stop; the peer was ALIVE and the channel open")] [:wat::kernel::RecvOutcome.Closed {} (:wat::kernel::assertion-failed! :message "recv': peer closed")])) [:wat::kernel::SendOutcome.Sent {} nil] [:wat::kernel::SendOutcome.HandleClosed {} nil] [:wat::kernel::SendOutcome.Closed {:cause _c} nil] [:wat::kernel::SendOutcome.Failed {:cause _c} nil] [:wat::kernel::SendOutcome.Stopped {} nil])
                     _sig (:wat::core::match (:wat::kernel::recv self) [:wat::kernel::RecvOutcome.Message {:msg __d} __d] [:wat::kernel::RecvOutcome.Lost {:cause __c} (:wat::kernel::assertion-failed! :message (:wat::kernel::LociDiedError/message __c))] [:wat::kernel::RecvOutcome.Stopped {} (:wat::kernel::assertion-failed! :message "recv': stopped — the substrate was asked to stop; the peer was ALIVE and the channel open")] [:wat::kernel::RecvOutcome.Closed {} (:wat::kernel::assertion-failed! :message "recv': peer closed")])
                     c2   (:wat::core::match (:wat::kernel::connect addr) [:wat::kernel::ConnectOutcome.Connected {:peer p} p] [:wat::kernel::ConnectOutcome.Closed {:cause c} (:wat::kernel::assertion-failed! :message (:wat::kernel::Failure/message c))] [:wat::kernel::ConnectOutcome.Undialable {:cause c} (:wat::kernel::assertion-failed! :message (:wat::kernel::Failure/message c))] [:wat::kernel::ConnectOutcome.WrongPeer {:cause c} (:wat::kernel::assertion-failed! :message (:wat::kernel::Failure/message c))] [:wat::kernel::ConnectOutcome.Failed {:cause c} (:wat::kernel::assertion-failed! :message (:wat::kernel::Failure/message c))])
                     er2  (:probe::Echo/echo c2 (:probe::Echo::EchoRequest :msg "hi"))
                     _2   (:wat::kernel::send self (:wat::core::match er2 [:wat::kernel::RecvOutcome.Message {:msg __recv} (:wat::core::match __recv [:probe::Echo::EchoResponse.Ok {:reply reply} reply]
  [:probe::Echo::EchoResponse.RequestTooLarge {:bytes bytes :cap cap}
    (:wat::kernel::assertion-failed! :message "unexpected RequestTooLarge")]
  [:probe::Echo::EchoResponse.RequestMalformed {:path mpath :expected mexpected :got mgot}
    (:wat::kernel::assertion-failed! :message "unexpected RequestMalformed")])] [:wat::kernel::RecvOutcome.Lost {:cause __cause} (:wat::kernel::assertion-failed! :message (:wat::kernel::LociDiedError/message __cause))] [:wat::kernel::RecvOutcome.Stopped {} (:wat::kernel::assertion-failed! :message "recv': stopped — the substrate was asked to stop; the peer was ALIVE and the channel open")] [:wat::kernel::RecvOutcome.Closed {} (:wat::kernel::assertion-failed! :message "recv': peer closed")]))]
                    nil))))
     _   (:wat::core::match (:wat::kernel::peer-pid prober) 
           [:wat::core::Option.Some {:value p}
             (:wat::core::let
               [_  (:probe::echo/grant  eh (:wat::core::Vector :- [:wat::core::i64] p))
                _  (:wat::core::match (:wat::kernel::send prober ea) [:wat::kernel::SendOutcome.Sent {} nil] [:wat::kernel::SendOutcome.HandleClosed {} nil] [:wat::kernel::SendOutcome.Stopped {} nil] [:wat::kernel::SendOutcome.Closed {:cause _c} nil] [:wat::kernel::SendOutcome.Failed {:cause _c} nil])
                r1 (:wat::kernel::recv prober)
                ;; <<< the echo'/revoke line is REMOVED here (the counterfactual) >>>
                _  (:wat::core::match (:wat::kernel::send prober ea) [:wat::kernel::SendOutcome.Sent {} nil] [:wat::kernel::SendOutcome.HandleClosed {} nil] [:wat::kernel::SendOutcome.Stopped {} nil] [:wat::kernel::SendOutcome.Closed {:cause _c} nil] [:wat::kernel::SendOutcome.Failed {:cause _c} nil])
                rr2 (:wat::kernel::recv prober)
                r2 (:wat::core::match rr2
                     [:wat::kernel::RecvOutcome.Message {:msg m} m]
                     [:wat::kernel::RecvOutcome.Lost {:cause cause}
                       (:wat::kernel::assertion-failed! :message (:wat::kernel::LociDiedError/message cause))]
                     [:wat::kernel::RecvOutcome.Stopped {}
                       (:wat::kernel::assertion-failed! :message "recv': stopped — the substrate was asked to stop; the peer was ALIVE and the channel open")]
                     [:wat::kernel::RecvOutcome.Closed {}
                       (:wat::kernel::assertion-failed! :message "recv': prober closed unexpectedly")])]
               (:wat::kernel::println (:wat::string::concat "NOREVOKE-REACHED-END: " r2)))]
           [:wat::core::Option.None {}
             (:wat::kernel::assertion-failed! :message "peer-pid None on process prober")])]
    nil))
