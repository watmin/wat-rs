;; tests/macros/make_deftest.wat — co-located fixture for make_deftest.rs,
;; slurped via startup_beside(file!()).
;;
;; macroexpand_self_recursive_macro_fails_with_macro_expansion_failed:
;; Registers :my::ping and :my::pong (mutual recursion); exposes :probe::run-macroexpand.
;;
;; (arc 278: the make-deftest factory was annihilated; its former
;; diag_make_deftest_with_prelude_expansion probe + :probe::get-expansion fn
;; were retired with it. The self-recursive macroexpand probe is independent.)

(:wat::core::defmacro :my::ping
  []
  -> wat.type/AST
  `(:my::pong))

(:wat::core::defmacro :my::pong
  []
  -> wat.type/AST
  `(:my::ping))

(:wat::core::defn :probe::run-macroexpand [] -> wat.type/AST
  (:wat::core::macroexpand
    (:wat::core::quote (:my::ping))))

