;; wat-scripts/bench/call-heavy.wat — call-heavy perf: three call shapes that exercise the
;; interpreter's name-resolution path (docs/arc/2026/10/name-resolution/BRIEF.md R0):
;;
;;   1. a NON-TAIL `fib` at n=27 — the Rust call stack grows with n (both recursive calls are
;;      wrapped in `+`, so neither is in tail position); every level allocates its own
;;      `Environment` via `EnvBuilder` and looks its own name up on the way back up.
;;   2. a SELF-TAIL loop of 1,000,000 iterations whose body is a `let` of six bindings that
;;      read one another before the self-tail call. TCO keeps the Rust stack flat; every
;;      iteration still builds and resolves names in a fresh six-binding `Environment`.
;;   3. 1,000,000 calls through a stored closure value — the closure's own `closed_env` (one
;;      captured binding) is walked on every application, independent of the caller's loop
;;      environment.
;;
;; Prints one checksum (the sum of the three sub-results) so a correctness mismatch between
;; before/after a runtime change shows up as a different number, not just a different time.
;;
;; Usage (no stdin; stdout = an EDN :perf::CallHeavyResult map):
;;   target/release/wat ./wat-scripts/bench/call-heavy.wat

(:wat::core::defrecord :perf::CallHeavyResult
  [fib-checksum      <- :wat::core::i64
   fib-ns            <- :wat::core::i64
   tailloop-checksum <- :wat::core::i64
   tailloop-ns       <- :wat::core::i64
   closure-checksum  <- :wat::core::i64
   closure-ns        <- :wat::core::i64
   checksum          <- :wat::core::i64])

(:wat::core::defn :perf::ns-between [t0 <- :wat::time::Instant  t1 <- :wat::time::Instant] -> :wat::core::i64
  (:wat::i64::- (:wat::time::epoch-nanos t1) (:wat::time::epoch-nanos t0)))

;; 1. non-tail fib — neither recursive call is in tail position (both are wrapped by `+`).
(:wat::core::defn :perf::fib [n <- :wat::core::i64] -> :wat::core::i64
  (:wat::core::if (:wat::i64::< n 2)
    n
    (:wat::i64::+ (:perf::fib (:wat::i64::- n 1)) (:perf::fib (:wat::i64::- n 2)))))

;; 2. self-tail loop — the recursive call is the LAST expression of the `let` body, in tail
;; position; TCO applies. Six bindings, each reading the previous.
(:wat::core::defn :perf::tail-loop [i <- :wat::core::i64  acc <- :wat::core::i64] -> :wat::core::i64
  (:wat::core::if (:wat::i64::= i 0)
    acc
    (:wat::core::let [a (:wat::i64::+ acc 1)
                       b (:wat::i64::+ a 1)
                       c (:wat::i64::+ b 1)
                       d (:wat::i64::+ c 1)
                       e (:wat::i64::+ d 1)
                       g (:wat::i64::+ e 1)]
      (:perf::tail-loop (:wat::i64::- i 1) g))))

;; 3. closure-loop — `f` is a stored closure value, applied once per iteration. Self-tail.
(:wat::core::defn :perf::closure-loop
  [i <- :wat::core::i64
   acc <- :wat::core::i64
   f <- :wat::core::Fn(wat::core::i64)->wat::core::i64]
  -> :wat::core::i64
  (:wat::core::if (:wat::i64::= i 0)
    acc
    (:perf::closure-loop (:wat::i64::- i 1) (:wat::i64::+ acc (f i)) f)))

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::let [base     7
                    bump     (:wat::core::fn [x <- :wat::core::i64] -> :wat::core::i64 (:wat::i64::+ x base))
                    f0       (:wat::time::now)
                    fib-sum  (:perf::fib 27)
                    f1       (:wat::time::now)
                    fib-ns   (:perf::ns-between f0 f1)
                    t0       (:wat::time::now)
                    tail-sum (:perf::tail-loop 1000000 0)
                    t1       (:wat::time::now)
                    tail-ns  (:perf::ns-between t0 t1)
                    c0       (:wat::time::now)
                    clo-sum  (:perf::closure-loop 1000000 0 bump)
                    c1       (:wat::time::now)
                    clo-ns   (:perf::ns-between c0 c1)
                    total    (:wat::i64::+ (:wat::i64::+ fib-sum tail-sum) clo-sum)]
    (:wat::kernel::println
      (:perf::CallHeavyResult
        :fib-checksum fib-sum
        :fib-ns fib-ns
        :tailloop-checksum tail-sum
        :tailloop-ns tail-ns
        :closure-checksum clo-sum
        :closure-ns clo-ns
        :checksum total))))
