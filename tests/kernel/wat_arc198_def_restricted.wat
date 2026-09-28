;; Co-located fixture for wat_arc198_def_restricted.rs — slurped via startup_beside(file!()).
;; Test 1 positive: caller inside allowed namespace (prefix match) passes.

(:wat::core::defn :my::kernel::restricted-fn
  {:restricted-to [:my::kernel::]}
  [x <- wat.type/i64] -> wat.type/i64 x)

(:wat::core::defn :my::kernel::caller [] -> wat.type/i64
  (:my::kernel::restricted-fn 7))

