;; 251.8d-i — a :restricted-to entry that is neither keyword nor symbol
;; is a hard error (the silent drop is how the post-flip empty-whitelist
;; hazard was built).

(:wat::core::defn :my::kernel::restricted-fn
  {:restricted-to [42]}
  [x <- wat.type/i64] -> wat.type/i64 x)

(:wat::core::defn :my::kernel::caller [] -> wat.type/i64
  (:my::kernel::restricted-fn 7))
