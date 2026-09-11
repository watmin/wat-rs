;; Negative control for a-give-up-has-no-form-for-attempts.
;;
;; circuit.wat cannot be load-file!'d: it holds `set-redef!` (entry-file only;
;; SetterInLoadedFile). This file copies Verdict + require! byte-for-byte from
;; circuit.wat so the consumer is the same match. Type-checked by
;; every_wat_scripts_file_loads (parse + check, does not run main).
;;
;; Run:
;;   ./target/release/wat wat-scripts/scratch-pad/probe-verdict-stalled.wat
;; Expected: raise; message names 600 polls and elapsed=42.

(:wat::core::defenum :fanout::Verdict :wat::enum::Pure
  :Done    []
  :Stalled [no-progress-polls <- :wat::core::i64
            elapsed-ms        <- :wat::core::i64
            snapshot          <- :wat::core::String]
  :Ceiling [elapsed-ms <- :wat::core::i64
            ceiling-ms <- :wat::core::i64
            snapshot   <- :wat::core::String])

(:wat::core::defn :fanout::require!
  [v <- :fanout::Verdict] -> :wat::core::nil
  (:wat::core::match v
    ((:fanout::Verdict::Done) nil)
    ((:fanout::Verdict::Stalled _k _ms snap)
      (:wat::kernel::assertion-failed! snap :wat::core::None :wat::core::None))
    ((:fanout::Verdict::Ceiling _ms _cap snap)
      (:wat::kernel::assertion-failed! snap :wat::core::None :wat::core::None))))

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:fanout::require!
    (:fanout::Verdict::Stalled 600 42
      "filled-stalled: no arrival progress in 600 polls; last=[31/0] elapsed=42")))
