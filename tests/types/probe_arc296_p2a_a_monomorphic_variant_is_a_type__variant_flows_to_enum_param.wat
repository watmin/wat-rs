;; ⛔ THE SUBSUMPTION CONTROL. Green today, and the row that breaks if the ctor
;; stops erasing without the variant->enum edge carrying it. Colour::Red must
;; still be accepted where Colour is expected.
(:wat::core::defenum :usr::Colour :wat::enum::Pure
  :Red  [shade <- :wat::core::i64]
  :Blue [shade <- :wat::core::i64])
(:wat::core::defn :user::takes-colour [c <- :usr::Colour] -> :wat::core::nil
  (:wat::kernel::println "ok"))
(:wat::core::defn :user::main [] -> :wat::core::nil
  (:user::takes-colour (:usr::Colour::Red {:shade 7})))
