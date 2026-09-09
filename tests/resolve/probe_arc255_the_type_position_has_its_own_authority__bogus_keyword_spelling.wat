;; RED — the KEYWORD spelling of the same shape. normalize never touches this form
;; (it rewrites Symbols only), so this row isolates the wall from the rewrite.
(:wat::core::defn :user::h [p <- (:wat::core::Tuple :- [:wat::core::Bogus])] -> :wat::core::i64 1)
(:wat::core::defn :user::main [] -> :wat::core::nil (:wat::kernel::println "x"))
