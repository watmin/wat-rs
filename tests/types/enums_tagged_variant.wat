;; enums_tagged_variant.wat — tagged variant construction + match with binders.
(:wat::core::defenum :my::Event :wat::enum::Pure
  :Candle  [open <- :wat::core::f64 close <- :wat::core::f64]
  :Deposit [amount <- :wat::core::f64]
  :Nothing)
(:wat::core::defn :my::a-candle [] -> :my::Event (:my::Event::Candle 100.0 105.0))
(:wat::core::defn :my::summary [e <- :my::Event] -> :wat::core::String
  (:wat::core::match e 
    ((:my::Event::Candle  o c) (:wat::f64::to-string c))
    ((:my::Event::Deposit amt) (:wat::f64::to-string amt))
    (:my::Event::Nothing       "nothing")))
(:wat::core::defn :user::main [] -> :wat::core::nil (:wat::kernel::println (:my::summary (:my::a-candle))))
