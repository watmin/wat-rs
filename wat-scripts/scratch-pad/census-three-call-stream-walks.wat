;; census-three-call-stream-walks.wat — THE INSTRUMENT THIS QUESTION EARNED.
;;
;; ⛔ WHY THIS EXISTS: I counted the three-call Stream walkers with grep THREE TIMES and was wrong
;; THREE TIMES, each with a DIFFERENT parsing flaw:
;;   1. `defn [a-z-]*-stream`      — missed namespaced heads (`defn :wat::core::reduce-stream`).
;;   2. sliced params at the first `->` — `stream->pvec` has `->` in its NAME, and
;;      `keep-indexed-walk` has one inside its `Fn(...)` type. Both silently vanished.
;;   3. took the FIRST `[` in a form — so a multi-arm `defclause` was judged by arm 1 only, and
;;      `reduce`'s Stream arms (arms 4 and 8) were invisible.
;; Every fix was real; every one left a different hole. The answer is not a fourth regex.
;;
;; Builder, 2026-08-18: *"wat-fix is... phenomenal... you can't miss what you know you're looking
;; for."* — this walks the REAL FORM TREE via `read-string` + `ast->children`, so form boundaries
;; come from the reader instead of from a pattern I guessed. All three failures above were BOUNDARY
;; errors; the reader cannot make them.
;;
;; ★ AND THE MATCH IS STRUCTURAL, not textual: a hit is a LIST WHOSE HEAD IS the keyword
;; `:wat::core::first` / `rest` / `empty?`. A comment mentioning `rest`, or a type named
;; `Stream<T>`, cannot score — which is what made grep's counts untrustworthy in both directions.
;;
;; Usage (one EDN vector of paths on stdin):
;;   printf '["wat/seq.wat"]\n' | ./target/release/wat wat-scripts/scratch-pad/census-three-call-stream-walks.wat
;;
;; Reports one line per UNIT (a `defn`, or ONE ARM of a `defclause`) whose parameter vector names
;; a `Stream<` AND whose body structurally calls the three-call protocol.

;; Does this form, or anything beneath it, CALL first/rest/empty? — head-position only.
(:wat::core::defn :census::walks?
  [form <- :wat::WatAST] -> :wat::core::bool
  (:wat::core::let
    [ch (:wat::core::ast->children form)
     h  (:wat::core::if (:wat::core::empty? ch)
          ""
          (:wat::core::ast->source (:wat::core::first ch)))]
    (:wat::core::if
      (:wat::core::or
        (:wat::core::or (:wat::core::= h ":wat::core::first")
                        (:wat::core::= h ":wat::core::rest"))
        (:wat::core::= h ":wat::core::empty?"))
      true
      (:wat::core::reduce
        (:wat::core::fn [acc <- :wat::core::bool f <- :wat::WatAST] -> :wat::core::bool
          (:wat::core::or acc (:census::walks? f)))
        false
        ch))))

;; A UNIT is `[<param-vector> <arrow> <ret> <body>…]` — the shape of a `defn` after its head+name,
;; and the shape of ONE `defclause` arm. Judging units, not forms, is what makes multi-arm
;; defclauses visible.
(:wat::core::defn :census::unit-hit?
  [unit <- (:wat::core::Vector :- [:wat::WatAST])] -> :wat::core::bool
  (:wat::core::if (:wat::core::empty? unit)
    false
    (:wat::core::and
      (:wat::core::string::contains? (:wat::core::ast->source (:wat::core::first unit))
                                     "stream::Stream<")
      (:wat::core::reduce
        (:wat::core::fn [acc <- :wat::core::bool f <- :wat::WatAST] -> :wat::core::bool
          (:wat::core::or acc (:census::walks? f)))
        false
        (:wat::core::into [] (:wat::core::rest unit))))))

(:wat::core::defn :census::report-unit
  [label <- :wat::core::String unit <- (:wat::core::Vector :- [:wat::WatAST])] -> :wat::core::nil
  (:wat::core::if (:census::unit-hit? unit)
    (:wat::kernel::println (:wat::core::string::concat "  THREE-CALL  " label))
    nil))

;; One top-level form → zero or more units.
(:wat::core::defn :census::form
  [form <- :wat::WatAST] -> :wat::core::nil
  (:wat::core::let
    [ch   (:wat::core::into [] (:wat::core::ast->children form))
     head (:wat::core::if (:wat::core::empty? ch) "" (:wat::core::ast->source (:wat::core::first ch)))
     name (:wat::core::if (:wat::core::< (:wat::core::length ch) 2)
            "" (:wat::core::ast->source (:wat::core::second ch)))
     tail (:wat::core::into [] (:wat::core::drop ch 2))]
    (:wat::core::if (:wat::core::= head ":wat::core::defclause")
      ;; each remaining child is an ARM — judge every one.
      (:wat::core::run!
        (:wat::core::fn [arm <- :wat::WatAST] -> :wat::core::nil
          (:census::report-unit name (:wat::core::into [] (:wat::core::ast->children arm))))
        tail)
      (:wat::core::if (:wat::core::= head ":wat::core::defn")
        (:census::report-unit name tail)
        nil))))

(:wat::core::defn :census::file
  [path <- :wat::core::String] -> :wat::core::nil
  (:wat::core::do
    (:wat::kernel::println (:wat::core::string::concat "== " path))
    (:wat::core::run!
      (:wat::core::fn [f <- :wat::WatAST] -> :wat::core::nil (:census::form f))
      (:wat::core::into []
        (:wat::core::ast->children
          (:wat::core::match (:wat::core::read-string (:wat::io::read-file path))
            ((:wat::core::ReadOutcome::Forms __forms) __forms)
            ((:wat::core::ReadOutcome::Malformed __cause)
              (:wat::kernel::assertion-failed! (:wat::core::Error/message __cause)
                :wat::core::None :wat::core::None))))))))

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::run!
    (:wat::core::fn [p <- :wat::core::String] -> :wat::core::nil (:census::file p))
    (:wat::core::match (:wat::kernel::readln )
      ((:wat::kernel::ReadlnOutcome::Datum __datum) __datum)
      (:wat::kernel::ReadlnOutcome::Eof
        (:wat::kernel::assertion-failed! "readln: end of input" :wat::core::None :wat::core::None))
      (:wat::kernel::ReadlnOutcome::Stopped
        (:wat::kernel::assertion-failed! "readln: stop requested" :wat::core::None :wat::core::None)))))
