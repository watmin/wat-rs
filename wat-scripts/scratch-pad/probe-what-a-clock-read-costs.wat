;; probe-what-a-clock-read-costs.wat — arc 278.
;;
;; WHY. Separating "the store's SQL work" from "waiting for the store actor"
;; needs a clock pair around each store op. At n=2000 the circuit makes ~10,124
;; store round trips, so ~20,000 extra clock reads would land INSIDE the phase
;; under measurement. `the queue counts its store round trips` deferred timing
;; on exactly that objection (STOP-5), and the objection was never priced.
;;
;; This prices it. If a read is ~1us, 20,000 reads is ~20ms against a 3,200ms
;; drain — 0.6%, and the objection dissolves. If it is ~50us, 20,000 reads is
;; ~1s of a 3,200ms drain — 31%, and per-op timing is not available at all.
;;
;; ⚠ NOT `await-timer-ms`, which the arc already measured at 1269us. That is a
;; timer arm + fire + select wake. This is the bare clock read.
;;
;; ⚠ The accumulator takes `rem 1000` of each reading: summing raw epoch-nanos
;; overflows i64 within a few thousand iterations (first draft did, and said so).
(:wat::core::defn :probe::read-n [n <- :wat::core::i64] -> :wat::core::i64
  (:wat::core::foldl
    (:wat::core::fn [acc <- :wat::core::i64  _i <- :wat::core::i64] -> :wat::core::i64
      (:wat::i64::+ acc
        (:wat::i64::rem (:wat::time::epoch-nanos (:wat::time::now)) 1000)))
    0
    (:wat::core::range 0 n)))

(:wat::core::defn :probe::bare-n [n <- :wat::core::i64] -> :wat::core::i64
  (:wat::core::foldl
    (:wat::core::fn [acc <- :wat::core::i64  i <- :wat::core::i64] -> :wat::core::i64
      (:wat::i64::+ acc (:wat::i64::rem i 1000)))
    0
    (:wat::core::range 0 n)))

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::let
    [n 100000
     ;; the fold itself costs something; measure it and subtract
     b0 (:wat::time::epoch-nanos (:wat::time::now))
     _b (:probe::bare-n n)
     b1 (:wat::time::epoch-nanos (:wat::time::now))
     t0 (:wat::time::epoch-nanos (:wat::time::now))
     _t (:probe::read-n n)
     t1 (:wat::time::epoch-nanos (:wat::time::now))
     bare-ns  (:wat::i64::- b1 b0)
     total-ns (:wat::i64::- t1 t0)
     clock-ns (:wat::i64::- total-ns bare-ns)
     per-read (:wat::i64::/ clock-ns n)
     cost-20k-us (:wat::i64::/ (:wat::i64::* per-read 20000) 1000)]
    (:wat::kernel::println
      (:wat::core::format
        "n={n} bare={b}ns withclock={t}ns clock-only={c}ns per-read={p}ns 20k-reads={k}us"
        :n n :b bare-ns :t total-ns :c clock-ns :p per-read :k cost-20k-us))))
