;; tests/value/probe_arc242_stone1_lexeme_role_bad.wat — NEGATIVE fixture.
;; Used by startup_from_file; startup must return Err with retirement remedy.
;;
;; C03: :wat::core::Char (uppercase/PascalCase) is a HARD CUT — scalar types are
;; lowercase per Doctrine 2 (lexeme-role-doctrine). The substrate must reject this
;; with a structured retirement remedy pointing at :wat::core::char.

(:wat::core::defn :test::needs-char [c <- :wat::core::Char] -> :wat::core::Char c)

(:wat::core::defn :user::main [] -> :wat::core::nil nil)
