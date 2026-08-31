;; red-creation-escape.wat — the DELIBERATELY-RED target for excursus 002 stone 1.
;;
;; This file MUST NOT type-check. It is the wall's acceptance criterion: a peer escaping the scope
;; that created its service's Handle. `tests/services/probe_ex002_creation_escape.rs` drives it and
;; asserts the rejection structurally — that the wall names `:red::dial-and-drop`, and that it does
;; NOT name `:red::conn`, which is the ordinary safe helper sitting beside it.
;;
;; ⚠ WHY IT LIVES HERE AND NOT IN wat-scripts/scratch-pad/.
;; `every_wat_scripts_file_loads` type-checks EVERY .wat under wat-scripts/, so a must-be-rejected
;; file there turns the floor red for as long as the wall works. That is a contradiction built into
;; the file's location, not a fault in the wall. The repo already has the home for this:
;; `docs/arc/2026/06/278-rules-engine/probes/red-*.wat`. Deliberately-red probes live under a
;; `probes/` directory in their own arc/excursus, and nothing gates them.
;;
;; The alternative — a `rune:` on the escape here — would be WRONG, and the executor was right to
;; refuse it: a rune would silence the wall's only proof that it fires at all, producing a green
;; floor from a wall that catches nothing. The rune belongs on
;; `tests/services/probe_severed_reaches_the_client.wat`, whose gate must construct an ownerless
;; service deliberately; it does not belong on the acceptance criterion itself.
;;
;; The GREEN half of the feasibility work — that the checker can derive a service's surface types
;; from a Handle — stays at `wat-scripts/scratch-pad/probe-handle-to-surface-relation.wat`, where
;; the loader gate keeps proving it still holds.

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
  [(ping [s ctx req] (:wat::service::Outcome::Reply s (:red::Alpha::PingResponse::Pong)))])

;; ✅ MUST KEEP COMPILING — the ordinary `conn` helper. The handle arrives as a PARAM, so the
;; CALLER owns it and the peer cannot outlive it. If the wall names this, the rule was keyed on the
;; parameter rather than on creation, and it would reject every conn helper in the corpus.
(:wat::core::defn :red::conn
  [h <- :red::alpha::Handle] -> (:wat::kernel::Peer :- [:red::Alpha::Op :red::Alpha::Reply])
  (:wat::core::match (:wat::kernel::connect (:red::alpha::Handle/addr h))
    ((:wat::kernel::ConnectOutcome::Connected p) p)
    ((:wat::kernel::ConnectOutcome::Refused c) (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message c) :wat::core::None :wat::core::None))
    ((:wat::kernel::ConnectOutcome::Rejected c) (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message c) :wat::core::None :wat::core::None))
    ((:wat::kernel::ConnectOutcome::Failed c) (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message c) :wat::core::None :wat::core::None))))

;; ⛔ MUST BE REJECTED — `h` is CREATED here by `/start`, so this scope owns it. Returning the peer
;; hands the caller a live channel to a service that dies the moment this function returns.
(:wat::core::defn :red::dial-and-drop
  [] -> (:wat::kernel::Peer :- [:red::Alpha::Op :red::Alpha::Reply])
  (:wat::core::let
    [h (:red::alpha/start :locus (:wat::spawn::thread) :record (:red::alpha::Record :n 0))
     c (:red::conn h)]
    c))

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::kernel::println "this file must never freeze"))
