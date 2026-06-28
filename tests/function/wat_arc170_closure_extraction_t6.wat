;; T6: lambda captures multiple values, mixed types.
(:wat::core::defstruct :my::Cfg
  [label <- :wat::core::String])
(:wat::core::defn :my::make-multi [] -> :wat::core::Fn(wat::core::i64)->wat::core::i64
  (:wat::core::let
              [n 7
               cfg (:my::Cfg/new "ok")
               xs (:wat::core::Vector :wat::core::i64 1 2 3)]
              (:wat::core::fn [m <- :wat::core::i64] -> :wat::core::i64
                (:wat::core::i64::+ m
                  (:wat::core::i64::+ n
                    (:wat::core::Vector/length xs))))))
(:wat::core::defn :user::main [] -> :wat::core::nil nil)
