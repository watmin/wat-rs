;; GateOutcome::GaveUp — park the serve loop in a handler so Admin::Grant is
;; never answered; grant's owner-recv-loop spends its budget on silence.
;;
;; Run:
;;   ./target/release/wat wat-scripts/scratch-pad/probe-crash-surface-gate-gaveup.wat

(:wat::core::defsurface :hold::Hold :nature :wat::kernel::Peer
  :messages
  [(:wat::core::defrecord :hold::Hold::PingRequest [])
   (:wat::core::defenum :hold::Hold::PingResponse :wat::enum::Pure
     :Ok []
     :RequestTooLarge [bytes <- :wat::core::i64  cap <- :wat::core::i64]
     :RequestMalformed [path <- (:wat::core::Vector :- [:wat::core::String])
                        expected <- :wat::core::String  got <- :wat::core::String])]
  :features
  [(ping [self <- :hold::Hold  req <- :hold::Hold::PingRequest] -> :hold::Hold::PingResponse
     :max-request-bytes 65536)])

(:wat::service::defservice :hold::hold
  :satisfies :hold::Hold
  :durable []
  :ephemeral []
  :impls
  [(ping [s ctx req]
     (:wat::core::let
       [tmr (:wat::kernel::after :wat::program::PeerKind::process
               (:wat::time::Milliseconds 3600000) 0)
        _ (:wat::core::match (:wat::kernel::recv tmr)
            (_ nil))]
       (:wat::service::Outcome::Continue s
         (:wat::core::Some (:hold::Hold::Reply::Ping (:hold::Hold::PingResponse::Ok)))
         (:wat::core::Vector :- [(:wat::service::Directed :- [:hold::Hold::Reply])])
         (:wat::core::Vector :- [(:wat::service::Alarm :- [:hold::hold::Op])]))))])

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::let
    [h (:hold::hold/start :locus (:wat::spawn::process) :record (:hold::hold::Record))
     p (:wat::core::match (:wat::kernel::connect (:hold::hold::Handle/addr h))
         ((:wat::kernel::ConnectOutcome::Connected c) c)
         (_ (:wat::kernel::assertion-failed! "probe: connect failed" :wat::core::None :wat::core::None)))
     inert (:hold::Hold::Reply::Ping (:hold::Hold::PingResponse::RequestTooLarge 0 0))
     ;; Park the serve loop in ping (1-hour timer) and wait out the client deadline
     ;; so Grant cannot race ahead of the handler.
     _ (:wat::core::match
         (:wat::service::call-by-deadline p
           (:hold::Hold::Op::Ping (:hold::Hold::PingRequest)) 300 inert)
         (_ nil))
     g (:wat::capability::Capability/grant h (:wat::core::Vector :- [:wat::core::i64] 42))]
    (:wat::core::match g
      ((:wat::service::GateOutcome::Applied)
        (:wat::kernel::println "grant-stuck-handler=Applied"))
      ((:wat::service::GateOutcome::Gone _)
        (:wat::kernel::println "grant-stuck-handler=Gone"))
      ((:wat::service::GateOutcome::GaveUp w l)
        (:wat::kernel::println
          (:wat::core::format "grant-stuck-handler=GaveUp waited={w} last={l}"
            :w w :l l))))))
