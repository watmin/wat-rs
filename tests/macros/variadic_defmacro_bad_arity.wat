;; tests/macros/variadic_defmacro_bad_arity.wat — NEGATIVE fixture for variadic_defmacro.rs
;; (variadic_macro_requires_at_least_fixed_arity). Must fail with StartupError::Macro.
;;
;; (:my::sum-of) with NO args — fixed-arity of :init is 1, so zero args is a short call.
(:wat::core::defmacro :my::sum-of
  [init <- (:AST :- [wat.type/i64])
   & items <- (:AST :- [:wat::holon::Holons])]
  -> (:AST :- [:wat::holon::HolonAST])
  `(:wat::core::foldl
      (:wat::core::fn [acc <- wat.type/i64 x <- wat.type/i64] -> wat.type/i64
        (:wat::i64::+ acc x))
      ~init
      (wat.type/Vector :- [wat.type/i64] ~@items)))

(:wat::core::defn :user::main [] -> wat.type/i64 (:my::sum-of))
