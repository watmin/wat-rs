;; tests/function/probe_arc241_stone2_fn_parser_migration.wat
;; Arc 241 Stone 241.2 — fn-signature parser migration (A1/A2/A3) behavioral parity.
;; Co-located fixture, slurped via startup_beside(file!()).
;; Negative (startup-fail) cases are in sibling *_bad.wat files.

;; Contract 1 — no-arg fn succeeds
(:wat::core::defn :user::c01-f [] -> :wat::core::i64
  ((:wat::core::fn [] -> :wat::core::i64 42)))

;; Contract 2 — single-arg fn succeeds
(:wat::core::defn :user::c02-f [] -> :wat::core::i64
  ((:wat::core::fn [x <- :wat::core::i64] -> :wat::core::i64 x) 7))

;; Contract 3 — multi-arg fn succeeds
(:wat::core::defn :user::c03-f [] -> :wat::core::i64
  ((:wat::core::fn [x <- :wat::core::i64 y <- :wat::core::i64] -> :wat::core::i64
                  (:wat::core::+ x y)) 3 4))

;; Contract 4 — let-bound fn succeeds
(:wat::core::defn :user::c04-f [] -> :wat::core::i64
  (:wat::core::let [g (:wat::core::fn [x <- :wat::core::i64] -> :wat::core::i64 x)]
                 (g 42)))
