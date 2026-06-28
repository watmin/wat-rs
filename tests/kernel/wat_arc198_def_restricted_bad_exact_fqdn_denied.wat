;; Negative fixture for wat_arc198_def_restricted.rs test 3 (exact FQDN denied).
;; Whitelist [:my::kernel::specific-caller] — :my::kernel::other-caller is a sibling, not the exact name.

(:wat::core::defn :my::kernel::restricted-fn
  {:restricted-to [:my::kernel::specific-caller]}
  [x <- :wat::core::i64] -> :wat::core::i64 x)

(:wat::core::defn :my::kernel::other-caller [] -> :wat::core::i64
  (:my::kernel::restricted-fn 7))

