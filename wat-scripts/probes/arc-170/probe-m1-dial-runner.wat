;; probe-m1-dial-runner.wat — drive the BAKED :wat::bracket::process-dial-runner directly.
;; Child uses stdlib (PoolMsg :- [(Address' :- [Op Reply]) String]) + process-dial-runner (concrete D).
;; Parent sends bare-D (PoolMsg :- [Address' (Tuple :- [i64 String])]) (erased address). Same enum name ⇒ wire ok.
;; EXPECT (green): "echo:a | echo:b"

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

;; PARENT-side PoolMsg alias: bare-D Setup so we can send the erased address.
(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::let
    [eh   (:probe::echo/start :locus (:wat::spawn::process) :record (:probe::echo::Record))
     ea   (:probe::echo::Handle/addr eh)
     eab  (:wat::core::ann-form ea :wat::kernel::Address)       ;; erase concrete -> bare
     worker (:wat::test::spawn-peer (:wat::spawn::process)
              (:wat::core::forms
                (:wat::core::defsurface :probe::Echo :nature :wat::kernel::Peer
                  :messages
                  [(:wat::core::defrecord :probe::Echo::EchoRequest  [msg   <- :wat::core::String])
                   (:wat::core::defenum :probe::Echo::EchoResponse :wat::enum::Pure :Ok [reply <- :wat::core::String] :RequestTooLarge [bytes <- :wat::core::i64  cap <- :wat::core::i64]
                                                                                                                      :RequestMalformed [path <- (:wat::core::Vector :- [:wat::core::String])  expected <- :wat::core::String  got <- :wat::core::String])]
                  :features
                  [(echo [self <- :probe::Echo  req <- :probe::Echo::EchoRequest] -> :probe::Echo::EchoResponse :max-request-bytes 524288)])
                ;; the user's 2-param work-fn [(Peer' :- [Op Reply]) String :-> String]
                (:wat::core::defn :user::bracket::work-fn
                  [c <- (:wat::kernel::Peer :- [:probe::Echo::Op :probe::Echo::Reply])  s <- :wat::core::String]
                  -> :wat::core::String
                  (:wat::core::match (:probe::Echo/echo c (:probe::Echo::EchoRequest :msg s)) ((:probe::Echo::EchoResponse::Ok reply) reply)
  ((:probe::Echo::EchoResponse::RequestTooLarge bytes cap)
    (:wat::kernel::assertion-failed! "unexpected RequestTooLarge" :wat::core::None :wat::core::None))
  ((:probe::Echo::EchoResponse::RequestMalformed mpath mexpected mgot)
    (:wat::kernel::assertion-failed! "unexpected RequestMalformed" :wat::core::None :wat::core::None))))
                (:wat::core::defn :user::main [] -> :wat::core::nil
                  (:wat::bracket::process-dial-runner
                    (:wat::program::self-peer
                      (:wat::core::Tuple :- [:wat::core::i64 :wat::core::String])
                      (:wat::bracket::PoolMsg :- [(:wat::kernel::Address :- [:probe::Echo::Op :probe::Echo::Reply]) :wat::core::String]))
                    :user::bracket::work-fn
                    :wat::core::None))))
     out  (:wat::core::match (:wat::kernel::peer-pid worker) 
            ((:wat::core::Some p)
              (:wat::core::let
                [_  (:probe::echo/grant eh (:wat::core::Vector :- [:wat::core::i64] p))
                 _  (:wat::core::match (:wat::kernel::send worker (:wat::bracket::PoolMsg::Setup eab)) (:wat::kernel::SendOutcome::Sent nil) (:wat::kernel::SendOutcome::Closed nil) (:wat::kernel::SendOutcome::Stopped nil) ((:wat::kernel::SendOutcome::Lost _c) nil))
                 _  (:wat::core::match (:wat::kernel::send worker (:wat::bracket::PoolMsg::Work (:wat::core::Tuple 0 "a"))) (:wat::kernel::SendOutcome::Sent nil) (:wat::kernel::SendOutcome::Closed nil) (:wat::kernel::SendOutcome::Stopped nil) ((:wat::kernel::SendOutcome::Lost _c) nil))
                 r1 (:wat::core::ann-form
                      (:wat::core::match (:wat::kernel::recv worker)
                        ((:wat::kernel::RecvOutcome::Message m) m)
                        ((:wat::kernel::RecvOutcome::Lost cause)
                          (:wat::kernel::assertion-failed! (:wat::kernel::LociDiedError/message cause) :wat::core::None :wat::core::None))
                        (:wat::kernel::RecvOutcome::Stopped
                          (:wat::kernel::assertion-failed! "recv': stopped — the substrate was asked to stop; the peer was ALIVE and the channel open" :wat::core::None :wat::core::None))
                        (:wat::kernel::RecvOutcome::Closed
                          (:wat::kernel::assertion-failed! "recv': worker closed unexpectedly" :wat::core::None :wat::core::None)) (:wat::kernel::RecvOutcome::TimedOut (:wat::kernel::assertion-failed! "recv: timed out — the peer is alive and silent" :wat::core::None :wat::core::None)))
                      (:wat::core::Tuple :- [:wat::core::i64 :wat::core::String]))
                 _  (:wat::core::match (:wat::kernel::send worker (:wat::bracket::PoolMsg::Work (:wat::core::Tuple 1 "b"))) (:wat::kernel::SendOutcome::Sent nil) (:wat::kernel::SendOutcome::Closed nil) (:wat::kernel::SendOutcome::Stopped nil) ((:wat::kernel::SendOutcome::Lost _c) nil))
                 r2 (:wat::core::ann-form
                      (:wat::core::match (:wat::kernel::recv worker)
                        ((:wat::kernel::RecvOutcome::Message m) m)
                        ((:wat::kernel::RecvOutcome::Lost cause)
                          (:wat::kernel::assertion-failed! (:wat::kernel::LociDiedError/message cause) :wat::core::None :wat::core::None))
                        (:wat::kernel::RecvOutcome::Stopped
                          (:wat::kernel::assertion-failed! "recv': stopped — the substrate was asked to stop; the peer was ALIVE and the channel open" :wat::core::None :wat::core::None))
                        (:wat::kernel::RecvOutcome::Closed
                          (:wat::kernel::assertion-failed! "recv': worker closed unexpectedly" :wat::core::None :wat::core::None)) (:wat::kernel::RecvOutcome::TimedOut (:wat::kernel::assertion-failed! "recv: timed out — the peer is alive and silent" :wat::core::None :wat::core::None)))
                      (:wat::core::Tuple :- [:wat::core::i64 :wat::core::String]))]
                (:wat::string::concat (:wat::core::second r1)
                  (:wat::string::concat " | " (:wat::core::second r2)))))
            (:wat::core::None
              (:wat::kernel::assertion-failed! "peer-pid None" :wat::core::None :wat::core::None)))]
    (:wat::kernel::println out)))
