;; tests/macros/variadic_defmacro_bad_rest_no_binder.wat — NEGATIVE fixture for variadic_defmacro.rs
;; (rest_marker_without_binder_refused_at_registration). Must fail with StartupError::Macro.
(:wat::core::defmacro :my::bogus
  [x <- (:AST :- [:wat::core::i64])
   &]
  -> (:AST :- [:wat::holon::HolonAST])
  `(:wat::core::i64::+ ~x 0))

(:wat::core::defn :user::main [] -> :wat::core::i64 0)
