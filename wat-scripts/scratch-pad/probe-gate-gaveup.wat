;; Negative control for the-gate-methods-face-an-outcome.
;; Hand-construct GateOutcome::GaveUp and pass it to require-granted.
;; Type-checked by every_wat_scripts_file_loads (parse + check, does not run main).
;;
;; Run:
;;   ./target/release/wat wat-scripts/scratch-pad/probe-gate-gaveup.wat
;; Expected: raise; message names waited-ms=10000 and last=TimedOut.

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::service::require-granted
    (:wat::service::GateOutcome::GaveUp 10000 "TimedOut")))
