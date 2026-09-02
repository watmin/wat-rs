;; THE NON-VACUITY CONTROL — MUST LOAD.
;;
;; Byte-identical to `probe_arc278_opaque_purity_wall.wat.bad` except for ONE field: the `:durable`
;; slot holds a plain `i64` instead of a live `Lru` handle. Without this control, the sibling's RED
;; would only prove "something in that fixture is bad", not "exactly the opaque field is refused"
;; (R59 `NISI FRANGAS, NIHIL PROBAS`). If THIS file ever stops loading, the gate's RED no longer
;; isolates the opaque and BOTH tests are lying — fix this one first.
;;
;; `:wat::cache::Lru` is a `#[wat_dispatch(scope = "thread_owned")]` handle: a live Rust
;; resource that can cross neither a wire nor a hibernation boundary. `:durable` is the slot
;; whose whole contract is "plain EDN that survives both". A defservice's `:durable` synthesizes
;; `<svc>::Record`, a PURE aggregate, so 293.W's `validate_aggregate_containment` governs it and
;; must refuse this at load with `ImpureFieldInPureAggregate`.
;;
;; It did not, until 2026-08-08: `is_pure_type` knew opaques only through two hand-written lists,
;; and a PARAMETRIC opaque fell through `_ => args.iter().all(is_pure_type)` — the container
;; presumed pure, only its type ARGS checked. This file compiled clean for a month.
;;
;; Its control (`_control.wat`) is byte-identical but for the one field, and MUST load.
(:wat::core::defsurface :probe::opq::Ctr :nature :wat::kernel::Peer
  :messages
  [(:wat::core::defrecord :probe::opq::Ctr::GetRequest [])
   (:wat::core::defenum :probe::opq::Ctr::GetResponse :wat::enum::Pure
     :Ok               [value <- :wat::core::i64]
     :RequestTooLarge  [bytes <- :wat::core::i64  cap <- :wat::core::i64]
     :RequestMalformed [path <- (:wat::core::Vector :- [:wat::core::String])
                        expected <- :wat::core::String
                        got <- :wat::core::String])]
  :features
  [(get [self <- :probe::opq::Ctr  req <- :probe::opq::Ctr::GetRequest]
     -> :probe::opq::Ctr::GetResponse :max-request-bytes 524288)])

;; The same service, with the one offending field replaced by plain EDN.
(:wat::service::defservice :probe::opq::ctr
  :satisfies :probe::opq::Ctr
  :durable   [capacity <- :wat::core::i64]
  :ephemeral []
  :impls
  [(get [s ctx req]
     (:wat::service::Outcome::Continue s (:wat::core::Some (:probe::opq::Ctr::Reply::Get (:probe::opq::Ctr::GetResponse::Ok 1))) (:wat::core::Vector :- [(:wat::service::Directed :- [:probe::opq::Ctr::Reply])]) (:wat::core::Vector :- [(:wat::service::Alarm :- [:probe::opq::ctr::Op])])))])
