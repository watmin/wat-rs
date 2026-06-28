;; enums_tagged_arity_mismatch_bad.wat — tagged variant pattern wrong binder count. Must FAIL.
(:wat::core::defenum :my::Event
  :Pair [a <- :wat::core::i64 b <- :wat::core::i64])
(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::match (:my::Event::Pair 1 2) -> :wat::core::i64
    ((:my::Event::Pair just-one) just-one)))
