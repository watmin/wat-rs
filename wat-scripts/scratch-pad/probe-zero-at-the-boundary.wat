;; probe-zero-at-the-boundary.wat — EXPECTATIONS-time-crosses-the-boundary.md row 2.
;;
;; Stone A's constructor wall is rung 2 for a computed zero: LociDiedError/Panic,
;; which at process locus kills the child. A zero arriving over the wire is the
;; computed case by definition. This probe sends UpTo with Integer 0 as a foreign
;; EDN frame (the wat constructor cannot mint NonZeroDuration 0), then a valid
;; call on the SAME connection. RequestMalformed + ok: means the service lived.
;; LOST/CLOSED on the second call is STOP-3 — the panic wearing a different hat.

(:wat::config::set-redef! true)

(:wat::core::defsurface :zb::Echo :nature :wat::kernel::Peer
  :messages
  [(:wat::core::defenum :zb::Echo::Wait :wat::enum::Pure
     :Immediate []
     :UpTo [d <- :wat::time::NonZeroDuration])
   (:wat::core::defrecord :zb::Echo::AskRequest [w <- :zb::Echo::Wait])
   (:wat::core::defenum :zb::Echo::AskResponse :wat::enum::Pure
     :Ok [ns <- :wat::core::i64]
     :RequestTooLarge [bytes <- :wat::core::i64  cap <- :wat::core::i64]
     :RequestMalformed [path <- (:wat::core::Vector :- [:wat::core::String])
                        expected <- :wat::core::String  got <- :wat::core::String])]
  :features
  [(ask [self <- :zb::Echo  req <- :zb::Echo::AskRequest]
     -> :zb::Echo::AskResponse :max-request-bytes 65536)])

(:wat::service::defservice :zb::echo
  :satisfies :zb::Echo
  :durable   [tag <- :wat::core::i64]
  :ephemeral []
  :init (:wat::core::fn [record <- :zb::echo::Record] -> :zb::echo::State
          (:zb::echo::State :durable record))
  :impls
  [(ask [s ctx req]
     (:wat::service::Outcome::Continue s
       (:wat::core::Some
         (:zb::Echo::Reply::Ask
           (:zb::Echo::AskResponse::Ok
             (:wat::core::match (:zb::Echo::AskRequest/w req)
               ((:zb::Echo::Wait::Immediate) 0)
               ((:zb::Echo::Wait::UpTo d) (:wat::time::nanoseconds d))))))
       (:wat::core::Vector :- [(:wat::service::Directed :- [:zb::Echo::Reply])])
       (:wat::core::Vector :- [(:wat::service::Alarm :- [:zb::echo::Op])])))])

(:wat::core::defn :zb::ask
  [p <- :zb::Echo  req <- :zb::Echo::AskRequest] -> :wat::core::String
  (:wat::core::match (:zb::Echo/ask p req)
    ((:wat::kernel::RecvOutcome::Message r)
      (:wat::core::match r
        ((:zb::Echo::AskResponse::Ok ns) (:wat::core::format "ok:{n}" :n ns))
        ((:zb::Echo::AskResponse::RequestTooLarge b c)
          (:wat::core::format "TOO-LARGE:{b}/{c}" :b b :c c))
        ((:zb::Echo::AskResponse::RequestMalformed _p e g)
          (:wat::core::format "MALFORMED:expected={e};got={g}" :e e :g g))))
    ((:wat::kernel::RecvOutcome::Lost c)
      (:wat::core::format "LOST:{m}" :m (:wat::kernel::LociDiedError/message c)))
    (:wat::kernel::RecvOutcome::Stopped "STOPPED")
    (:wat::kernel::RecvOutcome::Closed "CLOSED")))

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::let
    [h (:zb::echo/start :locus (:wat::spawn::process)
         :record (:zb::echo::Record :tag 1))
     p (:wat::core::match (:wat::kernel::connect (:zb::echo::Handle/addr h))
         ((:wat::kernel::ConnectOutcome::Connected c) c)
         (_ (:wat::kernel::assertion-failed! "zb: dial failed" :wat::core::None :wat::core::None)))
     zero (:wat::edn::read "#zb.Echo/AskRequest {:w #zb.Echo.Wait/UpTo [0]}")
     r1 (:zb::ask p zero)
     r2 (:zb::ask p (:zb::Echo::AskRequest :w (:zb::Echo::Wait::UpTo (:wat::time::Millisecond 250))))]
    (:wat::kernel::println
      (:wat::core::format "zero=[{a}];then=[{b}]"
        :a r1
        :b r2))))
