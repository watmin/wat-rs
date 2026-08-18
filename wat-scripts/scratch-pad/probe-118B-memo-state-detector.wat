;; probe-118B-memo-state-detector.wat — is the WHNF memo ON or OFF in this binary?
;;
;; This is the NON-VACUITY CONTROL for the memo-off measurement. Its whole job is to prove that a
;; throwaway "memo bypassed" build ACTUALLY bypassed it. Without this, a build where the edit
;; silently did nothing produces memory numbers that look exactly like a successful intervention —
;; and the conclusion would be drawn from an unchanged substrate.
;; `[[feedback_a_green_test_can_prove_nothing]]`
;;
;; MECHANISM: `f` prints one line per invocation. The drain is `into`, which walks each cell with the
;; three-call protocol (`empty?` -> `first` -> `rest`) via `stream->pvec`. So:
;;
;;     memo ON   ->  5 lines   (the three calls share one force)
;;     memo OFF  -> 15 lines   (three independent forces per cell, user code 3x)
;;
;; 15 lines is not a bug in this probe — it IS the defect the whole arc exists to remove, made
;; visible. A memo-off build that prints 5 has not had its memo removed.
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
