;; Fixture: UNPRIMED two-param generic head — must lex + check. The control for the
;; primed twin: identical shape, the apostrophe the ONLY variable. Uses a LIVE
;; two-param generic; the subject is the LEXER, which never consults the registry.
(:wat::core::defn :user::take [m <- (:wat::core::HashMap :- [:wat::core::String :wat::core::String])] -> :wat::core::nil nil)
