;; Arc 278 S4c migration: :ops RETIRED — the service wears a surface (:satisfies + :impls).
;; NEGATIVE (subject preserved): a bare type keyword in :durable is UNEXPRESSIBLE — :durable
;; takes a field vector. Everything else here is a VALID :satisfies service, so the sole
;; defect (and the sole reason for rejection) is the bare `:durable :wat::core::i64`.
(:wat::core::defsurface :my::Counter :nature :wat::kernel::Peer'
  :messages
  [(:wat::core::defrecord :my::Counter::GetRequest  [])
   (:wat::core::defrecord :my::Counter::GetResponse [value <- :wat::core::i64])]
  :features
  [(get [self <- :my::Counter  req <- :my::Counter::GetRequest] -> :my::Counter::GetResponse)])

(:wat::service::defservice :my::counter
  :satisfies :my::Counter
  :durable :wat::core::i64          ;; ← the DEFECT under test: bare type keyword, not a field vector
  :ephemeral []
  :impls
  [(get [s req]
     (:wat::service::Outcome::Reply s (:my::Counter::GetResponse 0)))])
