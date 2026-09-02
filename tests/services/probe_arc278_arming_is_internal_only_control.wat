;; THE NON-VACUITY CONTROL — MUST LOAD.
;;
;; IDENTICAL to `probe_arc278_arming_is_internal_only.wat.bad` but for ONE expression: `start`
;; arms the INTERNAL `-tick` op instead of the PUBLIC `bump` op. Without this control, the
;; sibling's RED would only prove "something in that fixture is bad", not "exactly the
;; public-op arm is refused" (R59 `NISI FRANGAS, NIHIL PROBAS`). If THIS file ever stops
;; loading, the gate's RED no longer isolates the public-op arm and BOTH tests are lying —
;; fix this one first.
;;
;; An alarm has no client — when the timer fires, the handler runs with a TIMER in the `idx`
;; slot, not a client. `-tick`'s `<service>::Op` variant name begins with `-`
;; (wat/service.wat:876-892): it is declared to have no client, so arming it is exactly the
;; legitimate self-scheduling case this rule must NOT refuse (mirrors
;; `tests/services/probe_arc278_self_scheduling.wat`, the non-vacuity control for the whole
;; strike per BRIEF-arming-is-internal-only.md).
(:wat::core::defsurface :probe::Tick2 :nature :wat::kernel::Peer
  :messages
  [(:wat::core::defrecord :probe::Tick2::StartRequest [])
   (:wat::core::defenum :probe::Tick2::StartResponse :wat::enum::Pure
     :Ok               []
     :RequestTooLarge  [bytes <- :wat::core::i64  cap <- :wat::core::i64]
     :RequestMalformed [path <- (:wat::core::Vector :- [:wat::core::String])  expected <- :wat::core::String  got <- :wat::core::String])
   (:wat::core::defrecord :probe::Tick2::BumpRequest [])
   (:wat::core::defenum :probe::Tick2::BumpResponse :wat::enum::Pure
     :Ok               []
     :RequestTooLarge  [bytes <- :wat::core::i64  cap <- :wat::core::i64]
     :RequestMalformed [path <- (:wat::core::Vector :- [:wat::core::String])  expected <- :wat::core::String  got <- :wat::core::String])]
  :features
  [(start [self <- :probe::Tick2  req <- :probe::Tick2::StartRequest] -> :probe::Tick2::StartResponse
     :max-request-bytes 524288)
   (bump  [self <- :probe::Tick2  req <- :probe::Tick2::BumpRequest]  -> :probe::Tick2::BumpResponse
     :max-request-bytes 524288)])

(:wat::service::defservice :probe::tick2
  :satisfies :probe::Tick2
  :durable   [count <- :wat::core::i64]
  :ephemeral []
  :impls
  [;; The one difference from the sibling `.wat.bad`: arms `-tick` (INTERNAL), not `bump`.
   (start [s ctx req]
     (:wat::service::Outcome::Continue s (:wat::core::Some (:probe::Tick2::Reply::Start (:probe::Tick2::StartResponse::Ok)))
       (:wat::core::Vector :- [(:wat::service::Directed :- [:probe::Tick2::Reply])]) [(:wat::service::Alarm :after (:wat::time::Millisecond 5)
          :op (:probe::tick2::Op::-Tick))]))

   (bump [s ctx req]
     (:wat::service::Outcome::Continue s
       (:wat::core::Some (:probe::Tick2::Reply::Bump (:probe::Tick2::BumpResponse::Ok))) (:wat::core::Vector :- [(:wat::service::Directed :- [:probe::Tick2::Reply])]) (:wat::core::Vector :- [(:wat::service::Alarm :- [:probe::tick2::Op])])))

   (-tick [s ctx]
     (:wat::service::SelfOutcome::Continue s (:wat::core::Vector :- [(:wat::service::Directed :- [:probe::Tick2::Reply])]) (:wat::core::Vector :- [(:wat::service::Alarm :- [:probe::tick2::Op])])))])
