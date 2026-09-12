;; Oversized frame against :max-frame-bytes 256. The old frame-cap probe's
;; Malformed arm is an assertion-failed placeholder; this one NAMES the variant.
;;
;; Run:
;;   ./target/release/wat wat-scripts/scratch-pad/probe-crash-surface-frame-cap.wat

(:wat::core::defsurface :fc::Echo :nature :wat::kernel::Peer
  :messages
  [(:wat::core::defrecord :fc::Echo::PingRequest [pad <- :wat::core::String])
   (:wat::core::defenum :fc::Echo::PingResponse :wat::enum::Pure
     :Ok               []
     :RequestTooLarge  [bytes <- :wat::core::i64  cap <- :wat::core::i64]
     :RequestMalformed [path <- (:wat::core::Vector :- [:wat::core::String])
                        expected <- :wat::core::String  got <- :wat::core::String])]
  :features
  [(ping [self <- :fc::Echo  req <- :fc::Echo::PingRequest]
     -> :fc::Echo::PingResponse :max-request-bytes 524288)])

(:wat::service::defservice :fc::echo
  :satisfies :fc::Echo
  :max-frame-bytes 256
  :durable   []
  :ephemeral []
  :impls
  [(ping [s ctx req]
     (:wat::service::Outcome::Continue s
       (:wat::core::Some (:fc::Echo::Reply::Ping (:fc::Echo::PingResponse::Ok)))
       (:wat::core::Vector :- [(:wat::service::Directed :- [:fc::Echo::Reply])])
       (:wat::core::Vector :- [(:wat::service::Alarm :- [:fc::echo::Op])])))])

(:wat::core::defn :fc::dial
  [a <- (:wat::kernel::Address :- [:fc::Echo::Op :fc::Echo::Reply])]
  -> (:wat::kernel::Peer :- [:fc::Echo::Op :fc::Echo::Reply])
  (:wat::core::match (:wat::kernel::connect a)
    ((:wat::kernel::ConnectOutcome::Connected c) c)
    (_ (:wat::kernel::assertion-failed! "fc: dial failed" :wat::core::None :wat::core::None))))

(:wat::core::defn :fc::pad [n <- :wat::core::i64] -> :wat::core::String
  (:wat::core::foldl
    (:wat::core::fn [acc <- :wat::core::String  _i <- :wat::core::i64] -> :wat::core::String
      (:wat::string::concat acc "xxxxxxxxxx"))
    ""
    (:wat::core::range 0 n)))

(:wat::core::defn :fc::label
  [c <- (:wat::kernel::Peer :- [:fc::Echo::Op :fc::Echo::Reply])  pad <- :wat::core::String] -> :wat::core::String
  (:wat::core::match (:fc::Echo/ping c (:fc::Echo::PingRequest :pad pad))
    ((:wat::kernel::RecvOutcome::Message r)
      (:wat::core::match r
        ((:fc::Echo::PingResponse::Ok) "Message/Ok")
        ((:fc::Echo::PingResponse::RequestTooLarge _b _c) "Message/TooLarge")
        (_ "Message/other")))
    ((:wat::kernel::RecvOutcome::Lost _) "Lost")
    (:wat::kernel::RecvOutcome::Stopped "Stopped")
    (:wat::kernel::RecvOutcome::Closed "Closed")
    (:wat::kernel::RecvOutcome::TimedOut "TimedOut")
    ((:wat::kernel::RecvOutcome::Malformed _) "Malformed")))

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::let
    [h  (:fc::echo/start :locus (:wat::spawn::process) :record (:fc::echo::Record))
     a  (:fc::dial (:fc::echo::Handle/addr h))
     b  (:fc::dial (:fc::echo::Handle/addr h))
     small (:fc::label a "")
     big   (:fc::label a (:fc::pad 200))
     other (:fc::label b "")
     again (:fc::label a "")
     send-torn (:wat::core::match
                 (:wat::kernel::send a (:fc::Echo::Op::Ping (:fc::Echo::PingRequest :pad "")))
                 (:wat::kernel::SendOutcome::Sent "Sent")
                 (:wat::kernel::SendOutcome::Closed "Closed")
                 (:wat::kernel::SendOutcome::Stopped "Stopped")
                 ((:wat::kernel::SendOutcome::Lost _) "Lost"))
     inert (:fc::Echo::Reply::Ping (:fc::Echo::PingResponse::RequestTooLarge 0 0))
     call-torn (:wat::core::match
                 (:wat::service::call-by-deadline a
                   (:fc::Echo::Op::Ping (:fc::Echo::PingRequest :pad ""))
                   300 inert)
                 ((:wat::service::CallOutcome::Answered _) "Answered")
                 ((:wat::service::CallOutcome::DeadlineFired) "DeadlineFired")
                 ((:wat::service::CallOutcome::Closed) "Closed")
                 ((:wat::service::CallOutcome::Lost _) "Lost")
                 ((:wat::service::CallOutcome::Malformed _) "Malformed"))
     call-big (:wat::core::match
                (:wat::service::call-by-deadline b
                  (:fc::Echo::Op::Ping (:fc::Echo::PingRequest :pad (:fc::pad 200)))
                  300 inert)
                ((:wat::service::CallOutcome::Answered _) "Answered")
                ((:wat::service::CallOutcome::DeadlineFired) "DeadlineFired")
                ((:wat::service::CallOutcome::Closed) "Closed")
                ((:wat::service::CallOutcome::Lost _) "Lost")
                ((:wat::service::CallOutcome::Malformed _) "Malformed"))
     _ (:wat::service::stop-faced (:fc::echo/stop h))]
    (:wat::kernel::println
      (:wat::core::format "a-small={s};a-big={g};b-other={o};a-again={r};send-torn={t};call-torn={c};call-big={k}"
        :s small :g big :o other :r again :t send-torn :c call-torn :k call-big))))
