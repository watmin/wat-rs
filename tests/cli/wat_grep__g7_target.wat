;; wat_grep__g7_target.wat — TARGET for G7 (end-to-end `--grep`): exactly one `<-` binder symbol
;; (in the `[x <- :wat::core::i64]` param list), so the printed Match is asserted on precisely.
(:wat::core::defn :user::identity
  [x <- :wat::core::i64]
  -> :wat::core::i64
  x)
