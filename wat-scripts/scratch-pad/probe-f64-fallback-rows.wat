;; probe-f64-fallback-rows.wat — scorecard runner for BRIEF-f64-fallback-rows.md.
;;
;; Exercises the four new `:wat::rete::core::f64::{+,-,*,/}` Fallback rows and the
;; `dispatch_rete_op` arm's extended `Ok`-value check (NaN/±Inf take the fallback;
;; every ordinary finite float, including -0.0, passes through). Not a test file
;; itself (see `tests/rete` for the durable gate) — a loadable, type-checked
;; reference proving the scorecard's numbered claims by RUN.
;;
(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::let
    [;; row 2 — ordinary arithmetic untouched.
     r2 (:wat::rete::f64::+ 1.5 2.0 :undefined 0.0)
     ;; row 3 — NaN result takes the fallback.
     r3 (:wat::rete::f64::/ 0.0 0.0 :undefined -1.0)
     ;; row 4 — +Inf result takes the fallback.
     r4 (:wat::rete::f64::/ 1.0 0.0 :undefined -1.0)
     ;; o4 is a plain finite literal; o4*o4 (~1e400) overflows f64::MAX (~1.8e308)
     ;; to +Inf.
     o4 1e200
     ;; row 5 — overflow-to-Inf takes the fallback.
     r5 (:wat::rete::f64::* o4 o4 :undefined -1.0)
     ;; row 6 — SAME expression as row 5, DIFFERENT fallback value.
     r6a (:wat::rete::f64::* o4 o4 :undefined -1.0)
     r6b (:wat::rete::f64::* o4 o4 :undefined 42.0)
     ;; row 7 — -0.0 is finite, passes through (not the fallback).
     r7 (:wat::rete::f64::* 0.0 -1.0 :undefined 99.0)
     ;; row 9 — i64 rows still fall back (Part A must not regress the Err path).
     r9 (:wat::rete::i64::/ 1 0 :undefined -1)
     ;; row 14 — core is untouched, still raw IEEE, still exit 0.
     r14 (:wat::f64::/ 0.0 0.0)]
    (:wat::kernel::println (:wat::string::concat "row2  ordinary (expect 3.5): "   (:wat::core::str r2)))
    (:wat::kernel::println (:wat::string::concat "row3  NaN->fallback (expect -1.0): " (:wat::core::str r3)))
    (:wat::kernel::println (:wat::string::concat "row4  +Inf->fallback (expect -1.0): " (:wat::core::str r4)))
    (:wat::kernel::println (:wat::string::concat "row5  overflow->fallback (expect -1.0): " (:wat::core::str r5)))
    (:wat::kernel::println (:wat::string::concat "row6a fallback=-1.0 (expect -1.0): " (:wat::core::str r6a)))
    (:wat::kernel::println (:wat::string::concat "row6b fallback=42.0 (expect 42.0): " (:wat::core::str r6b)))
    (:wat::kernel::println (:wat::string::concat "row7  -0.0 passthrough (expect -0.0, not 99.0): " (:wat::core::str r7)))
    (:wat::kernel::println (:wat::string::concat "row9  i64 fallback still fires (expect -1): " (:wat::core::str r9)))
    (:wat::kernel::println (:wat::string::concat "row14 core untouched (expect nan): " (:wat::core::str r14)))))
