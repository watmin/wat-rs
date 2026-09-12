;; TrySendOutcome::WouldBlock — flood a process-tier client that is not draining.
;; The service never reads; we try-send until WouldBlock or a cap.
;;
;; Run:
;;   ./target/release/wat wat-scripts/scratch-pad/probe-crash-surface-try-send-wouldblock.wat

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
   n <- :wat::core::i64
   cap <- :wat::core::i64] -> :wat::core::String
  (:wat::core::if (:wat::i64::>= n cap)
    (:wat::core::format "gave-up-after={n}" :n n)
    (:wat::core::match
      (:wat::kernel::try-send p (:hold::Hold::Op::Ping (:hold::Hold::PingRequest)))
      (:wat::kernel::TrySendOutcome::Sent (:cs::flood p (:wat::i64::+ n 1) cap))
      (:wat::kernel::TrySendOutcome::WouldBlock
        (:wat::core::format "WouldBlock-after={n}" :n n))
      (:wat::kernel::TrySendOutcome::Closed
        (:wat::core::format "Closed-after={n}" :n n))
      ((:wat::kernel::TrySendOutcome::Lost _)
        (:wat::core::format "Lost-after={n}" :n n)))))

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::let
    [h (:hold::hold/start :locus (:wat::spawn::process) :record (:hold::hold::Record))
     p (:wat::core::match (:wat::kernel::connect (:hold::hold::Handle/addr h))
         ((:wat::kernel::ConnectOutcome::Connected c) c)
         (_ (:wat::kernel::assertion-failed! "probe: connect failed" :wat::core::None :wat::core::None)))
     r (:cs::flood p 0 100000)
     _ (:wat::kernel::println r)
     _ (:wat::service::stop-faced (:hold::hold/stop h))]
    nil))
