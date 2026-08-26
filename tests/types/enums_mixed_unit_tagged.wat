;; enums_mixed_unit_tagged.wat — mixed unit + tagged arms in one match.
(:wat::core::defenum :my::Event :wat::enum::Pure
  :Open [size <- :wat::core::f64]
  :Hold)
(:wat::core::defn :my::act [e <- :my::Event] -> :wat::core::String
  (:wat::core::match e 
    ((:my::Event::Open size) (:wat::f64::to-string size))
    (:my::Event::Hold        "hold")))
(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::let
    [line1 (:my::act (:my::Event::Open 7.5))
     line2 (:my::act :my::Event::Hold)]
    (:wat::core::do
      (:wat::kernel::println line1)
      (:wat::kernel::println line2))))
