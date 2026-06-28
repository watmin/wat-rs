;; tests/function/stone18a_e06.wat — NEGATIVE fixture: fn-form keyword in argspec name slot.
;; E06: `:kw` (a keyword) in the name slot instead of a symbol.

(:wat::core::defn :test::bad [] -> :wat::core::nil
  ((:wat::core::fn [:kw <- :wat::core::i64] -> :wat::core::i64 42)))

(:wat::core::defn :user::main [] -> :wat::core::nil nil)
