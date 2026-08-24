;; BENCH — what would collapsing `reduce` to ONE `(Seqable :- [T])` clause COST on an eager container?
;;
;; ⛔ WHY THIS EXISTS. Stones B2c/B2d opened the doors that let a multi-arity verb live as one
;; clause over `(Seqable :- [T])`. `reductions`' ten arms are pure duplication — every one delegates to
;; the same walker — so collapsing it is free. **`reduce` is NOT that shape.** Its eager arms
;; delegate to `:wat::core::foldl`, a NATIVE intrinsic (dispatched at src/runtime.rs:6354).
;; Collapsing `reduce` would route every eager reduce through the interpreted `reduce-walk`
;; instead. That is a different axis from surface-dispatch overhead, and it must be measured
;; before it is traded away.
;;
;; ⚠ READ `bench-surface-dispatch-cost.wat` BEFORE CITING THIS NUMBER. Its ruling stands and this
;; bench does not contradict it: *"DO NOT cite this bench to argue against surfaces, Seqable, or
;; extend-type … interpretted wat has a death sentence."* This is NOT a measurement of dispatch and
;; NOT an argument against `Seqable`. It measures ONE thing: losing a native fold. If a compiler
;; later consumes the surface, the walker arm is exactly what it compiles, and this number dies
;; with the interpreter.
;;
;; Shape discipline (`[[feedback_a_benchmarks_shape_manufactures_its_result]]`): fixed n, BOTH
;; block orderings, and a non-vacuity control proving all four arms compute the SAME sum. No
;; recalibration inside the run.

;; ARM A — what `reduce` does TODAY for a Vector: the native foldl intrinsic.
(:wat::core::defn :bench::via-foldl [v <- (:wat::core::Vector :- [:wat::core::i64])] -> :wat::core::i64
  (:wat::core::foldl (:wat::core::fn [acc <- :wat::core::i64 x <- :wat::core::i64] -> :wat::core::i64
                       (:wat::core::i64::+ acc x))
    0 v))

;; ARM B — what a collapsed `reduce` would do: normalise to a Stream, then walk it interpreted.
(:wat::core::defn :bench::via-walk [v <- (:wat::core::Vector :- [:wat::core::i64])] -> :wat::core::i64
  (:wat::core::reduce-walk (:wat::core::fn [acc <- :wat::core::i64 x <- :wat::core::i64] -> :wat::core::i64
                             (:wat::core::i64::+ acc x))
    0 (:wat::core::Seqable/seq v)))

(:wat::core::defn :bench::ns [t0 <- :wat::time::Instant t1 <- :wat::time::Instant] -> :wat::core::i64
  (:wat::core::i64::- (:wat::time::epoch-nanos t1) (:wat::time::epoch-nanos t0)))

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::let
    [n  200000
     v  (:wat::core::into (:wat::core::Vector :wat::core::i64) (:wat::core::range 0 n))
     ;; ORDER A: walk first, then foldl
     a0 (:wat::time::now) ra (:bench::via-walk v)  a1 (:wat::time::now)
     b0 (:wat::time::now) rb (:bench::via-foldl v) b1 (:wat::time::now)
     ;; ORDER B: reversed — the block-ordering control
     c0 (:wat::time::now) rc (:bench::via-foldl v) c1 (:wat::time::now)
     d0 (:wat::time::now) rd (:bench::via-walk v)  d1 (:wat::time::now)]
    (:wat::kernel::println
      (:wat::core::string::interpolate
        "n={n} NONVACUITY ra={ra} rb={rb} rc={rc} rd={rd} | A: walk={ad}ms foldl={bd}ms | B: foldl={cd}ms walk={dd}ms"
        :n n :ra ra :rb rb :rc rc :rd rd
        :ad (:wat::core::i64::/ (:bench::ns a0 a1) 1000000)
        :bd (:wat::core::i64::/ (:bench::ns b0 b1) 1000000)
        :cd (:wat::core::i64::/ (:bench::ns c0 c1) 1000000)
        :dd (:wat::core::i64::/ (:bench::ns d0 d1) 1000000)))))
