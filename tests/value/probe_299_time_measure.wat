;; tests/value/probe_299_time_measure.wat — arc 299.2, the TIME entropic measurement.
;; `now` is wall-clock ENTROPY (a syscall) — its value can't be pinned, only BOUNDED.
;; The orchestrator (Rust) brackets [lo,hi] around the call; wat measures the value
;; fell inside the window and is after epoch 0. Entropy conformed, not equated.

;; Rust reads wat's `now` — the entropy source itself must come FROM wat (the claim being
;; measured), not be constructed on the Rust side.
(:wat::core::defn :probe::now-nanos [] -> :wat::core::i64
  (:wat::time::epoch-nanos (:wat::time::now)))

(:wat::core::defn :probe::measure
    [t <- :wat::time::Instant  lo <- :wat::core::i64  hi <- :wat::core::i64] -> :wat::core::bool
  (:wat::core::let [ns (:wat::time::epoch-nanos t)]
    (:wat::core::and (:wat::i64::> ns 0)
      (:wat::core::and (:wat::i64::>= ns lo)
                       (:wat::i64::<= ns hi)))))
