;; tests/macros/probe_macro_hygiene_capture.wat — co-located fixture for
;; probe_macro_hygiene_capture.rs, slurped via startup_beside(file!()).
;;
;; Contains all declarations for the three tests plus named compute functions.

;; From MAKE_MACRO_ADD: a macro that emits a defclause for :test::macro-add.
(:wat::core::defmacro :test::make-macro-add
  [] -> :wat::WatAST
  `(:wat::core::defclause :test::macro-add
     ([x <- :wat::core::i64 y <- :wat::core::i64] -> :wat::core::i64
       (:wat::i64::+ x y))))

;; From CALL_MAKE_MACRO_ADD: register the defclause at top level.
(:test::make-macro-add)

;; From CAPTURE_MACRO: macro with a let-bound tmp that should NOT capture caller's tmp.
(:wat::core::defmacro :test::add-via-tmp
  [x <- :wat::WatAST] -> :wat::WatAST
  `(:wat::core::let [tmp 100] (:wat::i64::+ tmp ~x)))

;; From two_scope test: outer macro that registers inner-add with 2-scope accum.
(:wat::core::defmacro :test::make-add-inner
  [] -> :wat::WatAST
  `(:wat::core::defmacro :test::inner-add
     [x <- :wat::WatAST] -> :wat::WatAST
     `(:wat::core::let [tmp 10] (:wat::i64::+ tmp ~x))))
(:test::make-add-inner)

;; Named compute function for test 1 (macro_generated_defclause_resolves_params).
(:wat::core::defn :test::compute-1 [] -> :wat::core::i64
  (:test::macro-add 3 4))

;; Named compute function for test 2 (classic_macro_capture_is_prevented).
(:wat::core::defn :test::compute-2 [] -> :wat::core::i64
  (:wat::core::let [tmp 5] (:test::add-via-tmp tmp)))

;; Named compute function for test 3 (two_scope_identifier_resolves_correctly_end_to_end).
(:wat::core::defn :test::compute-3 [] -> :wat::core::i64
  (:test::inner-add 7))

