(:wat::core::defmacro :probe::splice-i64
  [xs <- :wat::WatAST]
  -> :wat::WatAST
  (:wat::core::quasiquote
    (:wat::core::Vector :- [:wat::core::i64]
      (:wat::core::unquote-splicing
        ;; Arc 118.2a — `map` flipped LAZY; `doubled` is unquote-spliced (computed unquote-
        ;; splicing runs through the SAME restricted macro-eval evaluator as a program-body
        ;; macro — wat-defined `mapv` is `UnknownFunction` there), so `foldl`+`conj` (Rust-
        ;; native) stand in instead of `mapv`.
        (:wat::core::let
          [doubled (:wat::core::foldl
                     (:wat::core::fn [acc <- (:wat::core::Vector :- [:wat::core::i64]) x <- :wat::core::i64] -> (:wat::core::Vector :- [:wat::core::i64])
                       (:wat::core::conj acc (:wat::i64::* x 2)))
                     (:wat::core::Vector :- [:wat::core::i64])
                     xs)]
          doubled)))))

(:wat::core::defn :user::compute [] -> (:wat::core::Vector :- [:wat::core::i64]) (:probe::splice-i64 [1 2 3]))
