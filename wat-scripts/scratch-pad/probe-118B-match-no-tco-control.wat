;; probe-118B-match-no-tco-control.wat — stone 118.B lair probe #1's NON-VACUITY CONTROL
;;
;; Sibling of `probe-118B-match-tco-drain.wat`. Same `match` form, same 200,000-element depth —
;; the ONLY difference is that the recursive call sits in ARGUMENT position (`(+ 1 (recur …))`)
;; instead of tail position, so the TCO trampoline cannot apply.
;;
;; THIS PROBE IS EXPECTED TO DIE. That is its whole job: it proves 200,000 is deep enough for a
;; missing tail position to be DETECTED. Without it, the tail probe's "prints 200000" could mean
;; "match carries a tail position" OR "200,000 frames happen to fit" — and those are different
;; worlds. `[[feedback_a_green_test_can_prove_nothing]]`: a probe that never exercises the
;; mechanism returns a meaningless green.
;;
;; Per tasks #58/#86 the expected death is a SILENT SIGSEGV (signal 11), not a located wat raise.
;; A clean raise would ALSO be an acceptable control outcome — what must not happen is that it
;; prints 200000.

;; Deliberately NON-tail: the recursion is an argument to `+`.
(:wat::core::defn :probe::count-deep
  [s <- (:wat::stream::Stream :- [:wat::core::i64])] -> :wat::core::i64
  (:wat::core::match (:wat::stream::next s)
    ((:wat::stream::NextOutcome::Item value rest)
      (:wat::core::+ 1 (:probe::count-deep rest)))
    (:wat::stream::NextOutcome::Exhausted 0)))

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::let
    [n 200000
     s (:wat::core::map
         (:wat::core::fn [x <- :wat::core::i64] -> :wat::core::i64 x)
         (:wat::core::range 0 n))]
    (:wat::kernel::println (:probe::count-deep s))))
