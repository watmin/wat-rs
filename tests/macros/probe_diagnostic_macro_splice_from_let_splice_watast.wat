(:wat::core::defmacro :probe::splice-watast
  [xs <- wat.type/AST]
  -> wat.type/AST
  (:wat::core::quasiquote
    (wat.type/Vector :- [wat.type/i64]
      (:wat::core::unquote-splicing
        ;; Arc 118.2a — `map` flipped LAZY; `forms` is unquote-spliced (computed unquote-
        ;; splicing runs through the restricted macro-eval evaluator — wat-defined `mapv` is
        ;; `UnknownFunction` there), so `foldl`+`conj` (Rust-native) stand in.
        (:wat::core::let
          [forms (:wat::core::foldl
                   (:wat::core::fn [acc <- (wat.type/Vector :- [wat.type/AST]) x <- wat.type/i64] -> (wat.type/Vector :- [wat.type/AST])
                     (:wat::core::conj acc
                       (:wat::core::quasiquote
                         (:wat::core::unquote (:wat::i64::* x 10)))))
                   (wat.type/Vector :- [wat.type/AST])
                   xs)]
          forms)))))

(:wat::core::defn :user::compute [] -> (wat.type/Vector :- [wat.type/i64]) (:probe::splice-watast [1 2 3]))
