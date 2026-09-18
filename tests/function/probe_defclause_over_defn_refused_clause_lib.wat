;; The twice-loaded library. Its defclause re-registers on the second load, and the
;; stub the FIRST load left behind must be recognized as this same form's own.
(:wat::core::defclause :libc::twice ([n <- :wat::core::i64] -> :wat::core::i64 (:wat::core::* n 2)))
