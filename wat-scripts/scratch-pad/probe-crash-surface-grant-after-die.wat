;; GateOutcome after the service has died (raise! on ping).
;;
;; Run:
;;   ./target/release/wat wat-scripts/scratch-pad/probe-crash-surface-grant-after-die.wat

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
     _ (:wat::core::match (:hold::Hold/ping p (:hold::Hold::PingRequest))
         (_ nil))
     g (:wat::capability::Capability/grant h (:wat::core::Vector :- [:wat::core::i64] 42))]
    (:wat::core::match g
      ((:wat::service::GateOutcome::Applied)
        (:wat::kernel::println "grant-after-die=Applied"))
      ((:wat::service::GateOutcome::Gone _)
        (:wat::kernel::println "grant-after-die=Gone"))
      ((:wat::service::GateOutcome::GaveUp _w _l)
        (:wat::kernel::println "grant-after-die=GaveUp")))))
