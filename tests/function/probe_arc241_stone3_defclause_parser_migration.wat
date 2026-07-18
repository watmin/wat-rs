;; tests/function/probe_arc241_stone3_defclause_parser_migration.wat
;; Arc 241 Stone 241.3 — A4 defclause parser migration behavioral parity.
;; Co-located fixture, slurped via startup_beside(file!()).
;; Negative (startup-fail) cases are in sibling *.wat.bad files.

;; Contract 1 — no-arg defclause succeeds
;; defclause registers its ClauseSet in runtime_def_values (Value::wat__core__clauses),
;; NOT sym.functions — so it is not retrievable via SymbolTable::get / apply_function
;; (which operate on Arc<Function> from sym.functions, per the passing sibling fixture
;; probe_arc241_stone5_defclause_rest_dispatch.wat's pattern). A thin `defn` wrapper
;; (which DOES register in sym.functions) forwards to the defclause impl so the .rs
;; harness's `startup_beside` + `symbols().get` + `apply_function` path can reach it,
;; while the defclause form itself still exercises the A4 parser under test.
(:wat::core::defclause :impl::c01
  ([] -> :wat::core::i64 42))
(:wat::core::defn :user::c01-f [] -> :wat::core::i64 (:impl::c01))

;; Contract 2 — single-arg defclause succeeds
(:wat::core::defclause :impl::c02
  ([x <- :wat::core::i64] -> :wat::core::i64 x))
(:wat::core::defn :user::c02-f [x <- :wat::core::i64] -> :wat::core::i64 (:impl::c02 x))

;; Contract 3 — multi-arg defclause succeeds
(:wat::core::defclause :impl::c03
  ([x <- :wat::core::i64 y <- :wat::core::i64] -> :wat::core::i64
    (:wat::core::+ x y)))
(:wat::core::defn :user::c03-f [x <- :wat::core::i64 y <- :wat::core::i64] -> :wat::core::i64 (:impl::c03 x y))
