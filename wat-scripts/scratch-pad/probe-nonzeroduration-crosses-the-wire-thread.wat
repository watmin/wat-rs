;; Thread-locus twin of probe-nonzeroduration-crosses-the-wire.wat.
;; Discriminator: thread never reconstructs from EDN (crossbeam Value).
(:wat::config::set-redef! true)

(:wat::core::defsurface :wt::Echo :nature :wat::kernel::Peer
  :messages
  [(:wat::core::defenum :wt::Echo::Wait :wat::enum::Pure
     :Immediate []
     :UpTo [d <- :wat::time::NonZeroDuration]
     :Measured [m <- :wat::time::Duration]
     :At [t <- :wat::time::Instant])
   (:wat::core::defrecord :wt::Echo::AskRequest [w <- :wt::Echo::Wait])
   (:wat::core::defenum :wt::Echo::AskResponse :wat::enum::Pure
     :Ok [ns <- :wat::core::i64]
     :RequestTooLarge [bytes <- :wat::core::i64  cap <- :wat::core::i64]
     :RequestMalformed [path <- (:wat::core::Vector :- [:wat::core::String])
                        expected <- :wat::core::String  got <- :wat::core::String])]
  :features
  [(ask [self <- :wt::Echo  req <- :wt::Echo::AskRequest]
     -> :wt::Echo::AskResponse :max-request-bytes 65536)])

(:wat::service::defservice :wt::echo
  :satisfies :wt::Echo
  :durable   [tag <- :wat::core::i64]
  :ephemeral []
  :init (:wat::core::fn [record <- :wt::echo::Record] -> :wt::echo::State
          (:wt::echo::State :durable record))
  :impls
  [(ask [s ctx req]
     (:wat::service::Outcome::Continue s
       (:wat::core::Some
         (:wt::Echo::Reply::Ask
           (:wt::Echo::AskResponse::Ok
             (:wat::core::match (:wt::Echo::AskRequest/w req)
               ((:wt::Echo::Wait::Immediate) 0)
               ((:wt::Echo::Wait::UpTo d) (:wat::time::nanoseconds d))
               ((:wt::Echo::Wait::Measured m) (:wat::time::nanoseconds m))
               ((:wt::Echo::Wait::At t) (:wat::time::epoch-nanos t))))))
       (:wat::core::Vector :- [(:wat::service::Directed :- [:wt::Echo::Reply])])
       (:wat::core::Vector :- [(:wat::service::Alarm :- [:wt::echo::Op])])))])

(:wat::core::defn :wt::ask
  [p <- :wt::Echo  w <- :wt::Echo::Wait] -> :wat::core::String
  (:wat::core::match (:wt::Echo/ask p (:wt::Echo::AskRequest :w w))
    ((:wat::kernel::RecvOutcome::Message r)
      (:wat::core::match r
        ((:wt::Echo::AskResponse::Ok ns) (:wat::core::format "ok:{n}" :n ns))
        ((:wt::Echo::AskResponse::RequestTooLarge b c)
          (:wat::core::format "TOO-LARGE:{b}/{c}" :b b :c c))
        ((:wt::Echo::AskResponse::RequestMalformed _p e g)
          (:wat::core::format "MALFORMED:expected={e};got={g}" :e e :g g))))
    ((:wat::kernel::RecvOutcome::Lost c)
      (:wat::core::format "LOST:{m}" :m (:wat::kernel::LociDiedError/message c)))
    (:wat::kernel::RecvOutcome::Stopped "STOPPED")
    (:wat::kernel::RecvOutcome::Closed "CLOSED")))

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::let
    [h (:wt::echo/start :locus (:wat::spawn::thread)
         :record (:wt::echo::Record :tag 1))
     p (:wat::core::match (:wat::kernel::connect (:wt::echo::Handle/addr h))
         ((:wat::kernel::ConnectOutcome::Connected c) c)
         (_ (:wat::kernel::assertion-failed! "wt: dial failed" :wat::core::None :wat::core::None)))
     a (:wt::ask p (:wt::Echo::Wait::Immediate))
     b (:wt::ask p (:wt::Echo::Wait::UpTo (:wat::time::Milliseconds 250)))
     c (:wt::ask p (:wt::Echo::Wait::Measured
          (:wat::time::- (:wat::time::at 2000000) (:wat::time::at 1000000))))
     d (:wt::ask p (:wt::Echo::Wait::At (:wat::time::at 1000000)))]
    (:wat::kernel::println
      (:wat::core::format "immediate=[{a}];upto=[{b}];duration-CONTROL=[{c}];instant-EXEMPLAR=[{d}];verdict={r}"
        :a a :b b :c c :d d
        :r (:wat::core::if (:wat::core::= b "ok:250000000")
             "NonZeroDuration-CROSSES-THE-WIRE"
             "NonZeroDuration-DOES-NOT-CROSS")))))
