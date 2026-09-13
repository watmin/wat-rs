;; Does a CALLEE's :deadline-ms set the CALLER's observed timeout?
;;
;; Load-bearing for the store-fault injector's cost model: if a proxy can declare
;; :deadline-ms 300, a dropped reply costs 300 ms. If the deadline is fixed per
;; SURFACE (default 10000), every injected fault costs 10 s and the harness has to
;; be sized around that.
;;
;; The handler parks on a 1-hour timer, so the reply never comes. We measure how
;; long the client's generated method takes to give up.
;;
;; Run:
;;   ./target/release/wat wat-scripts/scratch-pad/probe-deadline-ms-is-callee-derived.wat

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
     ;; park forever — the reply never goes out. Idiom copied verbatim from
     ;; probe-crash-surface-gate-gaveup.wat:26 (a cited exemplar, not a sketch).
     (:wat::core::let
       [tmr (:wat::kernel::after :wat::program::PeerKind::process
               (:wat::time::Milliseconds 3600000) 0)
        _ (:wat::core::match (:wat::kernel::recv tmr)
            (_ nil))]
       (:wat::service::Outcome::Continue s
         (:wat::core::Some (:slow::Slow::Reply::Ping (:slow::Slow::PingResponse::Ok)))
         (:wat::core::Vector :- [(:wat::service::Directed :- [:slow::Slow::Reply])])
         (:wat::core::Vector :- [(:wat::service::Alarm :- [:slow::slow::Op])]))))])

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::let
    [h (:slow::slow/start :locus (:wat::spawn::process) :record (:slow::slow::Record))
     p (:wat::core::match (:wat::kernel::connect (:slow::slow::Handle/addr h))
         ((:wat::kernel::ConnectOutcome::Connected c) c)
         (_ (:wat::kernel::assertion-failed! "probe: connect failed" :wat::core::None :wat::core::None)))
     t0 (:wat::time::epoch-nanos (:wat::time::now))
     r (:wat::core::match (:slow::Slow/ping p (:slow::Slow::PingRequest))
         ((:wat::kernel::RecvOutcome::Message _m) "Message")
         (:wat::kernel::RecvOutcome::TimedOut "TimedOut")
         (:wat::kernel::RecvOutcome::Closed "Closed")
         (:wat::kernel::RecvOutcome::Stopped "Stopped")
         ((:wat::kernel::RecvOutcome::Lost _c) "Lost")
         ((:wat::kernel::RecvOutcome::Malformed _c) "Malformed"))
     el (:wat::i64::/ (:wat::i64::- (:wat::time::epoch-nanos (:wat::time::now)) t0) 1000000)
     _ (:wat::kernel::println
         (:wat::core::format "outcome={r};elapsed-ms={e};declared=300" :r r :e el))]
    nil))
