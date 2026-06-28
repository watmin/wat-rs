;; tests/function/probe_arc241_stone3_defclause_parser_migration.wat
;; Arc 241 Stone 241.3 — A4 defclause parser migration behavioral parity.
;; Co-located fixture, slurped via startup_beside(file!()).
;; Negative (startup-fail) cases are in sibling *_bad.wat files.

;; Contract 1 — no-arg defclause succeeds
(:wat::core::defclause :user::c01-f
  ([] -> :wat::core::i64 42))

;; Contract 2 — single-arg defclause succeeds
(:wat::core::defclause :user::c02-f
  ([x <- :wat::core::i64] -> :wat::core::i64 x))

;; Contract 3 — multi-arg defclause succeeds
(:wat::core::defclause :user::c03-f
  ([x <- :wat::core::i64 y <- :wat::core::i64] -> :wat::core::i64
    (:wat::core::+ x y)))
