;; T7: factory pattern — factory captures its arg.
(:wat::core::defstruct :my::Cfg
  [val <- :wat::core::i64])
(:wat::core::defn :my::factory [config <- :my::Cfg] -> :wat::core::Fn(wat::core::i64)->wat::core::i64
  (:wat::core::fn [n <- :wat::core::i64] -> :wat::core::i64
              (:wat::i64::+ n (:my::Cfg/val config))))
(:wat::core::defn :my::make [] -> :wat::core::Fn(wat::core::i64)->wat::core::i64 (:my::factory (:my::Cfg :val 100)))
