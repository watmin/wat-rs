;; Arc 109 wave 2 ("annihilate the angle bracket") — the rest-param's original type
;; text here was `:AST<wat::holon::Holons>`, a legacy parametric placeholder. Its
;; CONTENT was always discarded by `fix-macro-param-types` — `argspec-type-edits-walk`
;; emits the CONSTANT text `(:wat::core::Vector :- [:wat::WatAST])` for any Keyword-
;; shaped rest-param type, never derived from the old annotation — so any Keyword type
;; here exercises the identical rule. `read-string` now refuses the angle form at the
;; lexer wall before this rule ever runs, so the decoration is migrated to a plain,
;; still-legal keyword; the rule under test — Keyword-shaped type slots on a defmacro's
;; rest param get rewritten, content-agnostic — is unaffected and its golden is
;; unchanged. Class 3 (a): subject survives.
(:wat::core::defn :user::run [] -> :wat::core::String
  (:wat::fix::fix-macro-param-types ";; keep me byte-identical\n(:wat::core::defmacro :user::m [a <- :wat::holon::HolonAST & rest <- :wat::holon::Holons] -> :wat::holon::HolonAST a)\n(:wat::core::defn :user::f [x <- :wat::core::i64] -> :wat::core::i64 x)"))

