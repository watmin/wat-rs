;; Row 3 of the-owner-wait-has-a-deadline: a silent lineage wait returns
;; GaveUp last=TimedOut instead of hanging. `/stop` is send + this loop;
;; the hang was the recv. We call the loop with a 500 ms budget and no send,
;; then `/stop` for cleanup (the service is idle and replies).
;;
;; Run:
;;   ./target/release/wat wat-scripts/scratch-pad/probe-silent-stop-gaveup.wat

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
     lineage (:hold::hold::Handle/handle h)
     t0 (:wat::time::epoch-nanos (:wat::time::now))
     o (:wat::service::owner-recv-loop lineage t0 500 "recv")
     _ (:wat::kernel::println o)
     _ (:wat::service::stop-faced (:hold::hold/stop h))]
    nil))
