;; CONTROL — builtins and an instantiated generic. GREEN now, must stay green.
(:wat::core::defn :user::f [o <- (:wat::core::Option :- [:wat::core::i64])] -> :wat::core::i64
  (:wat::core::match o
    [:wat::core::Option::Some {:value v} v]
    [:wat::core::Option::None {}         0]))
(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::kernel::println (:user::f (:wat::core::Option::Some {:value 42}))))
