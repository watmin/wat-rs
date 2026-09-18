;; The narrowing, stated positively: a restricted FQDN appearing as TEMPLATE TEXT —
;; inside `(:wat::core::forms …)` and inside a quasiquote with no unquote around it —
;; is source for a child program, not a mention by the enclosing fn. Neither fires.
;; `restricted-fn` is whitelisted to `:my::kernel::`; both enclosing fns are `:user::`.
(:wat::core::defn :my::kernel::restricted-fn
  {:restricted-to [:my::kernel::]}
  [x <- :wat::core::i64] -> :wat::core::i64 x)

(:wat::core::defn :user::child-program [] -> (:wat::core::Vector :- [:wat::WatAST])
  (:wat::core::forms
    (:wat::core::defn :user::main [] -> :wat::core::nil
      (:my::kernel::restricted-fn 7))))

(:wat::core::defn :user::template [] -> :wat::WatAST
  (:wat::core::quasiquote
    (:wat::core::do (:my::kernel::restricted-fn 7))))
