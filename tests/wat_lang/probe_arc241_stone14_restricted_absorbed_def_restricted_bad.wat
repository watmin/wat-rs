;; tests/wat_lang/probe_arc241_stone14_restricted_absorbed_def_restricted_bad.wat
;; :wat::core::def-restricted — must be HARD-CUT-rejected at startup (Stone 241.14).

(:wat::core::def-restricted :test::r
  :restricted-to [:test::]
  (:wat::core::fn [] -> :wat::core::i64 42))
