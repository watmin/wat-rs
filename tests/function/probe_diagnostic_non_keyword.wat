;; tests/function/probe_diagnostic_non_keyword.wat — NEGATIVE probe 7: non-keyword head.
;; apply with a String literal as head type-checks (result inferred) but errors at EVAL (not callable).

(:wat::core::defn :user::bad [] -> :wat::core::String
  (:wat::core::apply  "not-a-keyword" []))
