;; tests/function/probe_diagnostic_non_keyword_bad.wat — NEGATIVE probe 7: non-keyword head.
;; apply with a String literal as head must reject at check time. startup MUST fail.

(:wat::core::defn :user::bad [] -> :wat::core::String
  (:wat::core::apply -> :wat::core::String "not-a-keyword" []))
