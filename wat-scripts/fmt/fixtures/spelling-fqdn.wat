;; Same form as spelling-dotted.wat and spelling-clojure.wat. ≥2 defn args, ≥2 let binders.
(:wat::core::defn :fix::two
  [a <- wat.type/i64
   b <- wat.type/i64]
  -> wat.type/i64
  (:wat::core::let
    [x a
     y b]
    x))
