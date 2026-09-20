;; 251.8d-i — :restricted-to accepts a namespace SYMBOL (no `/`).
;; `my.kernel` admits any caller in that namespace.

(:wat::core::defn :my::kernel::restricted-fn
  {:restricted-to [my.kernel]}
  [x <- :wat::core::i64] -> :wat::core::i64 x)

(:wat::core::defn :my::kernel::caller [] -> :wat::core::i64
  (:my::kernel::restricted-fn 7))
