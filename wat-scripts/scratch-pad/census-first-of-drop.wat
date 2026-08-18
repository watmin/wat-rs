;; census-first-of-drop.wat — the HONEST count of the `(first (drop X n))` idiom.
;;
;; ⛔ WHY THIS EXISTS: stone 118.B4 quotes "44 hits across 13 files" and labels that number
;; A GREP, NOT A CENSUS, in its own text. Grep cannot tell a call from a comment, cannot see a
;; form broken across lines, and this project has been wrong five separate times counting
;; structured source with a pattern. B4-ii migrates every hit; a migration that works from a wrong
;; worklist leaves survivors that only surface when the wall goes up in B4-iii.
;; `[[feedback_three_boundary_errors_need_a_reader_not_a_fourth_pattern]]`
;;
;; ★ THE MATCH IS STRUCTURAL: a hit is a LIST whose head is the keyword `:wat::core::first` and
;; whose FIRST ARGUMENT is a LIST whose head is the keyword `:wat::core::drop`. A comment
;; mentioning either name cannot score, and neither can `(first xs)` beside an unrelated `drop`.
;;
;; It reports the RECEIVER and INDEX source text of each hit, because B4-ii's rewrite is
;; `(first (drop X n))` -> `(nth X n)` and the two operands are what the codemod must carry across.
;;
;; Usage (one EDN vector of paths on stdin):
;;   printf '["wat/service.wat" "wat/lint.wat"]\n' | ./target/release/wat \
;;     wat-scripts/scratch-pad/census-first-of-drop.wat

(:wat::core::defn :census::head-is?
  [form <- :wat::WatAST name <- :wat::core::String] -> :wat::core::bool
  (:wat::core::let
    [ch (:wat::core::into [] (:wat::core::ast->children form))]
    (:wat::core::if (:wat::core::empty? ch)
      false
      (:wat::core::= (:wat::core::ast->source (:wat::core::first ch)) name))))

;; Is THIS form `(first (drop X n))`? Head is `first`, arity 1, and that one arg is a `drop` call.
(:wat::core::defn :census::is-first-of-drop?
  [form <- :wat::WatAST] -> :wat::core::bool
  (:wat::core::let
    [ch (:wat::core::into [] (:wat::core::ast->children form))]
    (:wat::core::if (:wat::core::< (:wat::core::length ch) 2)
      false
      (:wat::core::and
        (:census::head-is? form ":wat::core::first")
        (:census::head-is? (:wat::core::nth ch 1) ":wat::core::drop")))))

;; Report one hit as `receiver | index`, the two operands the codemod carries into `(nth X n)`.
(:wat::core::defn :census::report-hit
  [path <- :wat::core::String form <- :wat::WatAST] -> :wat::core::nil
  (:wat::core::let
    [inner (:wat::core::into [] (:wat::core::ast->children
             (:wat::core::nth (:wat::core::into [] (:wat::core::ast->children form)) 1)))]
    (:wat::kernel::println
      (:wat::core::string::concat
        (:wat::core::string::concat "  FIRST-OF-DROP  " path)
        (:wat::core::string::concat "  ::  "
          (:wat::core::if (:wat::core::< (:wat::core::length inner) 3)
            "<malformed drop — REPORT THIS, do not migrate it>"
            (:wat::core::string::concat
              (:wat::core::ast->source (:wat::core::nth inner 1))
              (:wat::core::string::concat " | "
                (:wat::core::ast->source (:wat::core::nth inner 2))))))))))

;; Walk every form beneath this one. A hit does not stop the descent — `(first (drop (first
;; (drop x 1)) 2))` is two hits and the codemod must see both.
(:wat::core::defn :census::walk
  [path <- :wat::core::String form <- :wat::WatAST] -> :wat::core::nil
  (:wat::core::do
    (:wat::core::if (:census::is-first-of-drop? form)
      (:census::report-hit path form)
      nil)
    (:wat::core::run!
      (:wat::core::fn [child <- :wat::WatAST] -> :wat::core::nil (:census::walk path child))
      (:wat::core::into [] (:wat::core::ast->children form)))))

(:wat::core::defn :census::file
  [path <- :wat::core::String] -> :wat::core::nil
  (:wat::core::do
    (:wat::kernel::println (:wat::core::string::concat "== " path))
    (:census::walk path
      (:wat::core::match (:wat::core::read-string (:wat::io::read-file path))
        ((:wat::core::ReadOutcome::Forms __forms) __forms)
        ((:wat::core::ReadOutcome::Malformed __cause)
          (:wat::kernel::assertion-failed! (:wat::core::Error/message __cause)
            :wat::core::None :wat::core::None))))))

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::run!
    (:wat::core::fn [p <- :wat::core::String] -> :wat::core::nil (:census::file p))
    (:wat::core::match (:wat::kernel::readln )
      ((:wat::kernel::ReadlnOutcome::Datum __datum) __datum)
      (:wat::kernel::ReadlnOutcome::Eof
        (:wat::kernel::assertion-failed! "readln: end of input" :wat::core::None :wat::core::None))
      (:wat::kernel::ReadlnOutcome::Stopped
        (:wat::kernel::assertion-failed! "readln: stop requested" :wat::core::None :wat::core::None)))))
