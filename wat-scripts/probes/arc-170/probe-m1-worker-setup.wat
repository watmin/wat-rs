;; probe-m1-worker-setup.wat — DISCONFIRMING PROBE for M1-pool shape (a), the ONE new composition:
;; a worker recv's a Setup(addr) over the wire → DIALS-and-HOLDS the service (admitted via grant) →
;; serves Work(item) messages using the HELD peer, reused across items. Isolates the runner
;; setup-dial-hold-reuse-over-a-union pattern (the rest of shape (a) is already-proven pieces).
;;
;; EXPECT (green):  echo:a echo:b   (two Work items served through one held, granted connection)

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
            (:wat::service::Outcome::Reply s
              (:probe::Echo::EchoResponse::Ok (:wat::string::concat "echo:" (:probe::Echo::EchoRequest/msg req)))))])

;; the union the worker recv's: Setup hands the address; Work is one unit of work.
(:wat::core::defenum :probe::Msg :wat::enum::Pure
  :Setup [addr <- (:wat::kernel::Address :- [:probe::Echo::Op :probe::Echo::Reply])]
  :Work  [s    <- :wat::core::String])

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::let
    [eh   (:probe::echo/start :locus (:wat::spawn::process) :record (:probe::echo::Record))
     ea   (:probe::echo::Handle/addr eh)
     worker (:wat::test::spawn-peer (:wat::spawn::process)
              (:wat::core::forms
                ;; child fresh world — re-declare the surface + the union it dials/receives
                (:wat::core::defsurface :probe::Echo :nature :wat::kernel::Peer
                  :messages
                  [(:wat::core::defrecord :probe::Echo::EchoRequest  [msg   <- :wat::core::String])
                   (:wat::core::defenum :probe::Echo::EchoResponse :wat::enum::Pure :Ok [reply <- :wat::core::String] :RequestTooLarge [bytes <- :wat::core::i64  cap <- :wat::core::i64]
                                                                                                                      :RequestMalformed [path <- (:wat::core::Vector :- [:wat::core::String])  expected <- :wat::core::String  got <- :wat::core::String])]
                  :features
                  [(echo [self <- :probe::Echo  req <- :probe::Echo::EchoRequest] -> :probe::Echo::EchoResponse :max-request-bytes 524288)])
                (:wat::core::defenum :probe::Msg :wat::enum::Pure
                  :Setup [addr <- (:wat::kernel::Address :- [:probe::Echo::Op :probe::Echo::Reply])]
                  :Work  [s    <- :wat::core::String])
                ;; the serve loop: threads the held service peer (Option, None until Setup)
                (:wat::core::defn :probe::serve
                  [self <- (:wat::kernel::Peer :- [:wat::core::String :probe::Msg])
                   held <- (:wat::core::Option (:wat::kernel::Peer :- [:probe::Echo::Op :probe::Echo::Reply]))]
                  -> :wat::core::nil
                  (:wat::core::match (:wat::kernel::recv self) 
                    ((:probe::Msg::Setup addr)
                      (:probe::serve self (:wat::core::Some (:wat::core::match (:wat::kernel::connect addr) ((:wat::kernel::ConnectOutcome::Connected p) p) ((:wat::kernel::ConnectOutcome::Refused c) (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message c) :wat::core::None :wat::core::None)) ((:wat::kernel::ConnectOutcome::Rejected c) (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message c) :wat::core::None :wat::core::None)) ((:wat::kernel::ConnectOutcome::Failed c) (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message c) :wat::core::None :wat::core::None))))))   ;; DIAL-and-HOLD
                    ((:probe::Msg::Work s)
                      (:wat::core::let
                        [c  (:wat::core::Option/expect held "Work before Setup")
                         er (:probe::Echo/echo c (:probe::Echo::EchoRequest :msg s))               ;; via the HELD peer
                         _  (:wat::core::match (:wat::kernel::send self (:wat::core::match er ((:probe::Echo::EchoResponse::Ok reply) reply)
  ((:probe::Echo::EchoResponse::RequestTooLarge bytes cap)
    (:wat::kernel::assertion-failed! "unexpected RequestTooLarge" :wat::core::None :wat::core::None))
  ((:probe::Echo::EchoResponse::RequestMalformed mpath mexpected mgot)
    (:wat::kernel::assertion-failed! "unexpected RequestMalformed" :wat::core::None :wat::core::None)))) (:wat::kernel::SendOutcome::Sent nil) (:wat::kernel::SendOutcome::Closed nil) ((:wat::kernel::SendOutcome::Lost _c) nil))]
                        (:probe::serve self held)))))
                (:wat::core::defn :user::main [] -> :wat::core::nil
                  (:wat::core::let
                    [self (:wat::program::self-peer :wat::core::String :probe::Msg)]
                    (:probe::serve self :wat::core::None)))))
     out  (:wat::core::match (:wat::kernel::peer-pid worker) 
            ((:wat::core::Some p)
              (:wat::core::let
                [_  (:probe::echo/grant eh (:wat::core::Vector :wat::core::i64 p)) ;; grant BEFORE the setup dial
                 _  (:wat::core::match (:wat::kernel::send worker (:probe::Msg::Setup ea)) (:wat::kernel::SendOutcome::Sent nil) (:wat::kernel::SendOutcome::Closed nil) (:wat::kernel::SendOutcome::Stopped nil) ((:wat::kernel::SendOutcome::Lost _c) nil))            ;; worker dials-and-holds (admitted)
                 _  (:wat::core::match (:wat::kernel::send worker (:probe::Msg::Work "a")) (:wat::kernel::SendOutcome::Sent nil) (:wat::kernel::SendOutcome::Closed nil) (:wat::kernel::SendOutcome::Stopped nil) ((:wat::kernel::SendOutcome::Lost _c) nil))
                 rr1 (:wat::kernel::recv worker)
                 r1  (:wat::core::match rr1
                       ((:wat::kernel::RecvOutcome::Message m) m)
                       ((:wat::kernel::RecvOutcome::Lost cause)
                         (:wat::kernel::assertion-failed! (:wat::kernel::LociDiedError/message cause) :wat::core::None :wat::core::None))
                       (:wat::kernel::RecvOutcome::Stopped
                         (:wat::kernel::assertion-failed! "recv': stopped — the substrate was asked to stop; the peer was ALIVE and the channel open" :wat::core::None :wat::core::None))
                       (:wat::kernel::RecvOutcome::Closed
                         (:wat::kernel::assertion-failed! "recv': worker closed unexpectedly" :wat::core::None :wat::core::None)))
                 _  (:wat::core::match (:wat::kernel::send worker (:probe::Msg::Work "b")) (:wat::kernel::SendOutcome::Sent nil) (:wat::kernel::SendOutcome::Closed nil) (:wat::kernel::SendOutcome::Stopped nil) ((:wat::kernel::SendOutcome::Lost _c) nil))
                 rr2 (:wat::kernel::recv worker)
                 r2  (:wat::core::match rr2
                       ((:wat::kernel::RecvOutcome::Message m) m)
                       ((:wat::kernel::RecvOutcome::Lost cause)
                         (:wat::kernel::assertion-failed! (:wat::kernel::LociDiedError/message cause) :wat::core::None :wat::core::None))
                       (:wat::kernel::RecvOutcome::Stopped
                         (:wat::kernel::assertion-failed! "recv': stopped — the substrate was asked to stop; the peer was ALIVE and the channel open" :wat::core::None :wat::core::None))
                       (:wat::kernel::RecvOutcome::Closed
                         (:wat::kernel::assertion-failed! "recv': worker closed unexpectedly" :wat::core::None :wat::core::None)))]
                (:wat::string::concat r1 (:wat::string::concat " " r2))))
            (:wat::core::None
              (:wat::kernel::assertion-failed! "peer-pid None on process worker"
                :wat::core::None :wat::core::None)))]
    (:wat::kernel::println out)))
