;; Positive fixture for wat_arc198_def_restricted.rs test 3 (exact FQDN allowed).
;; Whitelist [:my::kernel::specific-caller] — named caller matches exactly.

(:wat::core::defn :my::kernel::restricted-fn
  {:restricted-to [:my::kernel::specific-caller]}
  [x <- :wat::core::i64] -> :wat::core::i64 x)

(:wat::core::defn :my::kernel::specific-caller [] -> :wat::core::i64
  (:my::kernel::restricted-fn 7))

