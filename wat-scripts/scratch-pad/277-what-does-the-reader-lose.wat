;; 277-what-does-the-reader-lose.wat — "what don't we know when we read the file?"
;; Builder's question, 2026-09-05. Answered by ROUND-TRIPPING, not by reading the printer.
;; Each row prints  ORIGINAL -> (ast->source (read-string ORIGINAL)).  Identical = nothing lost.

(:wat::core::defn :rt::show [src <- :wat::core::String] -> :wat::core::nil
  (:wat::core::match (:wat::core::read-string src)
    ((:wat::core::ReadOutcome::Forms forms)
      (:wat::core::let [out (:wat::core::ast->source forms)]
        (:wat::kernel::println (:wat::string::interpolate
          "{v}  in={i}   out={o}"
          :v (:wat::core::if (:wat::core::= src out) "SAME" " ***") :i src :o out))))
    ((:wat::core::ReadOutcome::Malformed c)
      (:wat::kernel::println (:wat::string::concat "UNREADABLE  " src)))))

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::do
    (:rt::show "(a b)")
    (:rt::show "3.0")
    (:rt::show "3.00")
    (:rt::show "1e3")
    (:rt::show "42")
    (:rt::show "\"hi\"")
    (:rt::show "\"\\u0041\"")
    (:rt::show ":wat::core::defn")
    (:rt::show "[x <- :wat::core::i64]")
    (:rt::show "(a  b)")
    (:rt::show "(a\n  b)")
    (:rt::show ";; a comment\n(a b)")
    (:rt::show "(a b) ;; trailing")))
