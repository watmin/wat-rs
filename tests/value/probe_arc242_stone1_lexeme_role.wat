;; tests/value/probe_arc242_stone1_lexeme_role.wat — co-located fixture.
;; Slurped via startup_beside(file!()). Startup SUCCESS is the assertion for C01/C02/C04.
;;
;; C01: bare nil works as primitive value in expression position.
;; C02: :wat::core::nil preserved as type in signature position.
;; C04: :wat::core::char (lowercase) works as type.

;; C01: bare nil as primitive VALUE (body expression)
(:wat::core::defn :test::returns-nil [] -> :wat::core::nil nil)

;; C02: :wat::core::nil as TYPE in parameter/return signatures
(:wat::core::defn :test::accepts-nil [x <- :wat::core::nil] -> :wat::core::nil x)

;; C04: :wat::core::char (lowercase) as TYPE in parameter/return signatures
(:wat::core::defn :test::needs-char-lowercase [c <- :wat::core::char] -> :wat::core::char c)

