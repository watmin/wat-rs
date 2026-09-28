;; Positive fixture for wat_arc198_def_restricted.rs test 4 (multi-prefix whitelist).
;; Whitelist [:my::kernel:: :my::test::] admits callers in either namespace.

(:wat::core::defn :my::kernel::restricted-fn
  {:restricted-to [:my::kernel:: :my::test::]}
  [x <- wat.type/i64] -> wat.type/i64 x)

(:wat::core::defn :my::kernel::kernel-caller [] -> wat.type/i64
  (:my::kernel::restricted-fn 1))

(:wat::core::defn :my::test::test-caller [] -> wat.type/i64
  (:my::kernel::restricted-fn 2))

