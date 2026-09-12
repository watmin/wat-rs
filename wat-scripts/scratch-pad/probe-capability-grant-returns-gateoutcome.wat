;; Row 5 of the-gate-outcome-outlives-its-file: a Capability-tier grant
;; RETURNS a GateOutcome. Zero userland callers, so the floor will not
;; give this. Grant through Capability, revoke through TypedCapability
;; (the sibling surface, served off the same grantable-extend bodies).
;;
;; Run:
;;   ./target/release/wat wat-scripts/scratch-pad/probe-capability-grant-returns-gateoutcome.wat

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
     pids (:wat::core::Vector :- [:wat::core::i64] 42)
     g (:wat::capability::Capability/grant h pids)
     r (:wat::capability::TypedCapability/revoke h pids)
     _ (:wat::kernel::println g)
     _ (:wat::kernel::println r)
     _ (:wat::service::stop-faced (:hold::hold/stop h))]
    nil))
