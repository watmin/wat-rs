;; ★ GUARD — an EMPTY list `()`.
;;
;; Any implementation that reaches for `&items[1..]` to test the `:-` shape PANICS here:
;; `range start index 1 out of range for slice of length 0`. Measured on the first strike:
;; the compiler died with a Rust panic, EXIT=101.
;;
;; Clean main answers with a NAMED diagnostic instead — `BareLegacyUnitValue` (arc 179) —
;; and that is what must survive: a crash is not a diagnostic.
(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::kernel::println ()))
