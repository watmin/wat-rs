;; Arc 255 Stone 255.11 — the wall audit.
;;
;; A restriction governs MENTION, not head position (arc 198's own ruling). A DATA
;; position is never normalized (`resolve/boundary.rs`), so before 255.11 this exact
;; file — identical to the keyword-spelled sibling except for ONE mention's spelling —
;; type-checked CLEAN, and handing the quoted form to `(:wat::eval-ast! ...)` ran the
;; restricted fn. The walker now reads both spellings.

(:wat::core::defn :my::kernel::restricted-fn
  {:restricted-to [:my::kernel::]}
  [x <- :wat::core::i64] -> :wat::core::i64 x)

(:wat::core::defn :user::app::caller [] -> :wat::WatAST
  (:wat::core::quote (my.kernel/restricted-fn 7)))
