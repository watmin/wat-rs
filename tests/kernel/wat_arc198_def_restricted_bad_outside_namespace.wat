;; Negative fixture for wat_arc198_def_restricted.rs test 2 (and test 5 negative).
;; Caller :user::app::caller is OUTSIDE the :my::kernel:: prefix → startup fails.

(:wat::core::defn :my::kernel::restricted-fn
  {:restricted-to [:my::kernel::]}
  [x <- wat.type/i64] -> wat.type/i64 x)

(:wat::core::defn :user::app::caller [] -> wat.type/i64
  (:my::kernel::restricted-fn 7))

