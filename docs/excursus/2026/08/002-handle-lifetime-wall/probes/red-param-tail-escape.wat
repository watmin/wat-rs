;; red-param-tail-escape.wat — the DELIBERATELY-RED target for excursus 002 stone 3.
;;
;; This file MUST NOT type-check. It is the wall's acceptance criterion: a function that takes a
;; Handle as a parameter and tail-escapes a Peer of that service. The creating scope and the
;; escaping scope are different functions — stones 1 and 2 miss this because both treat a Handle
;; param as safe (that is what makes conn(h) legal).
;;
;; `tests/services/probe_ex002_param_ownership.rs` drives it and asserts the rejection structurally
;; — that the wall names `:red::drive-param`, and that it does NOT name `:red::conn` (upward: a
;; param is a BORROW, the caller owns the handle) or `:red::held-param` (drive sits in a binding,
;; so this frame outlives the call).
;;
;; ⚠ WHY IT LIVES HERE AND NOT IN wat-scripts/.
;; `every_wat_scripts_file_loads` type-checks EVERY .wat under wat-scripts/, so a must-be-rejected
;; file there turns the floor red for as long as the wall works. Deliberately-red probes live under
;; a `probes/` directory in their own arc/excursus.
;;
;; ⚠ A `rune:check` ON THIS FILE IS AN AUTOMATIC FAIL. Prose about runes is not a rune.
;; Rune the instrument (`:sched::drive-param` in probe-self-sched-bisect.wat); never the
;; acceptance criterion.

(:wat::core::defsurface :red::Alpha :nature :wat::kernel::Peer
  :messages
  [(:wat::core::defrecord :red::Alpha::PingRequest [])
   (:wat::core::defenum :red::Alpha::PingResponse :wat::enum::Pure
     :Pong            []
     :RequestTooLarge [bytes <- :wat::core::i64  cap <- :wat::core::i64]
     :RequestMalformed [path <- (:wat::core::Vector :- [:wat::core::String])  expected <- :wat::core::String  got <- :wat::core::String])]
  :features
  [(ping [self <- :red::Alpha  req <- :red::Alpha::PingRequest] -> :red::Alpha::PingResponse
     :max-request-bytes 524288)])

(:wat::service::defservice :red::alpha
  :satisfies :red::Alpha
  :durable   [n <- :wat::core::i64]
  :ephemeral []
  :init (:wat::core::fn [record <- :red::alpha::Record] -> :red::alpha::State
          (:red::alpha::State :durable record))
  :impls
  [(ping [s ctx req] (:wat::service::Outcome::Continue s (:wat::core::Some (:red::Alpha::Reply::Ping (:red::Alpha::PingResponse::Pong))) (:wat::core::Vector :- [(:wat::service::Directed :- [:red::Alpha::Reply])]) (:wat::core::Vector :- [(:wat::service::Alarm :- [:red::alpha::Op])])))])

;; ✅ MUST KEEP COMPILING — upward. Handle arrives as a PARAM, RETURNS a Peer. The CALLER owns
;; the handle and outlives the call, so a param is a BORROW here. If the wall names this, the
;; widening leaked across directions and every conn helper in the corpus dies (STOP-1).
(:wat::core::defn :red::conn
  [h <- :red::alpha::Handle] -> (:wat::kernel::Peer :- [:red::Alpha::Op :red::Alpha::Reply])
  (:wat::core::match (:wat::kernel::connect (:red::alpha::Handle/addr h))
    ((:wat::kernel::ConnectOutcome::Connected p) p)
    ((:wat::kernel::ConnectOutcome::Refused c) (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message c) :wat::core::None :wat::core::None))
    ((:wat::kernel::ConnectOutcome::Rejected c) (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message c) :wat::core::None :wat::core::None))
    ((:wat::kernel::ConnectOutcome::Failed c) (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message c) :wat::core::None :wat::core::None))))

(:wat::core::defn :red::consume-peer
  [c <- (:wat::kernel::Peer :- [:red::Alpha::Op :red::Alpha::Reply])] -> :wat::core::i64
  (:wat::core::match (:red::Alpha/ping c (:red::Alpha::PingRequest))
    ((:wat::kernel::RecvOutcome::Message __r) 1)
    ((:wat::kernel::RecvOutcome::Lost __c) -1)
    (:wat::kernel::RecvOutcome::Stopped -2)
    (:wat::kernel::RecvOutcome::Closed -3)))

;; ✅ MUST KEEP COMPILING — handle as a param, drive in a BINDING. This frame outlives the call.
;; If the wall names this, condition 2 (the let itself in tail with a user-fn tail expr) was
;; ignored and the conservative trade rejects programs that are safe.
(:wat::core::defn :red::held-param
  [h <- :red::alpha::Handle] -> :wat::core::i64
  (:wat::core::let
    [c (:red::conn h)
     n (:red::consume-peer c)]
    n))

;; ⛔ MUST BE REJECTED — handle as a PARAM, tail-escapes a peer. This frame dies before
;; `:red::consume-peer` runs, taking `h` with it. The caller that passed a temporary
;; `(:red::alpha/start …)` is left with a live channel to nothing.
(:wat::core::defn :red::drive-param
  [h <- :red::alpha::Handle] -> :wat::core::i64
  (:wat::core::let
    [c (:red::conn h)]
    (:red::consume-peer c)))

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::kernel::println "this file must never freeze"))
