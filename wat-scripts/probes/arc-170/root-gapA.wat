;; ROOT Gap A — fn-forms directly on the kwargs $impl vs a hand-written same-shape fn.
(:wat::core::defstruct :probe::Bag [n <- :wat::core::String])
(:wat::core::defn :probe::hand
  [item <- :wat::core::String  bag <- :probe::Bag] -> :wat::core::String
  (:wat::core::let [x (:probe::Bag/n bag)] (:wat::string::concat x item)))
(:wat::core::defn :probe::work
  [item <- :wat::core::String  & [n <- :wat::core::String]] -> :wat::core::String
  (:wat::string::concat n item))
(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::let
    [_  (:wat::kernel::println "--- fn-forms HAND-WRITTEN ---")
     hf (:wat::kernel::fn-forms :probe::hand :test::hand)
     _  (:wat::kernel::println "hand: ok")
     _  (:wat::kernel::println "--- fn-forms KWARGS impl ---")
     kf (:wat::kernel::fn-forms :probe::work$impl :test::work)
     _  (:wat::kernel::println "work impl: ok")]
    (:wat::kernel::println "both ok")))
