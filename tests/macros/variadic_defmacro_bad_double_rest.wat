;; tests/macros/variadic_defmacro_bad_double_rest.wat — NEGATIVE fixture for variadic_defmacro.rs
;; (double_rest_marker_refused_at_registration). Must fail with StartupError::Macro.
(:wat::core::defmacro :my::bogus
  [& & items <- (:AST :- [:wat::holon::Holons])]
  -> (:AST :- [:wat::holon::HolonAST])
  `(:wat::core::Vector :- [:wat::core::i64] ~@items))

(:wat::core::defn :user::main [] -> :wat::core::i64 0)
