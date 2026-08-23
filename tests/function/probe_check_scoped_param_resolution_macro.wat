;; tests/function/probe_check_scoped_param_resolution_macro.wat
;; NEGATIVE fixture — startup MUST fail (ReturnTypeMismatch) after Stone 249.5e fix.
;; A macro-generated defclause with the same ret-mismatch as the handwritten control.
;; At HEAD (before fix) this checks clean (bug); after fix it is rejected.

(:wat::core::defmacro :test::make-bad-ret
  [] -> (:AST :- [:wat::holon::HolonAST])
  `(:wat::core::defclause :test::bad-ret
     ([x <- :wat::core::i64] -> :wat::core::bool x)))

(:test::make-bad-ret)

