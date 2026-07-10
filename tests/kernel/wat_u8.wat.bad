;; Negative fixture for wat_u8.rs — startup must fail (type mismatch at check time).
;; :my::probe passes i64 literal 42 where :wat::core::u8 is expected.

(:wat::core::defn :my::app::byte-taker [b <- :wat::core::u8] -> :wat::core::u8 b)

(:wat::core::defn :my::probe [] -> :wat::core::u8
  (:my::app::byte-taker 42))

