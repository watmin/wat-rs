;; Board specimen — the-little-wat F-031: `length` on a String.
;;
;; SHAPE: LENIENT — the checker ACCEPTS this program and the runtime KILLS it.
;; ⛔ Do NOT "fix" this file. The file is correct AS A SPECIMEN; the finding is that
;; wat accepts it. Editing it to run cleanly does not close F-031, it blinds the board.
(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::kernel::println (:wat::core::length "abc")))
