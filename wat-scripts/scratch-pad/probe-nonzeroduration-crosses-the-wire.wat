;; probe-nonzeroduration-crosses-the-wire.wat — THE DISCONFIRMING PROBE FOR STONE B.
;;
;; Stone B replaces the queue's `wait-ns <- :wat::core::i64` with
;;   (defenum :queue::Queue::Wait :Immediate [] :UpTo [d <- :wat::time::NonZeroDuration])
;; inside `(defsurface :queue::Queue :nature :wat::kernel::Peer)`. That is a WIRE PROTOCOL
;; record, so the whole design rests on one assumption nothing in the tree has tested:
;;
;;   ★ CAN A defsurface MESSAGE CARRY A :wat::time::NonZeroDuration ACROSS THE WIRE?
;;
;; Every surface payload in sqs.wat today is i64 / String / Vector. Stone A minted
;; NonZeroDuration as a new Value variant and touched src/edn/render.rs and
;; src/value/observe.rs, but wire ADMISSIBILITY for a surface payload is a different axis
;; from rendering. If the answer is no, `:UpTo [NonZeroDuration]` is unbuildable and the
;; arm must carry a raw i64 -- which reopens zero and defeats the stone.
;;
;; Nine of the ten lines below are the ordinary shape; the one that matters is the
;; `d <- :wat::time::NonZeroDuration` field in the enum arm. If this file freezes and the
;; round-trip returns the nanos, the gap is closed and Stone B may be drawn on it.
;;
;;   expect: immediate=ok;upto-ns=250000000;roundtrip=yes

(:wat::config::set-redef! true)

(:wat::core::defsurface :wp::Echo :nature :wat::kernel::Peer
  :messages
  [(:wat::core::defenum :wp::Echo::Wait :wat::enum::Pure
     :Immediate []
     :UpTo [d <- :wat::time::NonZeroDuration]
     ;; THE CONTROL: does the OLD time type cross? If Duration also fails, this is a
     ;; pre-existing limit on every time type, not something Stone A left half-done.
     :Measured [m <- :wat::time::Duration])
   (:wat::core::defrecord :wp::Echo::AskRequest [w <- :wp::Echo::Wait])
   (:wat::core::defenum :wp::Echo::AskResponse :wat::enum::Pure
     :Ok [ns <- :wat::core::i64]
     :RequestTooLarge [bytes <- :wat::core::i64  cap <- :wat::core::i64]
     :RequestMalformed [path <- (:wat::core::Vector :- [:wat::core::String])
                        expected <- :wat::core::String  got <- :wat::core::String])]
  :features
  [(ask [self <- :wp::Echo  req <- :wp::Echo::AskRequest]
     -> :wp::Echo::AskResponse :max-request-bytes 65536)])

(:wat::service::defservice :wp::echo
  :satisfies :wp::Echo
  :durable   [tag <- :wat::core::i64]
  :ephemeral []
  :init (:wat::core::fn [record <- :wp::echo::Record] -> :wp::echo::State
          (:wp::echo::State :durable record))
  :impls
  [(ask [s ctx req]
     (:wat::service::Outcome::Continue s
       (:wat::core::Some
         (:wp::Echo::Reply::Ask
           (:wp::Echo::AskResponse::Ok
             (:wat::core::match (:wp::Echo::AskRequest/w req)
               ((:wp::Echo::Wait::Immediate) 0)
               ((:wp::Echo::Wait::UpTo d) (:wat::time::nanoseconds d))
               ((:wp::Echo::Wait::Measured m) (:wat::time::nanoseconds m))))))
       (:wat::core::Vector :- [(:wat::service::Directed :- [:wp::Echo::Reply])])
       (:wat::core::Vector :- [(:wat::service::Alarm :- [:wp::echo::Op])])))])

;; ★ The fallback DISCRIMINATES. A probe that collapses every failure into one sentinel
;; cannot tell "the wire refused the payload" from "the peer died" — the exact defect
;; intueri found in :demo::q-depth's (Tuple 1 1). Name the outcome or measure nothing.
(:wat::core::defn :wp::ask
  [p <- :wp::Echo  w <- :wp::Echo::Wait] -> :wat::core::String
  (:wat::core::match (:wp::Echo/ask p (:wp::Echo::AskRequest :w w))
    ((:wat::kernel::RecvOutcome::Message r)
      (:wat::core::match r
        ((:wp::Echo::AskResponse::Ok ns) (:wat::core::format "ok:{n}" :n ns))
        ((:wp::Echo::AskResponse::RequestTooLarge b c)
          (:wat::core::format "TOO-LARGE:{b}/{c}" :b b :c c))
        ((:wp::Echo::AskResponse::RequestMalformed _p e g)
          (:wat::core::format "MALFORMED:expected={e};got={g}" :e e :g g))))
    ((:wat::kernel::RecvOutcome::Lost c)
      (:wat::core::format "LOST:{m}" :m (:wat::kernel::LociDiedError/message c)))
    (:wat::kernel::RecvOutcome::Stopped "STOPPED")
    (:wat::kernel::RecvOutcome::Closed "CLOSED")))

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::let
    [h (:wp::echo/start :locus (:wat::spawn::process)
         :record (:wp::echo::Record :tag 1))
     p (:wat::core::match (:wat::kernel::connect (:wp::echo::Handle/addr h))
         ((:wat::kernel::ConnectOutcome::Connected c) c)
         (_ (:wat::kernel::assertion-failed! "wp: dial failed" :wat::core::None :wat::core::None)))
     a (:wp::ask p (:wp::Echo::Wait::Immediate))
     b (:wp::ask p (:wp::Echo::Wait::UpTo (:wat::time::Millisecond 250)))
     c (:wp::ask p (:wp::Echo::Wait::Measured
          (:wat::time::- (:wat::time::at 2000000) (:wat::time::at 1000000))))]
    (:wat::kernel::println
      (:wat::core::format "immediate=[{a}];upto=[{b}];measured-CONTROL=[{c}];verdict={r}"
        :a a
        :b b
        :c c
        :r (:wat::core::if (:wat::core::= b "ok:250000000")
             "NonZeroDuration-CROSSES-THE-WIRE"
             "NonZeroDuration-DOES-NOT-CROSS")))))
