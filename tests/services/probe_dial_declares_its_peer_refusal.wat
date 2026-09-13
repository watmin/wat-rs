;; the-dial-declares-its-peer — the refusal. Variant A of
;; FINDING-a-handler-local-dial-is-a-runtime-failure-that-could-be-compile-time.md:
;; a handler `connect`s an Address-typed durable field of another surface with no
;; `:peers`. Before this stone: the forked child died (`Disconnected []` on the
;; dialed peer; `RuntimeError ["unknown function: …"]` on the lineage). After:
;; `#wat.macro/MalformedTemplate` at expand time. This file must never run.

(:wat::core::defsurface :probe::Mid :nature :wat::kernel::Peer
  :messages
  [(:wat::core::defrecord :probe::Mid::PingRequest  [msg <- :wat::core::String])
   (:wat::core::defenum :probe::Mid::PingResponse :wat::enum::Pure
     :Ok              [reply <- :wat::core::String]
     :RequestTooLarge [bytes <- :wat::core::i64  cap <- :wat::core::i64]
     :RequestMalformed [path <- (:wat::core::Vector :- [:wat::core::String])  expected <- :wat::core::String  got <- :wat::core::String])]
  :features
  [(ping [self <- :probe::Mid  req <- :probe::Mid::PingRequest] -> :probe::Mid::PingResponse :max-request-bytes 524288)])

(:wat::service::defservice :probe::mid
  :satisfies :probe::Mid
  :durable   []
  :ephemeral []
  :impls
  [(ping [s ctx req]
     (:wat::service::Outcome::Continue s
       (:wat::core::Some (:probe::Mid::Reply::Ping (:probe::Mid::PingResponse::Ok
         (:wat::string::concat "pong:" (:probe::Mid::PingRequest/msg req)))))
       (:wat::core::Vector :- [(:wat::service::Directed :- [:probe::Mid::Reply])])
       (:wat::core::Vector :- [(:wat::service::Alarm :- [:probe::mid::Op])])))])

(:wat::core::defsurface :probe::Front :nature :wat::kernel::Peer
  :messages
  [(:wat::core::defrecord :probe::Front::RunRequest  [])
   (:wat::core::defenum :probe::Front::RunResponse :wat::enum::Pure
     :Ok              [out <- :wat::core::String]
     :RequestTooLarge [bytes <- :wat::core::i64  cap <- :wat::core::i64]
     :RequestMalformed [path <- (:wat::core::Vector :- [:wat::core::String])  expected <- :wat::core::String  got <- :wat::core::String])]
  :features
  [(run [self <- :probe::Front  req <- :probe::Front::RunRequest] -> :probe::Front::RunResponse :max-request-bytes 524288)])

(:wat::service::defservice :probe::front
  :satisfies :probe::Front
  :durable   [mid-addr <- (:wat::kernel::Address :- [:probe::Mid::Op :probe::Mid::Reply])]
  :ephemeral []
  :impls
  [(run [s ctx req]
     (:wat::core::let
       [_p (:wat::core::match
             (:wat::kernel::connect (:probe::front::Record/mid-addr (:probe::front::State/durable s)))
             ((:wat::kernel::ConnectOutcome::Connected p) p)
             (_ (:wat::kernel::assertion-failed! "front: unused — this file must not compile" :wat::core::None :wat::core::None)))]
       (:wat::service::Outcome::Continue s
         (:wat::core::Some (:probe::Front::Reply::Run (:probe::Front::RunResponse::Ok "never")))
         (:wat::core::Vector :- [(:wat::service::Directed :- [:probe::Front::Reply])])
         (:wat::core::Vector :- [(:wat::service::Alarm :- [:probe::front::Op])]))))])
