;; Same form as spelling-dotted.wat and spelling-clojure.wat. ≥2 defn args, ≥2 let binders.
(:wat::core::defn :fix::two
  [a <- :wat::core::i64
   b <- :wat::core::i64]
  -> :wat::core::i64
  (:wat::core::let
    [x a
     y b]
    x))
