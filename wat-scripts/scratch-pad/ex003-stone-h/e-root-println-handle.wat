;; Excursus 003 stone H — control: in the ROOT process stdout is text, not a wire, so println of a
;; handle still prints its nil-bodied tag (arc 294).
(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::kernel::println
    (:wat::core::Result/expect (:wat::cache::Lru/new :- [:wat::core::i64 :wat::core::i64] 2) "lru")))
