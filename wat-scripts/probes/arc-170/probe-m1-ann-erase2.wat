;; probe-m1-ann-erase.wat — can ann-form erase concrete (Address' :- [S R]) -> bare Address',
;; store in (Vector :- [Address']), then send it (bare) to a child that recv's it concrete + connects?
;; The parent's (PoolMsg :- [Address' ...]) and child's (PoolMsg :- [(Address' :- [S R]) ...]) are SEPARATE
;; typecheck universes; only the wire bytes must match.
;; EXPECT (green): "echo:z"

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

;; PARENT-side PoolMsg with BARE Address' payload (erased D).
(:wat::core::defenum :probe::PoolMsg :- [I] :wat::enum::Pure
  :Setup [addr <- :wat::kernel::Address]
  :Work  [s    <- :I])

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::let
    [eh   (:probe::echo/start :locus (:wat::spawn::process) :record (:probe::echo::Record))
     ea   (:probe::echo::Handle/addr eh)
     ;; ERASE concrete (Address' :- [Op Reply]) -> bare Address' via ann-form:
     eab  (:wat::core::ann-form ea :wat::kernel::Address)
     erased (:wat::core::Vector :- [:wat::kernel::Address] eab)
     worker (:wat::test::spawn-peer (:wat::spawn::process)
              (:wat::core::forms
                (:wat::core::defsurface :probe::Echo :nature :wat::kernel::Peer
                  :messages
                  [(:wat::core::defrecord :probe::Echo::EchoRequest  [msg   <- :wat::core::String])
                   (:wat::core::defenum :probe::Echo::EchoResponse :wat::enum::Pure :Ok [reply <- :wat::core::String] :RequestTooLarge [bytes <- :wat::core::i64  cap <- :wat::core::i64]
                                                                                                                      :RequestMalformed [path <- (:wat::core::Vector :- [:wat::core::String])  expected <- :wat::core::String  got <- :wat::core::String])]
                  :features
                  [(echo [self <- :probe::Echo  req <- :probe::Echo::EchoRequest] -> :probe::Echo::EchoResponse :max-request-bytes 524288)])
                ;; CHILD-side PoolMsg with CONCRETE (Address' :- [Op Reply]) payload.
                (:wat::core::defenum :probe::PoolMsg :wat::enum::Pure
                  :Setup [addr <- (:wat::kernel::Address :- [:probe::Echo::Op :probe::Echo::Reply])]
                  :Work  [s    <- :wat::core::String])
                (:wat::core::defn :probe::serve
                  [self <- (:wat::kernel::Peer :- [:wat::core::String :probe::PoolMsg])
                   held <- (:wat::core::Option :- [(:wat::kernel::Peer :- [:probe::Echo::Op :probe::Echo::Reply])])]
                  -> :wat::core::nil
                  (:wat::core::match (:wat::kernel::recv self)
                    [:wat::kernel::RecvOutcome.Message {:msg m}
                      (:wat::core::match m
                        [:probe::PoolMsg.Setup {:addr addr}
                          (:probe::serve self (:wat::core::Option.Some {:value (:wat::core::match (:wat::kernel::connect addr) [:wat::kernel::ConnectOutcome.Connected {:peer p} p] [:wat::kernel::ConnectOutcome.Refused {:cause c} (:wat::kernel::assertion-failed! :message (:wat::kernel::Failure/message c))] [:wat::kernel::ConnectOutcome.Rejected {:cause c} (:wat::kernel::assertion-failed! :message (:wat::kernel::Failure/message c))] [:wat::kernel::ConnectOutcome.Failed {:cause c} (:wat::kernel::assertion-failed! :message (:wat::kernel::Failure/message c))])}))]
                        [:probe::PoolMsg.Work {:s s}
                          (:wat::core::let
                            [c  (:wat::core::Option/expect held "Work before Setup")
                             er (:wat::core::match (:probe::Echo/echo c (:probe::Echo::EchoRequest :msg s))
                                   [:wat::kernel::RecvOutcome.Message {:msg __recv}
                                     (:wat::core::match __recv
                                       [:probe::Echo::EchoResponse.Ok {:reply reply} reply]
                                       [:probe::Echo::EchoResponse.RequestTooLarge {:bytes bytes :cap cap}
                                         (:wat::kernel::assertion-failed! :message "unexpected RequestTooLarge")]
                                       [:probe::Echo::EchoResponse.RequestMalformed {:path mpath :expected mexpected :got mgot}
                                         (:wat::kernel::assertion-failed! :message "unexpected RequestMalformed")])]
                                   [:wat::kernel::RecvOutcome.Lost {:cause __cause}
                                     (:wat::kernel::assertion-failed! :message (:wat::kernel::LociDiedError/message __cause))]
                                   [:wat::kernel::RecvOutcome.Stopped {}
                                     (:wat::kernel::assertion-failed! :message "recv': stopped — the substrate was asked to stop; the peer was ALIVE and the channel open")]
                                   [:wat::kernel::RecvOutcome.Closed {}
                                     (:wat::kernel::assertion-failed! :message "recv': peer closed")])
                             _  (:wat::core::match (:wat::kernel::send self er) [:wat::kernel::SendOutcome.Sent {} nil] [:wat::kernel::SendOutcome.Closed {} nil] [:wat::kernel::SendOutcome.Stopped {} nil] [:wat::kernel::SendOutcome.Lost {:cause _c} nil])]
                            (:probe::serve self held))])]
                    [:wat::kernel::RecvOutcome.Lost {:cause cause}
                      (:wat::kernel::assertion-failed! :message (:wat::kernel::LociDiedError/message cause))]
                    [:wat::kernel::RecvOutcome.Stopped {} nil]
                    [:wat::kernel::RecvOutcome.Closed {} nil]))
                (:wat::core::defn :user::main [] -> :wat::core::nil
                  (:wat::core::let
                    [self (:wat::program::self-peer :wat::core::String :probe::PoolMsg)]
                    (:probe::serve self :wat::core::Option.None)))))
     out  (:wat::core::match (:wat::kernel::peer-pid worker)
            [:wat::core::Option.Some {:value p}
              (:wat::core::let
                [_  (:probe::echo/grant eh (:wat::core::Vector :- [:wat::core::i64] p))
                 ;; parent sends a BARE-typed Setup; child decodes into concrete slot.
                 _  (:wat::core::match (:wat::kernel::send worker (:probe::PoolMsg.Setup {:addr (:wat::core::first erased)})) [:wat::kernel::SendOutcome.Sent {} nil] [:wat::kernel::SendOutcome.Closed {} nil] [:wat::kernel::SendOutcome.Stopped {} nil] [:wat::kernel::SendOutcome.Lost {:cause _c} nil])
                 _  (:wat::core::match (:wat::kernel::send worker (:probe::PoolMsg.Work {:s "z"})) [:wat::kernel::SendOutcome.Sent {} nil] [:wat::kernel::SendOutcome.Closed {} nil] [:wat::kernel::SendOutcome.Stopped {} nil] [:wat::kernel::SendOutcome.Lost {:cause _c} nil])
                 r1 (:wat::core::match (:wat::kernel::recv worker)
                      [:wat::kernel::RecvOutcome.Message {:msg m} m]
                      [:wat::kernel::RecvOutcome.Lost {:cause cause}
                        (:wat::kernel::assertion-failed! :message (:wat::kernel::LociDiedError/message cause))]
                      [:wat::kernel::RecvOutcome.Stopped {}
                        (:wat::kernel::assertion-failed! :message "recv': stopped — the substrate was asked to stop; the peer was ALIVE and the channel open")]
                      [:wat::kernel::RecvOutcome.Closed {}
                        (:wat::kernel::assertion-failed! :message "recv': peer closed")])]
                r1)]
            [:wat::core::Option.None {}
              (:wat::kernel::assertion-failed! :message "peer-pid None")])]
    (:wat::kernel::println out)))
