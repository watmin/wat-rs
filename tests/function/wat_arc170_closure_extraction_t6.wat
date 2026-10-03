;; T6: lambda captures multiple values, mixed types.
(:wat::core::defstruct :my::Cfg
  [label <- wat.type/String])
(:wat::core::defn :my::make-multi [] -> [wat.type/i64 :-> wat.type/i64]
  (:wat::core::let
              [n 7
               cfg (:my::Cfg :label "ok")
               xs (wat.type/Vector :- [wat.type/i64] 1 2 3)]
              (:wat::core::fn [m <- wat.type/i64] -> wat.type/i64
                (:wat::i64::+ m
                  (:wat::i64::+ n
                    (:wat::core::length xs))))))
