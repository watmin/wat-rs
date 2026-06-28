(:wat::core::defmacro :my::inc-vof
  [& items <- :wat::core::Vector<wat::WatAST>] -> :wat::WatAST
  (:wat::core::let [mapped (:wat::core::map
                             (:wat::core::fn [x <- :wat::holon::HolonAST] -> :wat::holon::HolonAST
                               `(:wat::core::i64::+ ~x 1))
                             items)]
    `(:wat::core::Vector :wat::core::i64 ~@mapped)))
(:wat::core::defn :user::compute [] -> :wat::core::i64
  (:wat::core::match (:wat::core::get (:my::inc-vof 10 20 30) 0) -> :wat::core::i64
    ((:wat::core::Some n) n)
    (:wat::core::None -1)))
