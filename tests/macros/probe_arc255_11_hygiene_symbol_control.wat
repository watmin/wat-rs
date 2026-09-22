;; Arc 255 Stone 255.11 — ⛔ THE POSITIVE CONTROL for the `.bad` sibling.
;;
;; The SAME symbol-spelled nested quasiquote, introducing NO literal binder: it must
;; still register, expand and compute. A wall that refuses every symbol-spelled
;; quasiquote is not an intact wall.
(:wat::core::defmacro :my::twice
  [x <- :wat::WatAST]
  -> :wat::WatAST
  (:wat::core::if (:wat::core::= 1 1)
    (wat.core/quasiquote (:wat::i64::+ (:wat::core::unquote x) (:wat::core::unquote x)))
    `~x))
(:wat::core::defn :user::compute [] -> :wat::core::bool
  (:wat::core::= (:my::twice 3) 6))
