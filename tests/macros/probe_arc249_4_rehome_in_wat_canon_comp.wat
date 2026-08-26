;; Arc 118.2a — `map` flipped LAZY; `mapped` is unquote-spliced (`~@mapped`) — computed
;; unquote-splicing runs through the restricted macro-eval evaluator (wat-defined `mapv` is
;; `UnknownFunction` there), so `foldl`+`conj` (Rust-native) stand in.
(:wat::core::defmacro :my::inc-vof
  [& items <- (:wat::core::Vector :- [:wat::WatAST])] -> :wat::WatAST
  (:wat::core::let [mapped (:wat::core::foldl
                             (:wat::core::fn [acc <- (:wat::core::Vector :- [:wat::WatAST]) x <- :wat::holon::HolonAST] -> (:wat::core::Vector :- [:wat::WatAST])
                               (:wat::core::conj acc `(:wat::i64::+ ~x 1)))
                             (:wat::core::Vector :wat::WatAST)
                             items)]
    `(:wat::core::Vector :wat::core::i64 ~@mapped)))
(:wat::core::defn :user::compute [] -> :wat::core::i64
  (:wat::core::match (:wat::core::get (:my::inc-vof 10 20 30) 0) 
    ((:wat::core::Some n) n)
    (:wat::core::None -1)))
