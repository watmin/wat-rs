;; Arc 278 S4c migration: :ops RETIRED — the service wears a surface (:satisfies + :impls).
;; NEGATIVE (subject preserved): a bogus trailing clause must be rejected directly (named),
;; not silently mis-read. Everything else here is a VALID :satisfies service, so the sole
;; defect (and the sole reason for rejection) is the unrecognized `:bogus-option` clause.
(:wat::core::defsurface :my::Counter :nature :wat::kernel::Peer
  :messages
  [(:wat::core::defrecord :my::Counter::GetRequest  [])
   (:wat::core::defenum :my::Counter::GetResponse :wat::enum::Pure
     :Ok              [value <- wat.type/i64]
     :RequestTooLarge [bytes <- wat.type/i64  cap <- wat.type/i64]
     :RequestMalformed [path <- (wat.type/Vector :- [wat.type/String])  expected <- wat.type/String  got <- wat.type/String])]
  :features
  [(get [self <- :my::Counter  req <- :my::Counter::GetRequest] -> :my::Counter::GetResponse :max-request-bytes 524288)])

(:wat::service::defservice :my::counter
  :satisfies :my::Counter
  :durable [count <- wat.type/i64]
  :ephemeral []
  :impls
  [(get [s ctx req]
     (:wat::service::Outcome.Reply {:state s :reply (:my::Counter::GetResponse.Ok {:value (:my::counter::Record/count (:my::counter::State/durable s))})}))]
  :bogus-option :wat::core::Record)   ;; ← the DEFECT under test: an unrecognized trailing clause
