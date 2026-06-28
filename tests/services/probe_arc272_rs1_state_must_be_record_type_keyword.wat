;; NEGATIVE: a bare type keyword in :durable is still unexpressible.
(:wat::service::defservice :my::counter
  :durable :wat::core::i64
  :ops
  [(:Get [s <- :State]
         -> [value <- :wat::core::i64]
     (:wat::service::Outcome::Reply s (:my::counter::GetResponse s)))])
