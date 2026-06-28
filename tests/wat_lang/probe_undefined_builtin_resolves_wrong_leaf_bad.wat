;; tests/wat_lang/probe_undefined_builtin_resolves_wrong_leaf_bad.wat
;; RED-at-HEAD: renamed-away operator leaf — should fail check/resolve after arc-255 fix.

(:wat::core::defn :user::main [] -> :wat::core::i64
  (:wat::core::i64::+'2 1 2))
