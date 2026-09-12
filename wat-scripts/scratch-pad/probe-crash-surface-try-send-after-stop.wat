;; TrySendOutcome / SendOutcome after a CLEAN stop (not a panic). The connection
;; should see Closed rather than Lost.
;;
;; Run:
;;   ./target/release/wat wat-scripts/scratch-pad/probe-crash-surface-try-send-after-stop.wat

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
       (:wat::core::Some (:hold::Hold::Reply::Ping (:hold::Hold::PingResponse::Ok)))
       (:wat::core::Vector :- [(:wat::service::Directed :- [:hold::Hold::Reply])])
       (:wat::core::Vector :- [(:wat::service::Alarm :- [:hold::hold::Op])])))])

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::let
    [h (:hold::hold/start :locus (:wat::spawn::process) :record (:hold::hold::Record))
     p (:wat::core::match (:wat::kernel::connect (:hold::hold::Handle/addr h))
         ((:wat::kernel::ConnectOutcome::Connected c) c)
         (_ (:wat::kernel::assertion-failed! "probe: connect failed" :wat::core::None :wat::core::None)))
     _ (:wat::service::stop-faced (:hold::hold/stop h))
     t (:wat::core::match
         (:wat::kernel::try-send p (:hold::Hold::Op::Ping (:hold::Hold::PingRequest)))
         (:wat::kernel::TrySendOutcome::Sent "Sent")
         (:wat::kernel::TrySendOutcome::WouldBlock "WouldBlock")
         (:wat::kernel::TrySendOutcome::Closed "Closed")
         ((:wat::kernel::TrySendOutcome::Lost _) "Lost"))
     s (:wat::core::match
         (:wat::kernel::send p (:hold::Hold::Op::Ping (:hold::Hold::PingRequest)))
         (:wat::kernel::SendOutcome::Sent "Sent")
         (:wat::kernel::SendOutcome::Closed "Closed")
         (:wat::kernel::SendOutcome::Stopped "Stopped")
         ((:wat::kernel::SendOutcome::Lost _) "Lost"))]
    (:wat::kernel::println
      (:wat::core::format "try-send-after-stop={t};send-after-stop={s}" :t t :s s))))
