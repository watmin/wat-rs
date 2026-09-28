;; Arc 118.2a — `map` flipped LAZY; `mapped` is unquote-spliced (`~@mapped`) — computed
;; unquote-splicing runs through the restricted macro-eval evaluator (wat-defined `mapv` is
;; `UnknownFunction` there), so `foldl`+`conj` (Rust-native) stand in.
(:wat::core::defmacro :my::inc-vof
  [& items <- (wat.type/Vector :- [wat.type/AST])] -> wat.type/AST
  (:wat::core::let [mapped (:wat::core::foldl
                             (:wat::core::fn [acc <- (wat.type/Vector :- [wat.type/AST]) x <- :wat::holon::HolonAST] -> (wat.type/Vector :- [wat.type/AST])
                               (:wat::core::conj acc `(:wat::i64::+ ~x 1)))
                             (wat.type/Vector :- [wat.type/AST])
                             items)]
    `(wat.type/Vector :- [wat.type/i64] ~@mapped)))
(:wat::core::defn :user::compute [] -> wat.type/i64
  (:wat::core::match (:wat::core::get (:my::inc-vof 10 20 30) 0) 
    [:wat::core::Option.Some {:value n} n]
    [:wat::core::Option.None {} -1]))
