;; census-parametric-surface-bindings.wat — the BLAST RADIUS instrument for stone 118.B2d.
;;
;; ⛔ WHY: B2d wants to change `src/check.rs`'s parametric-surface member resolution — and that path
;; is taken by EVERY parametric surface's method call, not just `Seqable`'s. The stone's own gate
;; says the blast radius must be MEASURED, not guessed. This is that measurement.
;;
;; THE QUESTION IT ANSWERS: for each `defsurface` and each `extend-type`, what is the protocol
;; target's spelling? A satisfier that binds the surface's params to CONCRETE types
;; (`(:Holds :- [wat::core::i64])`) takes path (1) and works today. A satisfier that binds them to
;; VARIABLES (`(:Seqable :- [T])`) is the broken class — the stored scheme's return keeps a free `T`.
;;
;; ★ IT PRINTS RAW SOURCE AND DOES NOT CLASSIFY. wat's string library has no substring/split, so any
;; in-language classifier here would be exactly the kind of hand-rolled text rule that produced three
;; wrong counts before the form-tree census existed. The instrument's job is EXHAUSTIVE ENUMERATION
;; WITH CORRECT BOUNDARIES — which is the part grep cannot do. The rows are few and are classified by
;; reading, with the raw text on the record so the classification is auditable.
;; `[[feedback_three_boundary_errors_need_a_reader_not_a_fourth_pattern]]`
;;
;; Walks RECURSIVELY: a `defsurface` can sit inside a `do` (arc-278 hoisting), so a top-level-only
;; scan would undercount.
;;
;; Usage (one EDN vector of paths on stdin):
;;   printf '["wat/seq.wat"]\n' | ./target/release/wat wat-scripts/scratch-pad/census-parametric-surface-bindings.wat

(:wat::core::defn :census::head-of [form <- :wat::WatAST] -> :wat::core::String
  (:wat::core::let [ch (:wat::core::ast->children form)]
    (:wat::core::if (:wat::core::empty? ch)
      ""
      (:wat::core::ast->source (:wat::core::first ch)))))

(:wat::core::defn :census::nth-source
  [form <- :wat::WatAST idx <- :wat::core::i64] -> :wat::core::String
  (:wat::core::let [ch (:wat::core::into [] (:wat::core::ast->children form))]
    (:wat::core::if (:wat::core::< (:wat::core::length ch) (:wat::core::+ idx 1))
      ""
      (:wat::core::ast->source (:wat::core::nth ch idx)))))

;; Report this form if it is a defsurface / extend-type, then recurse into every child.
(:wat::core::defn :census::walk [form <- :wat::WatAST] -> :wat::core::nil
  (:wat::core::do
    (:wat::core::let [h (:census::head-of form)]
      (:wat::core::if (:wat::core::= h ":wat::core::defsurface")
        (:wat::kernel::println
          (:wat::string::concat "  SURFACE     " (:census::nth-source form 1)))
        (:wat::core::if (:wat::core::= h ":wat::core::extend-type")
          (:wat::kernel::println
            (:wat::string::concat
              (:wat::string::concat
                (:wat::string::concat "  EXTEND      " (:census::nth-source form 1))
                "   AS   ")
              (:census::nth-source form 2)))
          nil)))
    (:wat::core::run!
      (:wat::core::fn [c <- :wat::WatAST] -> :wat::core::nil (:census::walk c))
      (:wat::core::into [] (:wat::core::ast->children form)))))

(:wat::core::defn :census::file [path <- :wat::core::String] -> :wat::core::nil
  (:wat::core::run!
    (:wat::core::fn [f <- :wat::WatAST] -> :wat::core::nil
      (:census::walk f))
    (:wat::core::into []
      (:wat::core::ast->children
        (:wat::core::match (:wat::core::read-string (:wat::io::read-file path))
          ((:wat::core::ReadOutcome::Forms __forms) __forms)
          ((:wat::core::ReadOutcome::Malformed __cause)
            (:wat::kernel::assertion-failed! (:wat::core::Error/message __cause)
              :wat::core::None :wat::core::None)))))))

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::run!
    (:wat::core::fn [p <- :wat::core::String] -> :wat::core::nil (:census::file p))
    (:wat::core::match (:wat::kernel::readln )
      ((:wat::kernel::ReadlnOutcome::Datum __datum) __datum)
      (:wat::kernel::ReadlnOutcome::Eof
        (:wat::kernel::assertion-failed! "readln: end of input" :wat::core::None :wat::core::None))
      (:wat::kernel::ReadlnOutcome::Stopped
        (:wat::kernel::assertion-failed! "readln: stop requested" :wat::core::None :wat::core::None)))))
