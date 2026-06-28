;; tests/types/probe_arc232_generic_method.wat — co-located fixture
;;
;; Arc 232 follow-on — GENERIC protocol method signatures.

(:wat::core::defprotocol :t::Maker
  (make<T> [self <- :t::Maker  x <- :T] -> :wat::core::Vector<T>))

(:wat::core::defrecord :t::Dup [])
(:wat::core::extend-type :t::Dup :t::Maker (make [self x] [x x]))

(:wat::core::defn :user::go [] -> :wat::core::i64
  (:wat::core::nth (:t::Maker/make (:t::Dup) 5) 0))

