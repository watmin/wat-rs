;; Arc 255 Stone 255.11 — THE POSITIVE CONTROL for the sibling `_denied` fixture.
;;
;; A wall that refuses everything is not intact. The SAME symbol-spelled mention in the
;; SAME data position, from a caller the whitelist DOES admit, must still pass.

(:wat::core::defn :my::kernel::restricted-fn
  {:restricted-to [:my::kernel::]}
  [x <- :wat::core::i64] -> :wat::core::i64 x)

(:wat::core::defn :my::kernel::ok-caller [] -> :wat::WatAST
  (:wat::core::quote (my.kernel/restricted-fn 7)))
