;; D4 PROBE 2 — does a service's :deadline-ms govern the calls ITS OWN HANDLERS make?
;;
;; Probe 1 (committed, NOTE-a-callee-deadline-ms-is-inert.md): a :satisfies-mode
;; CALLEE declaring :deadline-ms 300 does NOT shorten what its caller waits — 10000
;; observed. This asked the other direction, and decided whether circuit.wat's
;; live `:deadline-ms 120000` was doing anything at all.
;;
;; MEASURED (before D4-b refused the clause):
;;   handler-faced=TimedOut;handler-elapsed-ms=10000;declared=300
;; 10000 observed against 300 declared. Outbound calls are not governed either.
;;
;; D4-b (`the-inert-clause-is-refused`) makes `:deadline-ms` in `:satisfies` mode a
;; compile-time refusal. The clause is dropped here so this file still loads; the
;; numbers live in that stone's SCORE.
;;
;; fast :peers [:mid::Mid], dials mid in :init (the only sanctioned shape — the
;; third bijection check, 49bdf2b41). mid PARKS forever.

(:wat::core::defsurface :mid::Mid :nature :wat::kernel::Peer
  :messages
  [(:wat::core::defrecord :mid::Mid::PingRequest [])
   (:wat::core::defenum :mid::Mid::PingResponse :wat::enum::Pure
     :Ok []
     :RequestTooLarge [bytes <- :wat::core::i64  cap <- :wat::core::i64]
     :RequestMalformed [path <- (:wat::core::Vector :- [:wat::core::String])
                        expected <- :wat::core::String  got <- :wat::core::String])]
  :features
  [(ping [self <- :mid::Mid  req <- :mid::Mid::PingRequest] -> :mid::Mid::PingResponse
     :max-request-bytes 65536)])

(:wat::service::defservice :mid::mid
  :satisfies :mid::Mid
  :durable []
  :ephemeral []
  :impls
  [(ping [s ctx req]
     (:wat::core::let
       [tmr (:wat::kernel::after :wat::program::PeerKind::process
               (:wat::time::Milliseconds 3600000) 0)
        _ (:wat::core::match (:wat::kernel::recv tmr) (_ nil))]
       (:wat::service::Outcome::Continue s
         (:wat::core::Some (:mid::Mid::Reply::Ping (:mid::Mid::PingResponse::Ok)))
         (:wat::core::Vector :- [(:wat::service::Directed :- [:mid::Mid::Reply])])
         (:wat::core::Vector :- [(:wat::service::Alarm :- [:mid::mid::Op])]))))])

(:wat::core::defsurface :fast::Fast :nature :wat::kernel::Peer
  :messages
  [(:wat::core::defrecord :fast::Fast::AskRequest [])
   (:wat::core::defenum :fast::Fast::AskResponse :wat::enum::Pure
     :Faced [arm <- :wat::core::String  elapsed-ms <- :wat::core::i64]
     :RequestTooLarge [bytes <- :wat::core::i64  cap <- :wat::core::i64]
     :RequestMalformed [path <- (:wat::core::Vector :- [:wat::core::String])
                        expected <- :wat::core::String  got <- :wat::core::String])]
  :features
  [(ask [self <- :fast::Fast  req <- :fast::Fast::AskRequest] -> :fast::Fast::AskResponse
     :max-request-bytes 65536)])

(:wat::service::defservice :fast::fast
  :satisfies :fast::Fast
  :peers [:mid::Mid]
  :durable [mid-addr <- (:wat::kernel::Address :- [:mid::Mid::Op :mid::Mid::Reply])]
  :ephemeral [mid <- (:wat::kernel::Peer :- [:mid::Mid::Op :mid::Mid::Reply])]
  :init (:wat::core::fn [record <- :fast::fast::Record] -> :fast::fast::State
          (:fast::fast::State :durable record
            :mid (:wat::core::match (:wat::kernel::connect (:fast::fast::Record/mid-addr record))
                   ((:wat::kernel::ConnectOutcome::Connected p) p)
                   (_ (:wat::kernel::assertion-failed! "init: mid dial failed" :wat::core::None :wat::core::None)))))
  :impls
  [(ask [s ctx req]
     (:wat::core::let
       [t0 (:wat::time::epoch-nanos (:wat::time::now))
        arm (:wat::core::match (:mid::Mid/ping (:fast::fast::State/mid s) (:mid::Mid::PingRequest))
              ((:wat::kernel::RecvOutcome::Message _m) "Message")
              (:wat::kernel::RecvOutcome::TimedOut "TimedOut")
              (:wat::kernel::RecvOutcome::Closed "Closed")
              (:wat::kernel::RecvOutcome::Stopped "Stopped")
              ((:wat::kernel::RecvOutcome::Lost _c) "Lost")
              ((:wat::kernel::RecvOutcome::Malformed _c) "Malformed"))
        el (:wat::i64::/ (:wat::i64::- (:wat::time::epoch-nanos (:wat::time::now)) t0) 1000000)]
       (:wat::service::Outcome::Continue s
         (:wat::core::Some (:fast::Fast::Reply::Ask (:fast::Fast::AskResponse::Faced arm el)))
         (:wat::core::Vector :- [(:wat::service::Directed :- [:fast::Fast::Reply])])
         (:wat::core::Vector :- [(:wat::service::Alarm :- [:fast::fast::Op])]))))])

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::let
    [mh (:mid::mid/start :locus (:wat::spawn::process) :record (:mid::mid::Record))
     fh (:fast::fast/start
          :locus (:wat::spawn::process/post-spawn
                   (:wat::core::fn [pl <- :wat::spawn::ProcessLaunch] -> :wat::core::nil
                     (:wat::service::require-granted
                       (:mid::mid/grant mh
                         (:wat::core::Vector :- [:wat::core::i64] (:wat::spawn::ProcessLaunch/pid pl))))))
          :record (:fast::fast::Record :mid-addr (:mid::mid::Handle/addr mh)))
     fp (:wat::core::match (:wat::kernel::connect (:fast::fast::Handle/addr fh))
          ((:wat::kernel::ConnectOutcome::Connected c) c)
          (_ (:wat::kernel::assertion-failed! "probe: fast connect failed" :wat::core::None :wat::core::None)))
     out (:wat::core::match
           (:wat::service::call-by-deadline fp (:fast::Fast::Op::Ask (:fast::Fast::AskRequest)) 40000
             (:fast::Fast::Reply::Ask (:fast::Fast::AskResponse::RequestTooLarge 0 0)))
           ((:wat::service::CallOutcome::Answered r)
             (:wat::core::match r
               ((:fast::Fast::Reply::Ask ar)
                 (:wat::core::match ar
                   ((:fast::Fast::AskResponse::Faced arm el)
                     (:wat::core::format "OUTBOUND handler-faced={a};handler-elapsed-ms={e};fast-declared-deadline=300" :a arm :e el))
                   ((:fast::Fast::AskResponse::RequestTooLarge _b _c) "too-large")
                   ((:fast::Fast::AskResponse::RequestMalformed _p _e _g) "malformed")))
               (_ "other-reply")))
           ((:wat::service::CallOutcome::DeadlineFired) "outer-DeadlineFired")
           ((:wat::service::CallOutcome::Lost c) (:wat::core::format "outer-Lost cause={c}" :c (:wat::edn::write c)))
           ((:wat::service::CallOutcome::Closed) "outer-Closed")
           ((:wat::service::CallOutcome::Malformed _c) "outer-Malformed"))
     _ (:wat::kernel::println out)]
    nil))
