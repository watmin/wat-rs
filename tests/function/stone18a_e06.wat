;; tests/function/stone18a_e06.wat — NEGATIVE fixture: fn-form keyword in argspec name slot.
;; E06: `:kw` (a keyword) in the name slot instead of a symbol.

(:wat::core::defn :test::bad [] -> wat.type/nil
  ((:wat::core::fn [:kw <- wat.type/i64] -> wat.type/i64 42)))

