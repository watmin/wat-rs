;; Arc 118.2a — `map` flipped LAZY; materialize via `mapv` for the equality check against a
;; Vector literal (this probe's intent — fn-first no-threading regression — is unaffected).
(:wat::core::defn :user::compute [] -> :wat::core::bool
  (:wat::core::= (:wat::core::mapv (:wat::core::fn [x <- :wat::core::i64] -> :wat::core::i64 (:wat::i64::+ x 1)) [1 2 3]) [2 3 4]))
