;; Row 3: Stopped still carries the projected state after the close-wait.
;;
;; Run:
;;   ./target/release/wat wat-scripts/scratch-pad/probe-stop-means-gone-state.wat

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
     o (:hold::hold/stop h)]
    (:wat::core::match o
      ((:wat::service::StopOutcome::Stopped _)
        (:wat::kernel::println "stop=Stopped"))
      ((:wat::service::StopOutcome::Gone _)
        (:wat::kernel::println "stop=Gone"))
      ((:wat::service::StopOutcome::GaveUp w l)
        (:wat::kernel::println
          (:wat::core::format "stop=GaveUp waited={w} last={l}" :w w :l l))))))
