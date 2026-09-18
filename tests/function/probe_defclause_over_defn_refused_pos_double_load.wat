;; THE LIVE PATH the wall would most easily break: one file loaded TWICE.
(:wat::load-file! "clause-lib.wat")
(:wat::load-file! "clause-lib.wat")
(:wat::core::defn :user::probe [] -> :wat::core::i64 (:libc::twice 21))
