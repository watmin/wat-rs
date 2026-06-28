;; tests/wat_lang/probe_nil_return_value_position_bug.wat
;; Arc 242 disconfirming repro — return-type position vs value-position confusion.
;; Combined fixture: startup must succeed for all four test cases.

;; Case 1: nil return type with bare nil body
(:wat::core::defn :t::nil-bare-nil [] -> :wat::core::nil nil)

;; Case 2: i64 return type with bare int body (class-characterizer)
(:wat::core::defn :t::i64-bare-int [] -> :wat::core::i64 42)

;; Cases 3+4: defclause + nil-typed defns (the triggering combination)
(:wat::core::defclause :my::label
  ([x <- :wat::core::i64] -> :wat::core::String "i64")
  ([x <- :wat::core::f64] -> :wat::core::String "f64"))
(:wat::core::defn :t::compute [] -> :wat::core::String (:my::label 42))
