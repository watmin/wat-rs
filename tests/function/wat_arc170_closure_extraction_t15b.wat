;; T15b: Config struct + make-adder with offset=99 (used in T15's src5 shape).
(:wat::core::defstruct :my::Config [offset <- :wat::core::i64])
(:wat::core::defn :my::make-adder [] -> :wat::core::Fn(wat::core::i64)->wat::core::i64
  (:wat::core::let
              [cfg (:my::Config :offset 99)]
              (:wat::core::fn [n <- :wat::core::i64] -> :wat::core::i64
                (:wat::i64::+ n (:my::Config/offset cfg)))))
