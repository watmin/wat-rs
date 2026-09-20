;; 251.8d-i — :restricted-to accepts an exact-FQDN SYMBOL (has `/`).
;; `my.kernel/specific-caller` admits only that caller.

(:wat::core::defn :my::kernel::restricted-fn
  {:restricted-to [my.kernel/specific-caller]}
  [x <- :wat::core::i64] -> :wat::core::i64 x)

(:wat::core::defn :my::kernel::specific-caller [] -> :wat::core::i64
  (:my::kernel::restricted-fn 7))
