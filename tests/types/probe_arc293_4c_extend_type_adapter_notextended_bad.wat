;; 293.4c negative fixture — a foreign type NOT taught to satisfy the surface.
;;
;; Expected: startup fails (check error — TypeMismatch).
;; `:wat::core::i64` is NOT extended to satisfy `:t::TaggedNeg`, so passing an i64
;; where `:t::TaggedNeg` is required must be rejected at check time.
;; Proves that surface satisfaction is a real check, not always-true (STOP-3 guard).

(:wat::core::defsurface :t::TaggedNeg
  :holder :wat::core::Struct
  :features [(tag [self <- :t::TaggedNeg] -> :wat::core::i64)])

;; String IS taught — but i64 is NOT (proves selectivity, not blanket acceptance).
(:wat::core::extend-type :wat::core::String :t::TaggedNeg
  (tag [self] -> :wat::core::i64 42))

(:wat::core::defn :t::tag-neg [s <- :t::TaggedNeg] -> :wat::core::i64 (:t::TaggedNeg/tag s))

;; This must fail: 99 is an i64, which does NOT have extend-type for :t::TaggedNeg.
(:wat::core::defn :t::bad-probe [] -> :wat::core::i64 (:t::tag-neg 99))
