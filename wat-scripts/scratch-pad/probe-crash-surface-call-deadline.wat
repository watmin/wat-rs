;; CallOutcome::DeadlineFired — call-by-deadline against a service that accepts
;; and never replies. Also RecvOutcome::TimedOut via the generated method mapping.
;;
;; Run:
;;   ./target/release/wat wat-scripts/scratch-pad/probe-crash-surface-call-deadline.wat

(:wat::core::defsurface :dp::Silent :nature :wat::kernel::Peer
  :messages
  [(:wat::core::defrecord :dp::Silent::WaitRequest [])
   (:wat::core::defenum :dp::Silent::WaitResponse :wat::enum::Pure
     :Ok []
     :RequestTooLarge  [bytes <- :wat::core::i64  cap <- :wat::core::i64]
     :RequestMalformed [path <- (:wat::core::Vector :- [:wat::core::String])
                        expected <- :wat::core::String  got <- :wat::core::String])]
  :features
  [(wait [self <- :dp::Silent  req <- :dp::Silent::WaitRequest]
     -> :dp::Silent::WaitResponse :max-request-bytes 65536)])

(:wat::service::defservice :dp::silent
  :satisfies :dp::Silent
  :durable   [tag <- :wat::core::i64]
  :ephemeral []
  :init (:wat::core::fn [record <- :dp::silent::Record] -> :dp::silent::State
          (:dp::silent::State :durable record))
  :impls
  [(wait [s ctx req]
     (:wat::service::Outcome::Continue s
       :wat::core::None
       (:wat::core::Vector :- [(:wat::service::Directed :- [:dp::Silent::Reply])])
       (:wat::core::Vector :- [(:wat::service::Alarm :- [:dp::silent::Op])])))])

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::let
    [h (:dp::silent/start :locus (:wat::spawn::process)
          :record (:dp::silent::Record :tag 1))
     p (:wat::core::match (:wat::kernel::connect (:dp::silent::Handle/addr h))
         ((:wat::kernel::ConnectOutcome::Connected c) c)
         (_ (:wat::kernel::assertion-failed! "probe: connect failed" :wat::core::None :wat::core::None)))
     inert (:dp::Silent::Reply::Wait (:dp::Silent::WaitResponse::RequestTooLarge 0 0))
     o (:wat::service::call-by-deadline p (:dp::Silent::Op::Wait (:dp::Silent::WaitRequest)) 300 inert)
     _ (:wat::core::match o
         ((:wat::service::CallOutcome::Answered _)
           (:wat::kernel::println "call-by-deadline=Answered"))
         ((:wat::service::CallOutcome::DeadlineFired)
           (:wat::kernel::println "call-by-deadline=DeadlineFired"))
         ((:wat::service::CallOutcome::Closed)
           (:wat::kernel::println "call-by-deadline=Closed"))
         ((:wat::service::CallOutcome::Lost _)
           (:wat::kernel::println "call-by-deadline=Lost"))
         ((:wat::service::CallOutcome::Malformed _)
           (:wat::kernel::println "call-by-deadline=Malformed")))
     _ (:wat::service::stop-faced (:dp::silent/stop h))]
    nil))
