;; ⛔ THE WIDEST CONTROL — every existing construction in the corpus depends on this.
(:wat::core::defenum :usr::Box :- [T] :wat::enum::Pure
  :Full  [inside <- :T]
  :Empty [])
(:wat::core::defn :user::takes-box [b <- (:usr::Box :- [:wat::core::i64])] -> :wat::core::nil
  (:wat::kernel::println "ok"))
(:wat::core::defn :user::main [] -> :wat::core::nil
  (:user::takes-box (:usr::Box.Full {:inside 42})))
