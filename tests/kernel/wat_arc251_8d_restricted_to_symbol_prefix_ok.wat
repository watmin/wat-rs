;; 251.8d-i — :restricted-to accepts a namespace SYMBOL (no `/`).
;; `my.kernel` admits any caller in that namespace.

(:wat::core::defn :my::kernel::restricted-fn
  {:restricted-to [my.kernel]}
  [x <- wat.type/i64] -> wat.type/i64 x)

(:wat::core::defn :my::kernel::caller [] -> wat.type/i64
  (:my::kernel::restricted-fn 7))
