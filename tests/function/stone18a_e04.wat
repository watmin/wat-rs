;; tests/function/stone18a_e04.wat — NEGATIVE fixture: fn-form non-keyword return type.
;; E04: symbol `nil` where keyword expected.

(:wat::core::defn :test::bad [] -> :wat::core::nil
  ((:wat::core::fn [] -> nil nil)))

(:wat::core::defn :user::main [] -> :wat::core::nil nil)
