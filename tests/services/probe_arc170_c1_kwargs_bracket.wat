;; C1 DISCONFIRMING PROBE — a KWARGS work-fn invoked via the companion :key val call, on the
;; PROVEN M1 dial-hold shape (probe-m1-worker-setup). The ONLY new thing vs M1: the Work arm calls
;;   (:probe::work s :echo held)            ← the companion :key val form
;; instead of the direct (Echo/echo held req). Proves the crux for Strike C: a kwargs work-fn runs
;; when invoked with a runtime-dialed peer bound to a :key. The AST-walk's job becomes: synthesize
;; exactly this call (item + :key <held-peer> per uses field). EXPECT (green): echo:a echo:b echo:c

(:wat::core::defsurface :probe::Echo :nature :wat::kernel::Peer
  :messages
  [(:wat::core::defrecord :probe::Echo::EchoRequest  [msg   <- :wat::core::String])
   (:wat::core::defenum :probe::Echo::EchoResponse :wat::enum::Pure
     :Ok              [reply <- :wat::core::String]
     :RequestTooLarge [bytes <- :wat::core::i64  cap <- :wat::core::i64]
     :RequestMalformed [path <- (:wat::core::Vector :- [:wat::core::String])  expected <- :wat::core::String  got <- :wat::core::String])]
  :features
  [(echo [self <- :probe::Echo  req <- :probe::Echo::EchoRequest] -> :probe::Echo::EchoResponse :max-request-bytes 524288)])

(:wat::service::defservice :probe::echo
  :satisfies :probe::Echo  :durable [] :ephemeral []
  :impls [(echo [s ctx req]
            (:wat::service::Outcome::Reply {:state s
              :reply (:probe::Echo::EchoResponse::Ok
                {:reply (:wat::string::concat "echo:" (:probe::Echo::EchoRequest/msg req))})}))])

(:wat::core::defenum :probe::Msg :wat::enum::Pure
  :Setup [addr <- (:wat::kernel::Address :- [:probe::Echo::Op :probe::Echo::Reply])]
  :Work  [s    <- :wat::core::String])

(:wat::core::defn :probe::run [] -> :wat::core::String
  (:wat::core::let
    [eh   (:probe::echo/start :locus (:wat::spawn::process) :record (:probe::echo::Record))
     ea   (:probe::echo::Handle/addr eh)
     worker (:wat::test::spawn-peer (:wat::spawn::process)
              (:wat::core::forms
                (:wat::core::defsurface :probe::Echo :nature :wat::kernel::Peer
                  :messages
                  [(:wat::core::defrecord :probe::Echo::EchoRequest  [msg   <- :wat::core::String])
                   (:wat::core::defenum :probe::Echo::EchoResponse :wat::enum::Pure
                     :Ok              [reply <- :wat::core::String]
                     :RequestTooLarge [bytes <- :wat::core::i64  cap <- :wat::core::i64]
                     :RequestMalformed [path <- (:wat::core::Vector :- [:wat::core::String])  expected <- :wat::core::String  got <- :wat::core::String])]
                  :features
                  [(echo [self <- :probe::Echo  req <- :probe::Echo::EchoRequest] -> :probe::Echo::EchoResponse :max-request-bytes 524288)])
                (:wat::core::defenum :probe::Msg :wat::enum::Pure
                  :Setup [addr <- (:wat::kernel::Address :- [:probe::Echo::Op :probe::Echo::Reply])]
                  :Work  [s    <- :wat::core::String])
                ;; ── the KWARGS work-fn: item positional, `echo` a :key Peer' kwarg ──
                (:wat::core::defn :probe::work
                  [item <- :wat::core::String
                   & [echo <- (:wat::kernel::Peer :- [:probe::Echo::Op :probe::Echo::Reply])]]
                  -> :wat::core::String
                  (:wat::core::match (:probe::Echo/echo echo (:probe::Echo::EchoRequest :msg item)) [:wat::kernel::RecvOutcome::Message {:msg __recv} (:wat::core::match __recv 
                    [:probe::Echo::EchoResponse::Ok {:reply reply} reply]
                    [:probe::Echo::EchoResponse::RequestTooLarge {:bytes bytes :cap cap}
                      (:wat::kernel::assertion-failed! :message "work: unexpected RequestTooLarge")]
                    [:probe::Echo::EchoResponse::RequestMalformed {:path mpath :expected mexpected :got mgot}
                      (:wat::kernel::assertion-failed! :message "unexpected RequestMalformed")])] [:wat::kernel::RecvOutcome::Lost {:cause __cause} (:wat::kernel::assertion-failed! :message (:wat::kernel::LociDiedError/message __cause))] [:wat::kernel::RecvOutcome::Stopped {} (:wat::kernel::assertion-failed! :message "recv': stopped — the substrate was asked to stop; the peer was ALIVE and the channel open")] [:wat::kernel::RecvOutcome::Closed {} (:wat::kernel::assertion-failed! :message "recv': peer closed")]))
                ;; ── serve loop: Work arm invokes via the COMPANION :key val call ──
                (:wat::core::defn :probe::serve
                  [self <- (:wat::kernel::Peer :- [:wat::core::String :probe::Msg])
                   held <- (:wat::core::Option :- [(:wat::kernel::Peer :- [:probe::Echo::Op :probe::Echo::Reply])])]
                  -> :wat::core::nil
                  (:wat::core::match (:wat::kernel::recv self)
                    [:wat::kernel::RecvOutcome::Message {:msg m}
                      (:wat::core::match m
                        [:probe::Msg::Setup {:addr addr}
                          (:probe::serve self (:wat::core::Option::Some {:value (:wat::core::match (:wat::kernel::connect addr) [:wat::kernel::ConnectOutcome::Connected {:peer p} p] [:wat::kernel::ConnectOutcome::Refused {:cause c} (:wat::kernel::assertion-failed! :message (:wat::kernel::Failure/message c))] [:wat::kernel::ConnectOutcome::Rejected {:cause c} (:wat::kernel::assertion-failed! :message (:wat::kernel::Failure/message c))] [:wat::kernel::ConnectOutcome::Failed {:cause c} (:wat::kernel::assertion-failed! :message (:wat::kernel::Failure/message c))])}))]
                        [:probe::Msg::Work {:s s}
                          (:wat::core::let
                            [c (:wat::core::Option/expect held "Work before Setup")
                             r (:probe::work s :echo c)                   ;; ← companion :key val, held peer
                             _ (:wat::core::match (:wat::kernel::send self r) [:wat::kernel::SendOutcome::Sent {} nil] [:wat::kernel::SendOutcome::Closed {} nil] [:wat::kernel::SendOutcome::Stopped {} nil] [:wat::kernel::SendOutcome::Lost {:cause _c} nil])]
                            (:probe::serve self held))])]
                    [:wat::kernel::RecvOutcome::Lost {:cause cause}
                      (:wat::kernel::assertion-failed! :message (:wat::kernel::LociDiedError/message cause))]
                    ;; arc 278 #73 — a stop is a deliberate world-shutdown, not an error the way an
                    ;; unexpected close is here; return quietly (end the serve loop) — JUDGEMENT CALL.
                    [:wat::kernel::RecvOutcome::Stopped {} nil]
                    [:wat::kernel::RecvOutcome::Closed {}
                      (:wat::kernel::assertion-failed! :message "recv': self closed — serve loop terminating")]))
                (:wat::core::defn :user::main [] -> :wat::core::nil
                  (:wat::core::let
                    [self (:wat::program::self-peer :wat::core::String :probe::Msg)]
                    (:probe::serve self :wat::core::Option::None)))))
     out  (:wat::core::match (:wat::kernel::peer-pid worker) 
            [:wat::core::Option::Some {:value p}
              (:wat::core::let
                [_  (:probe::echo/grant eh [p])
                 _  (:wat::core::match (:wat::kernel::send worker (:probe::Msg::Setup {:addr ea})) [:wat::kernel::SendOutcome::Sent {} nil] [:wat::kernel::SendOutcome::Closed {} nil] [:wat::kernel::SendOutcome::Stopped {} nil] [:wat::kernel::SendOutcome::Lost {:cause _c} nil])  ;; arc 278 #73 — the recv' below already faces the stop
                 _  (:wat::core::match (:wat::kernel::send worker (:probe::Msg::Work {:s "a"})) [:wat::kernel::SendOutcome::Sent {} nil] [:wat::kernel::SendOutcome::Closed {} nil] [:wat::kernel::SendOutcome::Stopped {} nil] [:wat::kernel::SendOutcome::Lost {:cause _c} nil])  ;; arc 278 #73 — the recv' below already faces the stop
                 r1 (:wat::core::match (:wat::kernel::recv worker)
                      [:wat::kernel::RecvOutcome::Message {:msg m} m]
                      [:wat::kernel::RecvOutcome::Lost {:cause cause}
                        (:wat::kernel::assertion-failed! :message (:wat::kernel::LociDiedError/message cause))]
                      [:wat::kernel::RecvOutcome::Stopped {}
                        (:wat::kernel::assertion-failed! :message "recv': stopped before reply a — the peer was ALIVE")]
                      [:wat::kernel::RecvOutcome::Closed {}
                        (:wat::kernel::assertion-failed! :message "recv': worker closed before reply a")])
                 _  (:wat::core::match (:wat::kernel::send worker (:probe::Msg::Work {:s "b"})) [:wat::kernel::SendOutcome::Sent {} nil] [:wat::kernel::SendOutcome::Closed {} nil] [:wat::kernel::SendOutcome::Stopped {} nil] [:wat::kernel::SendOutcome::Lost {:cause _c} nil])  ;; arc 278 #73 — the recv' below already faces the stop
                 r2 (:wat::core::match (:wat::kernel::recv worker)
                      [:wat::kernel::RecvOutcome::Message {:msg m} m]
                      [:wat::kernel::RecvOutcome::Lost {:cause cause}
                        (:wat::kernel::assertion-failed! :message (:wat::kernel::LociDiedError/message cause))]
                      [:wat::kernel::RecvOutcome::Stopped {}
                        (:wat::kernel::assertion-failed! :message "recv': stopped before reply b — the peer was ALIVE")]
                      [:wat::kernel::RecvOutcome::Closed {}
                        (:wat::kernel::assertion-failed! :message "recv': worker closed before reply b")])
                 _  (:wat::core::match (:wat::kernel::send worker (:probe::Msg::Work {:s "c"})) [:wat::kernel::SendOutcome::Sent {} nil] [:wat::kernel::SendOutcome::Closed {} nil] [:wat::kernel::SendOutcome::Stopped {} nil] [:wat::kernel::SendOutcome::Lost {:cause _c} nil])  ;; arc 278 #73 — the recv' below already faces the stop
                 r3 (:wat::core::match (:wat::kernel::recv worker)
                      [:wat::kernel::RecvOutcome::Message {:msg m} m]
                      [:wat::kernel::RecvOutcome::Lost {:cause cause}
                        (:wat::kernel::assertion-failed! :message (:wat::kernel::LociDiedError/message cause))]
                      [:wat::kernel::RecvOutcome::Stopped {}
                        (:wat::kernel::assertion-failed! :message "recv': stopped before reply c — the peer was ALIVE")]
                      [:wat::kernel::RecvOutcome::Closed {}
                        (:wat::kernel::assertion-failed! :message "recv': worker closed before reply c")])]
                (:wat::string::concat r1
                  (:wat::string::concat " "
                    (:wat::string::concat r2
                      (:wat::string::concat " " r3)))))]
            [:wat::core::Option::None {}
              (:wat::kernel::assertion-failed! :message "peer-pid None on process worker")])]
    out))
