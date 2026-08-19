;; probe-118B-memo-state-detector.wat — how many times does user code run per element?
;;
;; ⛔⛔ READ THIS BEFORE QUOTING THE NUMBER. IT PRINTS 5 IN TWO OPPOSITE WORLDS.
;;
;;     before 118.B2   memo ON  + three-call stdlib walk   ->  5   (the memo is HIDING 15)
;;     before 118.B2   memo OFF + three-call stdlib walk   -> 15   (the defect, visible)
;;     after  118.B3   memo GONE + single-force `next` walk ->  5   ← SAME NUMBER, OPPOSITE REASON
;;
;; So a `5` does NOT tell you the memo is present, and it never did tell you that on its own. Today
;; it means the WALK is single-force, which is what we want. This file's original header read "A
;; memo-off build that prints 5 has not had its memo removed" — true when written, and false the
;; moment stone 118.B2b finished migrating the stdlib onto `:wat::stream::next`. Stone 118.B3
;; deleted both memos and this header was rewritten in the same commit, as that stone required.
;; `[[feedback_state_what_the_instrument_can_see_before_quoting_it]]`
;;
;; ★ WHAT THIS INSTRUMENT ACTUALLY MEASURES, stated so it cannot drift again: the number of times
;; user code `f` runs while draining 5 elements. 5 means once per element. Any number above 5 means
;; something is forcing cells more than once — which, with no memo to absorb it, is now a real cost
;; paid by the user's function and not just by memory.
;;
;; ⚠ IT CANNOT SEE THE MEMO. Nothing in this probe reads substrate state; it counts side effects.
;; The instruments that WITNESS the memo's absence are the RETENTION SLOPE
;; (`probe-118B-dorun-retention-slope.wat`, driven across sizes — flat at ~0.4 B/elem after B3,
;; against 3,188 B/elem before) and `distinct` at n=8000 (SIGKILL at a 2G cap before B3, rc=0 after).
;;
;; MECHANISM: `f` prints one line per invocation; `into` drains the mapped stream.
;;
;; RUN: ./target/release/wat wat-scripts/scratch-pad/probe-118B-memo-state-detector.wat | wc -l
;; Needs a real program (primed stdio for println), so run it as a subprocess, not in-process.

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::let
    [v   (:wat::core::range 0 5)
     f   (:wat::core::fn [x <- :wat::core::i64] -> :wat::core::i64
           (:wat::core::do
             (:wat::kernel::println "FORCED")
             x))
     out (:wat::core::into [] (:wat::core::map f v))]
    (:wat::kernel::println (:wat::core::length out))))
