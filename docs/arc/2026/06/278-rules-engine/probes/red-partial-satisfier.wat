;; red-partial-satisfier.wat — the DELIBERATELY-RED target for the :impls completeness guard.
;;
;; This file MUST NOT type-check. It is the wall's acceptance criterion: a defservice that
;; :satisfies a surface and implements only part of it. `tests/services/probe_impls_completeness.rs`
;; drives it and asserts the rejection structurally — that the wall names `:probe::partial`,
;; the surface `:probe::Trio`, and EVERY missing op (`pong` AND `pang`), and that it does
;; NOT name `:probe::complete` or `:probe::ticking`.
;;
;; ⚠ WHY IT LIVES HERE AND NOT IN wat-scripts/.
;; `every_wat_scripts_file_loads` type-checks EVERY .wat under wat-scripts/, so a must-be-rejected
;; file there turns the floor red for as long as the wall works. Deliberately-red probes live
;; under a `probes/` directory in their own arc, and nothing gates them.
;;
;; No rune on this file. A rune would silence the wall's only proof that it fires at all,
;; producing a green floor from a guard that catches nothing. That was excursus 002 stone 1's
;; specification error and it is not being repeated.
;;
;; Three shapes in one file:
;;   :probe::partial  — two of three features. MUST BE REJECTED, naming both missing ops.
;;   :probe::complete — all three features. MUST KEEP COMPILING.
;;   :probe::ticking  — all three features PLUS an internal `-tick`. MUST KEEP COMPILING.
;;                      This is what a symmetric `impls == features` rule would break.

(:wat::core::defsurface :probe::Trio :nature :wat::kernel::Peer
  :messages
  [(:wat::core::defrecord :probe::Trio::PingRequest [])
   (:wat::core::defenum :probe::Trio::PingResponse :wat::enum::Pure
     :Ok              []
     :RequestTooLarge [bytes <- :wat::core::i64  cap <- :wat::core::i64]
     :RequestMalformed [path <- (:wat::core::Vector :- [:wat::core::String])  expected <- :wat::core::String  got <- :wat::core::String])
   (:wat::core::defrecord :probe::Trio::PongRequest [])
   (:wat::core::defenum :probe::Trio::PongResponse :wat::enum::Pure
     :Ok              []
     :RequestTooLarge [bytes <- :wat::core::i64  cap <- :wat::core::i64]
     :RequestMalformed [path <- (:wat::core::Vector :- [:wat::core::String])  expected <- :wat::core::String  got <- :wat::core::String])
   (:wat::core::defrecord :probe::Trio::PangRequest [])
   (:wat::core::defenum :probe::Trio::PangResponse :wat::enum::Pure
     :Ok              []
     :RequestTooLarge [bytes <- :wat::core::i64  cap <- :wat::core::i64]
     :RequestMalformed [path <- (:wat::core::Vector :- [:wat::core::String])  expected <- :wat::core::String  got <- :wat::core::String])]
  :features
  [(ping [self <- :probe::Trio  req <- :probe::Trio::PingRequest]
     -> :probe::Trio::PingResponse :max-request-bytes 524288)
   (pong [self <- :probe::Trio  req <- :probe::Trio::PongRequest]
     -> :probe::Trio::PongResponse :max-request-bytes 524288)
   (pang [self <- :probe::Trio  req <- :probe::Trio::PangRequest]
     -> :probe::Trio::PangResponse :max-request-bytes 524288)])

;; ⛔ MUST BE REJECTED — :satisfies Trio, implements ping only. Missing pong AND pang;
;; the error must name both, in one message.
(:wat::service::defservice :probe::partial
  :satisfies :probe::Trio
  :durable   []
  :ephemeral []
  :impls
  [(ping [s ctx req] (:wat::service::Outcome::Continue s (:wat::core::Some (:probe::Trio::Reply::Ping (:probe::Trio::PingResponse::Ok))) (:wat::core::Vector :- [(:wat::service::Directed :- [:probe::Trio::Reply])]) (:wat::core::Vector :- [(:wat::service::Alarm :- [:probe::partial::Op])])))])

;; ✅ MUST KEEP COMPILING — every feature has an arm.
(:wat::service::defservice :probe::complete
  :satisfies :probe::Trio
  :durable   []
  :ephemeral []
  :impls
  [(ping [s ctx req] (:wat::service::Outcome::Continue s (:wat::core::Some (:probe::Trio::Reply::Ping (:probe::Trio::PingResponse::Ok))) (:wat::core::Vector :- [(:wat::service::Directed :- [:probe::Trio::Reply])]) (:wat::core::Vector :- [(:wat::service::Alarm :- [:probe::complete::Op])])))
   (pong [s ctx req] (:wat::service::Outcome::Continue s (:wat::core::Some (:probe::Trio::Reply::Pong (:probe::Trio::PongResponse::Ok))) (:wat::core::Vector :- [(:wat::service::Directed :- [:probe::Trio::Reply])]) (:wat::core::Vector :- [(:wat::service::Alarm :- [:probe::complete::Op])])))
   (pang [s ctx req] (:wat::service::Outcome::Continue s (:wat::core::Some (:probe::Trio::Reply::Pang (:probe::Trio::PangResponse::Ok))) (:wat::core::Vector :- [(:wat::service::Directed :- [:probe::Trio::Reply])]) (:wat::core::Vector :- [(:wat::service::Alarm :- [:probe::complete::Op])])))])

;; ✅ MUST KEEP COMPILING — every feature has an arm, PLUS an internal `-tick`.
;; A symmetric `impls == features` rule rejects this; `features ⊆ impls` must not.
(:wat::service::defservice :probe::ticking
  :satisfies :probe::Trio
  :durable   []
  :ephemeral []
  :impls
  [(ping [s ctx req] (:wat::service::Outcome::Continue s (:wat::core::Some (:probe::Trio::Reply::Ping (:probe::Trio::PingResponse::Ok))) (:wat::core::Vector :- [(:wat::service::Directed :- [:probe::Trio::Reply])]) (:wat::core::Vector :- [(:wat::service::Alarm :- [:probe::ticking::Op])])))
   (pong [s ctx req] (:wat::service::Outcome::Continue s (:wat::core::Some (:probe::Trio::Reply::Pong (:probe::Trio::PongResponse::Ok))) (:wat::core::Vector :- [(:wat::service::Directed :- [:probe::Trio::Reply])]) (:wat::core::Vector :- [(:wat::service::Alarm :- [:probe::ticking::Op])])))
   (pang [s ctx req] (:wat::service::Outcome::Continue s (:wat::core::Some (:probe::Trio::Reply::Pang (:probe::Trio::PangResponse::Ok))) (:wat::core::Vector :- [(:wat::service::Directed :- [:probe::Trio::Reply])]) (:wat::core::Vector :- [(:wat::service::Alarm :- [:probe::ticking::Op])])))
   (-tick [s ctx] (:wat::service::SelfOutcome::Continue s (:wat::core::Vector :- [(:wat::service::Directed :- [:probe::Trio::Reply])]) (:wat::core::Vector :- [(:wat::service::Alarm :- [:probe::ticking::Op])])))])

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::kernel::println "this file must never freeze"))
