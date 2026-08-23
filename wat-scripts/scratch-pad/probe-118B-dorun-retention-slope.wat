;; probe-118B-dorun-retention-slope.wat — stone 118.B lair probe #2 (THE ACCEPTANCE INSTRUMENT)
;;
;; This file exists so the number is REPRODUCIBLE rather than living in a session tmp dir
;; (`[[feedback_an_instrument_must_outlive_the_number_it_produced]]`).
;;
;; WHAT IT MEASURES: peak RSS of a walk whose OWN retention is O(1) — a `reduce +` accumulating a
;; single i64. Everything above that floor is the STREAM CHAIN being retained, which is the exact
;; quantity stone B is about.
;;
;; Two deliberate instrument choices, each closing a way this probe could lie:
;;
;;   1. THE SOURCE IS UNBOUNDED (`:probe::counter`) — no backing container is materialized, so the
;;      number is not polluted by the source. `(range 0 n)` would have allocated n i64s before the
;;      walk began, and that allocation would have swamped the signal.
;;   2. THE ACCUMULATOR IS ONE i64, NOT A VECTOR. An earlier draft of this probe proved
;;      non-vacuity with `(length (into [] …))` — which peaks at the full materialized Vector and
;;      would have MASKED the very thing being measured. maxRSS is a PEAK: any scaffolding that
;;      allocates more than the subject makes the instrument report its own scaffolding.
;;      `[[feedback_a_probe_that_recalibrates_under_load_measures_nothing]]`
;;
;; NON-VACUITY IS THE PRINTED SUM, and it is exact, not merely non-zero: summing 0..n-1 must equal
;; n*(n-1)/2. A walk that silently stopped early, or never ran, cannot print that number. So one
;; line of output proves the walk happened AND that it visited exactly n elements.
;;   n = 100000 -> 4999950000        n = 200000 -> 19999900000
;;   n = 400000 -> 79999800000       n = 800000 -> 319999600000
;;
;; WHY THE SLOPE IS NOT ZERO TODAY: every forced cell's `forced: OnceLock` memo links that cell to
;; its tail, so the head pins the whole realized chain. Note there are TWO memo'd cell kinds —
;; `LazyCell` (src/stream/mod.rs:66, a wat closure) and `NativeLazyCell` (:124, a Rust closure) —
;; and `reduce-stream`'s three-call `empty?`/`first`/`rest` walk is why the memo is load-bearing
;; and cannot simply be deleted first.
;;
;; ACCEPTANCE (stone B): this slope goes to ~0 B/element. ⚠ THAT IS A PREDICTION, NOT A RESULT.
;; The last prediction in this area — that deleting the memo alone reaches O(1) — was WRONG; it
;; reached eager parity. B measures; it does not assume.
;;
;; DRIVE IT: edit `n` into a temp copy per size and run each under
;;   /usr/bin/time -f 'maxRSS=%M KB'
;; Read the SLOPE across sizes, never a single point — one point cannot show a slope, and the
;; interpreter's own fixed footprint is a large constant that a single point would hide inside.

;; An UNBOUNDED source. Nothing is materialized: each cell is produced on force, and its tail is a
;; fresh thunk. `take` bounds it; `reduce` drains it.
(:wat::core::defn :probe::counter
  [i <- :wat::core::i64] -> (:wat::stream::Stream :- [:wat::core::i64])
  (:wat::stream::lazy
    (:wat::stream::cons i (:probe::counter (:wat::core::+ i 1)))))

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::let
    [n   100000
     sum (:wat::core::reduce
           (:wat::core::fn [acc <- :wat::core::i64 x <- :wat::core::i64] -> :wat::core::i64
             (:wat::core::+ acc x))
           0
           (:wat::core::take (:probe::counter 0) n))]
    (:wat::kernel::println sum)))
