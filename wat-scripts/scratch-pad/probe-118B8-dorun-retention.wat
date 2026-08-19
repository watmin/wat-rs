;; probe-118B8-dorun-retention.wat — stone 118.B8 acceptance instrument #1, POINTED AT `dorun`
;; ITSELF (not `reduce`). Adapted from `probe-118B-dorun-retention-slope.wat` — same two guards
;; against a lying instrument, carried over verbatim:
;;
;;   1. THE SOURCE IS UNBOUNDED (`:probe::counter`) — no backing container is materialized, so the
;;      number is not polluted by the source.
;;   2. dorun's own contract is O(1) retention (it returns nil) — so unlike the `reduce` probe we
;;      do not need a hand-picked O(1) accumulator; dorun IS the O(1) accumulator under test.
;;
;; NON-VACUITY: `f` (applied via `map` ahead of `dorun`) prints ONLY when it sees the last element
;; (x == n-1). A single "SAW n-1" line proves the walk reached the far end — dorun cannot print
;; that line by no-oping or stopping early. This mirrors the exact-sum discipline of the original
;; probe without requiring a return value from `dorun` (which returns nil by contract).
;;
;; DRIVE IT: edit `n` per size, run each under `/usr/bin/time -f 'maxRSS=%M KB'`.

(:wat::core::defn :probe::counter
  [i <- :wat::core::i64] -> :wat::stream::Stream<wat::core::i64>
  (:wat::stream::lazy
    (:wat::stream::cons i (:probe::counter (:wat::core::+ i 1)))))

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::let
    [n 100000
     f (:wat::core::fn [x <- :wat::core::i64] -> :wat::core::i64
         (:wat::core::if (:wat::core::= x (:wat::core::- n 1))
           (:wat::core::do (:wat::kernel::println x) x)
           x))]
    (:wat::core::dorun (:wat::core::map f (:wat::core::take (:probe::counter 0) n)))))
