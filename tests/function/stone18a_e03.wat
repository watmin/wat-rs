;; tests/function/stone18a_e03.wat — NEGATIVE fixture: fn-form missing arrow.
;; E03: no `->` symbol between args-vector and return type.

(:wat::core::defn :test::bad [] -> :wat::core::nil
  ((:wat::core::fn [] :wat::core::nil nil)))

