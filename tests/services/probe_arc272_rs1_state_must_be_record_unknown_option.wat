;; NEGATIVE: a bogus trailing option must be rejected directly (named), not silently mis-read.
(:wat::service::defservice :my::counter
  :durable [count <- :wat::core::i64]
  :ops
  [(:Get [s <- :State]
         -> [value <- :wat::core::i64]
     (:wat::service::Outcome::Reply s (:my::counter::GetResponse (:my::counter::Record/count (:my::counter::State/durable s)))))]
  :bogus-option :wat::core::Record)
