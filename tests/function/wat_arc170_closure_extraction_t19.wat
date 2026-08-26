;; T19: match arm body uses inner let referencing arm-bound name `i`.
(:wat::core::defn :my::inc-or-default [opt <- (:wat::core::Option :- [:wat::core::i64])] -> :wat::core::i64
  (:wat::core::match opt 
              ((:wat::core::Some i)
               (:wat::core::let
                 [s (:wat::i64::+ i 1)]
                 s))
              (:wat::core::None 0)))
