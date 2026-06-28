;; tests/macros/make_deftest.wat — co-located fixture for make_deftest.rs,
;; slurped via startup_beside(file!()). Contains both programs from the original
;; inlined test sources, merged under one :user::main.
;;
;; Program 1 (diag_make_deftest_with_prelude_expansion):
;; Registers :my-deftest via make-deftest; exposes :probe::get-expansion.
;;
;; Program 2 (macroexpand_self_recursive_macro_fails_with_macro_expansion_failed):
;; Registers :my::ping and :my::pong (mutual recursion); exposes :probe::run-macroexpand.

(:wat::test::make-deftest :my-deftest
  ((:wat::load-file! "foo.wat")))

(:wat::core::defn :probe::get-expansion [] -> :wat::WatAST
  (:wat::core::macroexpand-1
      (:wat::core::quote (:my-deftest :my-test (:wat::test::assert-eq 1 1)))))

(:wat::core::defmacro :my::ping
  []
  -> :wat::WatAST
  `(:my::pong))

(:wat::core::defmacro :my::pong
  []
  -> :wat::WatAST
  `(:my::ping))

(:wat::core::defn :probe::run-macroexpand [] -> :wat::WatAST
  (:wat::core::macroexpand
    (:wat::core::quote (:my::ping))))

