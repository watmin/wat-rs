;; Arc 278 S4c migration: :ops RETIRED — the service wears a surface (:satisfies + :impls).
;; NEGATIVE (subject preserved): a bare type keyword in :durable is UNEXPRESSIBLE — :durable
;; takes a field vector. Everything else here is a VALID :satisfies service, so the sole
;; defect (and the sole reason for rejection) is the bare `:durable :wat::core::i64`.
(:wat::core::defsurface :my::Counter :nature :wat::kernel::Peer
  :messages
  [(:wat::core::defrecord :my::Counter::GetRequest  [])
   (:wat::core::defenum :my::Counter::GetResponse :wat::enum::Pure
     :Ok              [value <- :wat::core::i64]
     :RequestTooLarge [bytes <- :wat::core::i64  cap <- :wat::core::i64]
     :RequestMalformed [path <- (:wat::core::Vector :- [:wat::core::String])  expected <- :wat::core::String  got <- :wat::core::String])]
  :features
  [(get [self <- :my::Counter  req <- :my::Counter::GetRequest] -> :my::Counter::GetResponse :max-request-bytes 524288)])

(:wat::service::defservice :my::counter
  :satisfies :my::Counter
  :durable :wat::core::i64          ;; ← the DEFECT under test: bare type keyword, not a field vector
  :ephemeral []
  :impls
  [(get [s ctx req]
     (:wat::service::Outcome::Reply s (:my::Counter::GetResponse::Ok 0)))])
