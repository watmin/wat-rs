;; probe-what-a-closure-costs.wat
;;
;; `transient means try again` costs +1065ms on publish+drain (my quiet-box runs:
;; 23819 -> 24884). The retry never fires on the happy path, so the SCORE's "no
;; timer on the happy path" is true and is NOT the cost. The visible suspect:
;; sqs.wat's send and ack arms each bind TWO closures (`nap`, `once-put`) in a
;; let BEFORE the outcome is known — ~16000 sends+acks per circuit run, all
;; discarded on the happy path.
;;
;; 1065ms / 16000 ops = ~66us per op, close to a whole 143us round trip. That is
;; suspiciously large for two allocations, so MEASURE it rather than assert it.
;; If a closure costs ~1us, the hypothesis is dead and the cost is elsewhere.

(:wat::config::set-redef! true)

(:wat::core::defn :cc::now [] -> :wat::core::i64
  (:wat::time::epoch-nanos (:wat::time::now)))

;; Body A — bind two closures per iteration, never call them. The sqs.wat shape.
(:wat::core::defn :cc::step-with [acc <- :wat::core::i64 i <- :wat::core::i64] -> :wat::core::i64
  (:wat::core::let
    [nap  (:wat::core::fn [] -> :wat::core::nil nil)
     once (:wat::core::fn [x <- :wat::core::i64] -> :wat::core::bool
            (:wat::i64::> x 0))]
    (:wat::i64::+ acc 1)))

;; Body B — identical fold, no closures.
(:wat::core::defn :cc::step-without [acc <- :wat::core::i64 i <- :wat::core::i64] -> :wat::core::i64
  (:wat::i64::+ acc 1))

(:wat::core::defn :cc::run [] -> :wat::core::String
  (:wat::core::let
    [xs    (:wat::core::range 0 16000)
     n     (:wat::core::count xs)
     w0    (:cc::now)
     _a    (:wat::core::foldl :cc::step-with 0 xs)
     w1    (:cc::now)
     _b    (:wat::core::foldl :cc::step-without 0 xs)
     w2    (:cc::now)
     with-ns    (:wat::i64::- w1 w0)
     without-ns (:wat::i64::- w2 w1)]
    (:wat::core::format
      "n={n};with-closures-us={a};without-closures-us={b};delta-us={d};per-op-ns={p}"
      :n n
      :a (:wat::i64::/ with-ns 1000)
      :b (:wat::i64::/ without-ns 1000)
      :d (:wat::i64::/ (:wat::i64::- with-ns without-ns) 1000)
      :p (:wat::i64::/ (:wat::i64::- with-ns without-ns) n))))

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::kernel::println (:cc::run)))
