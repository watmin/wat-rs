;; wat_grep__no_reader_macros.wat — a TARGET file with NEITHER reader macros (no `~`, `` ` ``,
;; `'`, `~@`, `\c`, `#holon`) NOR string literals, for G6: Written count == Named count. Every
;; nameable node here is a Symbol or Keyword, so the predicate's only-validated case (see the
;; report: symbol/keyword are exact, string is NOT) applies cleanly.
(:wat::core::defn :user::add
  [x <- :wat::core::i64
   y <- :wat::core::i64]
  -> :wat::core::i64
  (:wat::core::i64::+ x y))
