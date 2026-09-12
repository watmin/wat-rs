;; SendOutcome::Stopped — block in send filling a silent peer, then SIGTERM.
;; Prints READY, then floods send. Wrapper SIGTERMs after READY.
;;
;; Run (wrapper):
;;   ./target/release/wat wat-scripts/scratch-pad/probe-crash-surface-send-stopped.wat

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
     (:wat::service::Outcome::Continue s
       :wat::core::None
       (:wat::core::Vector :- [(:wat::service::Directed :- [:hold::Hold::Reply])])
       (:wat::core::Vector :- [(:wat::service::Alarm :- [:hold::hold::Op])])))])

(:wat::core::defn :cs::flood
  [p <- (:wat::kernel::Peer :- [:hold::Hold::Op :hold::Hold::Reply])
   n <- :wat::core::i64] -> :wat::core::String
  (:wat::core::match
    (:wat::kernel::send p (:hold::Hold::Op::Ping (:hold::Hold::PingRequest)))
    (:wat::kernel::SendOutcome::Sent (:cs::flood p (:wat::i64::+ n 1)))
    (:wat::kernel::SendOutcome::Closed
      (:wat::core::format "Closed-after={n}" :n n))
    (:wat::kernel::SendOutcome::Stopped
      (:wat::core::format "Stopped-after={n}" :n n))
    ((:wat::kernel::SendOutcome::Lost _)
      (:wat::core::format "Lost-after={n}" :n n))))

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::let
    [h (:hold::hold/start :locus (:wat::spawn::process) :record (:hold::hold::Record))
     p (:wat::core::match (:wat::kernel::connect (:hold::hold::Handle/addr h))
         ((:wat::kernel::ConnectOutcome::Connected c) c)
         (_ (:wat::kernel::assertion-failed! "probe: connect failed" :wat::core::None :wat::core::None)))
     _ (:wat::kernel::println "READY")
     r (:cs::flood p 0)]
    (:wat::kernel::println r)))
