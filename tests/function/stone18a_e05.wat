;; tests/function/stone18a_e05.wat — NEGATIVE fixture: fn-form wrong arrow symbol.
;; E05: symbol `=>` where `->` is expected.

(:wat::core::defn :test::bad [] -> wat.type/nil
  ((:wat::core::fn [] => wat.type/nil nil)))

