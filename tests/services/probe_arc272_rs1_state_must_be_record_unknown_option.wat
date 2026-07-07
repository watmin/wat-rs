;; Arc 278 S4c migration: :ops RETIRED — the service wears a surface (:satisfies + :impls).
;; NEGATIVE (subject preserved): a bogus trailing clause must be rejected directly (named),
;; not silently mis-read. Everything else here is a VALID :satisfies service, so the sole
;; defect (and the sole reason for rejection) is the unrecognized `:bogus-option` clause.
(:wat::core::defsurface :my::Counter :nature :wat::kernel::Peer'
  :messages
  [(:wat::core::defrecord :my::Counter::GetRequest  [])
   (:wat::core::defrecord :my::Counter::GetResponse [value <- :wat::core::i64])]
  :features
  [(get [self <- :my::Counter  req <- :my::Counter::GetRequest] -> :my::Counter::GetResponse)])

(:wat::service::defservice :my::counter
  :satisfies :my::Counter
  :durable [count <- :wat::core::i64]
  :ephemeral []
  :impls
  [(get [s req]
     (:wat::service::Outcome::Reply s (:my::Counter::GetResponse (:my::counter::Record/count (:my::counter::State/durable s)))))]
  :bogus-option :wat::core::Record)   ;; ← the DEFECT under test: an unrecognized trailing clause
