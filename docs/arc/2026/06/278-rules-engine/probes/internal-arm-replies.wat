;; internal-arm-replies.wat — the red probe for the internal-arm wall. MUST NOT type-check.
;;
;; HISTORY. Written 2026-09-01 to ask whether an internal arm (`[s ctx]`, no caller) could
;; construct a reply. It could: the check-time rejection was a `foldl`/`Directed` type collision
;; with a one-token bypass, and the real guard lived in the serve loop at run time, killing the
;; service. That measurement is what justified `SelfOutcome`.
;;
;; AFTER the outcome-composes stone the wall is a TYPE, and this file is its acceptance criterion:
;;
;;   :wat::core::match: parameter scrutinee expects (:wat::service::SelfOutcome :- [...]);
;;                                             got (:wat::service::Outcome :- [...])
;;
;; ⚠ DO NOT spell the offending arm `(:wat::service::Outcome::Reply …)`. That variant no longer
;; exists, and a nonexistent variant of a STDLIB enum is NOT resolve-checked — it passes `--check`
;; and dies at run time as `UnknownFunction`, which makes this file look ACCEPTING when it is not.
;; Measured, one variable (stdlib vs same-file):
;;     ACCEPTED at --check:  :wat::core::Option::Nope / :wat::kernel::RecvOutcome::Bogus
;;     rejected at --check:  :probe::Local::Nope
;; That gap is a separate finding and wants its own stone. Until it closes, the offending arm must
;; be spelled with a variant that DOES exist — `Outcome::Continue` — so the rejection is the type
;; wall and not a resolution accident.
;;
;; ⚠ WHY IT LIVES HERE AND NOT IN wat-scripts/. `every_wat_scripts_file_loads` type-checks every
;; .wat under wat-scripts/; a file whose whole job is to be rejected cannot be gated on acceptance.
;; No rune: a rune would silence the wall's only proof that it fires.

(:wat::core::defsurface :probe::Solo :nature :wat::kernel::Peer
  :messages
  [(:wat::core::defrecord :probe::Solo::PingRequest [])
   (:wat::core::defenum :probe::Solo::PingResponse :wat::enum::Pure
     :Ok               []
     :RequestTooLarge  [bytes <- :wat::core::i64  cap <- :wat::core::i64]
     :RequestMalformed [path <- (:wat::core::Vector :- [:wat::core::String])
                        expected <- :wat::core::String  got <- :wat::core::String])]
  :features
  [(ping [self <- :probe::Solo  req <- :probe::Solo::PingRequest]
     -> :probe::Solo::PingResponse :max-request-bytes 524288)])

;; ✅ CONTROL — an internal arm returning a SelfOutcome. Must keep compiling.
(:wat::service::defservice :probe::quiet
  :satisfies :probe::Solo
  :durable   []
  :ephemeral []
  :impls
  [(ping  [s ctx req] (:wat::service::Outcome::Reply s (:probe::Solo::PingResponse::Ok)))
   (-tick [s ctx]
     (:wat::service::SelfOutcome::Continue s
       (:wat::core::Vector :- [(:wat::service::Directed :- [:probe::Solo::Reply])])
       (:wat::core::Vector :- [(:wat::service::Alarm :- [:probe::quiet::Op])])))])

;; ⛔ MUST BE REJECTED — the internal arm returns a public Outcome carrying a reply.
;; The error must name SelfOutcome against Outcome.
(:wat::service::defservice :probe::loud
  :satisfies :probe::Solo
  :durable   []
  :ephemeral []
  :impls
  [(ping  [s ctx req] (:wat::service::Outcome::Reply s (:probe::Solo::PingResponse::Ok)))
   (-tick [s ctx]
     (:wat::service::Outcome::Continue s
       (:wat::core::Some (:probe::Solo::Reply::Ping (:probe::Solo::PingResponse::Ok)))
       (:wat::core::Vector :- [(:wat::service::Directed :- [:probe::Solo::Reply])])
       (:wat::core::Vector :- [(:wat::service::Alarm :- [:probe::loud::Op])])))])

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::kernel::println "this file must never freeze"))
