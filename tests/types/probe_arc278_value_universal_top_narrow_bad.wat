;; probe_arc278_value_universal_top_narrow_bad.wat — Surface B narrow (negative). Must FAIL.
;; A :wat::core::Value is NOT assignable where :wat::core::i64 is expected.

(:wat::core::defn :my::needs-int [n <- :wat::core::i64] -> :wat::core::i64 n)
(:wat::core::defn :my::down [v <- :wat::core::Value] -> :wat::core::i64 (:my::needs-int v))
