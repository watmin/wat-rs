;; the-inert-clause-is-refused — the refusal fixture.
;; Probe 1's shape (`:slow::slow` in `:satisfies` mode declaring `:deadline-ms 300`).
;; Before D4-b this compiled and measured `outcome=TimedOut;elapsed-ms=10000;declared=300`.
;; After: `#wat.macro/MalformedTemplate` at expand time. This file must never run.

(:wat::core::defsurface :slow::Slow :nature :wat::kernel::Peer
  :messages
  [(:wat::core::defrecord :slow::Slow::PingRequest [])
   (:wat::core::defenum :slow::Slow::PingResponse :wat::enum::Pure
     :Ok []
     :RequestTooLarge [bytes <- :wat::core::i64  cap <- :wat::core::i64]
     :RequestMalformed [path <- (:wat::core::Vector :- [:wat::core::String])
                        expected <- :wat::core::String  got <- :wat::core::String])]
  :features
  [(ping [self <- :slow::Slow  req <- :slow::Slow::PingRequest] -> :slow::Slow::PingResponse
     :max-request-bytes 65536)])

(:wat::service::defservice :slow::slow
  :satisfies :slow::Slow
  :deadline-ms 300
  :durable []
  :ephemeral []
  :impls
  [(ping [s ctx req]
     (:wat::service::Outcome::Continue s
       (:wat::core::Some (:slow::Slow::Reply::Ping (:slow::Slow::PingResponse::Ok)))
       (:wat::core::Vector :- [(:wat::service::Directed :- [:slow::Slow::Reply])])
       (:wat::core::Vector :- [(:wat::service::Alarm :- [:slow::slow::Op])])))])
