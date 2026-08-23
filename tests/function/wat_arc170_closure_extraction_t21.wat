;; T21: outer let binds n=100; match arm's Some-pattern shadows n; None arm uses outer n.
(:wat::core::defn :my::shadow-test [opt <- (:wat::core::Option :- [:wat::core::i64])] -> :wat::core::i64
  (:wat::core::let
              [n 100]
              (:wat::core::match opt 
                ((:wat::core::Some n) n)
                (:wat::core::None     n))))
