;; BRIEF-STONE-the-dormant-minter.md — control B: binder, NO kwargs. Expect 5.
(:wat::core::defn :dm109b::hold :- [T] [seed <- :T] -> :T seed)

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::kernel::println
    (:wat::string::interpolate "B={b}"
      :b (:wat::core::i64::to-string (:dm109b::hold 5)))))
