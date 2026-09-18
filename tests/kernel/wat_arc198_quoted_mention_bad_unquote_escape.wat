;; ⛔ THE BOUNDARY. Not everything under a quasiquote is quoted: an unquote escape
;; is evaluated NOW, in THIS program, so the name inside it is a REAL mention and
;; arc 198's hole must stay shut. A narrowing that exempted the whole quasiquote
;; would reopen that hole through `~`.
;;
;; The escape's body is the `apply`-laundering shape arc 198 was written against —
;; the restricted FQDN in ARGUMENT position, never a call head.
(:wat::core::defn :my::kernel::restricted-fn
  {:restricted-to [:my::kernel::]}
  [x <- :wat::core::i64] -> :wat::core::i64 x)

(:wat::core::defn :user::sneaky [] -> :wat::WatAST
  (:wat::core::quasiquote
    (:wat::core::do
      (:wat::core::unquote (:wat::core::apply :my::kernel::restricted-fn [7])))))
