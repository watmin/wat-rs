;; probe-clause-tco-ensure-still-fires.wat — ★ THE ROW THAT DECIDES THE STONE.
;;
;; `:ensure` is a POST-condition: it runs AFTER the body (runtime.rs:8364's own doc).
;; A tail call abandons the calling frame, so there is no frame left to run it in.
;; Therefore an ensure-bearing clause MUST NOT be tail-called — and this probe proves
;; the exclusion is real rather than accidental.
;;
;; The clause RECURSES (so it is exactly the shape that would be tail-called if the
;; stone were careless) and returns -1, which its `:ensure` rejects.
;; MUST RAISE — before AND after. A silent success here means the stone deleted a
;; post-condition the author wrote and the checker promised: a worse hole than the
;; stack exhaustion it set out to fix.
(:wat::core::defclause :probe::never-negative
  ([n <- :wat::core::i64] -> :wat::core::i64
    :ensure (:wat::core::fn [r <- :wat::core::i64] -> :wat::core::bool
              (:wat::core::i64::>= r 0))
    (:wat::core::if (:wat::core::= n 0)
      -1
      (:probe::never-negative (:wat::core::- n 1)))))
(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::kernel::println (:probe::never-negative 10)))
