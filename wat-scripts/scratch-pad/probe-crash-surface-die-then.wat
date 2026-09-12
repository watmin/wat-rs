;; Kill the service from inside (raise! on ping), then observe stop / grant /
;; call / send / try-send on the corpse. Avoids signal, which type-checks only
;; against Process, while Handle/handle is a typed Admin Peer.
;;
;; Run:
;;   ./target/release/wat wat-scripts/scratch-pad/probe-crash-surface-die-then.wat

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
     (:wat::kernel::raise! (:wat::core::Fault/of "die-on-ping")))])

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::let
    [h (:hold::hold/start :locus (:wat::spawn::process) :record (:hold::hold::Record))
     p (:wat::core::match (:wat::kernel::connect (:hold::hold::Handle/addr h))
         ((:wat::kernel::ConnectOutcome::Connected c) c)
         (_ (:wat::kernel::assertion-failed! "probe: connect failed" :wat::core::None :wat::core::None)))
     ping1 (:wat::core::match (:hold::Hold/ping p (:hold::Hold::PingRequest))
             ((:wat::kernel::RecvOutcome::Message _) "Message")
             (:wat::kernel::RecvOutcome::Closed "Closed")
             (:wat::kernel::RecvOutcome::Stopped "Stopped")
             (:wat::kernel::RecvOutcome::TimedOut "TimedOut")
             ((:wat::kernel::RecvOutcome::Lost _) "Lost")
             ((:wat::kernel::RecvOutcome::Malformed _) "Malformed"))
     inert (:hold::Hold::Reply::Ping (:hold::Hold::PingResponse::RequestTooLarge 0 0))
     call1 (:wat::core::match
             (:wat::service::call-by-deadline p
               (:hold::Hold::Op::Ping (:hold::Hold::PingRequest)) 500 inert)
             ((:wat::service::CallOutcome::Answered _) "Answered")
             ((:wat::service::CallOutcome::DeadlineFired) "DeadlineFired")
             ((:wat::service::CallOutcome::Closed) "Closed")
             ((:wat::service::CallOutcome::Lost _) "Lost")
             ((:wat::service::CallOutcome::Malformed _) "Malformed"))
     send1 (:wat::core::match
             (:wat::kernel::send p (:hold::Hold::Op::Ping (:hold::Hold::PingRequest)))
             (:wat::kernel::SendOutcome::Sent "Sent")
             (:wat::kernel::SendOutcome::Closed "Closed")
             (:wat::kernel::SendOutcome::Stopped "Stopped")
             ((:wat::kernel::SendOutcome::Lost _) "Lost"))
     try1 (:wat::core::match
            (:wat::kernel::try-send p (:hold::Hold::Op::Ping (:hold::Hold::PingRequest)))
            (:wat::kernel::TrySendOutcome::Sent "Sent")
            (:wat::kernel::TrySendOutcome::WouldBlock "WouldBlock")
            (:wat::kernel::TrySendOutcome::Closed "Closed")
            ((:wat::kernel::TrySendOutcome::Lost _) "Lost"))
     stop1 (:wat::core::match (:hold::hold/stop h)
             ((:wat::service::StopOutcome::Stopped _) "Stopped")
             ((:wat::service::StopOutcome::Gone _) "Gone")
             ((:wat::service::StopOutcome::GaveUp _w _l) "GaveUp"))]
    (:wat::kernel::println
      (:wat::core::format "ping-die={p};call-after={c};send-after={s};try-send-after={t};stop-after={o}"
        :p ping1 :c call1 :s send1 :t try1 :o stop1))))
