;; :wat::test::* — the wat-native test harness (arc 007 slice 3).
;;
;; Pure wat over three primitives:
;; - :wat::kernel::run-sandboxed        (arc 007 slice 2b)
;; - :wat::kernel::run-sandboxed-hermetic (arc 007 slice 2c)
;; - :wat::kernel::assertion-failed!    (this slice)
;; Plus the string/regex basics from :wat::core::string::* and
;; :wat::core::regex::*.
;;
;; Usage shape:
;;
;;   (:wat::core::define (:user::main
;;                        (stdin  :wat::io::IOReader)
;;                        (stdout :wat::io::IOWriter)
;;                        (stderr :wat::io::IOWriter)
;;                        -> :())
;;     (:wat::core::let*
;;       (((r :wat::kernel::RunResult)
;;         (:wat::test::run "(:user::main ...)" (:wat::core::vec :String))))
;;       (:wat::test::assert-stdout-is r
;;         (:wat::core::conj (:wat::core::vec :String) "expected-line"))))
;;
;; An assertion that fails panics internally; the outer run-sandboxed
;; catches the panic and surfaces the failure in its own RunResult.
;; Nested testing: a test file runs sandboxed to TEST a sandboxed
;; program.

;; ─── assert-eq<T> ─────────────────────────────────────────────────────
;;
;; Structural equality via :wat::core::=. Actual/expected stringification
;; is future work — requires generic show<T>. Until then, a failed
;; assert-eq reports the message only; the failure shape still carries
;; the actual/expected SLOTS (as :None) so callers that match on Failure
;; see a stable shape.
(:wat::core::define
  (:wat::test::assert-eq<T>
    (actual :T)
    (expected :T)
    -> :())
  (:wat::core::if (:wat::core::= actual expected) -> :()
    ()
    (:wat::kernel::assertion-failed!
      "assert-eq failed"
      :None
      :None)))

;; ─── assert-contains ──────────────────────────────────────────────────
;;
;; String substring check. Unlike assert-eq, both sides are :String so
;; we can populate actual/expected with the real values — the failure
;; in a RunResult shows the user which haystack/needle fired.
(:wat::core::define
  (:wat::test::assert-contains
    (haystack :String)
    (needle :String)
    -> :())
  (:wat::core::if (:wat::core::string::contains? haystack needle) -> :()
    ()
    (:wat::kernel::assertion-failed!
      "assert-contains failed"
      (Some haystack)
      (Some needle))))

;; ─── assert-stdout-is ─────────────────────────────────────────────────
;;
;; Compare a RunResult's stdout to an expected Vec<String>. Equality via
;; :wat::core::=, which is defined over T — for Vec<String> it compares
;; elementwise. Joins both sides with "\n" into the Failure payload so
;; the user sees the diff in a RunResult.
(:wat::core::define
  (:wat::test::assert-stdout-is
    (result :wat::kernel::RunResult)
    (expected :Vec<String>)
    -> :())
  (:wat::core::let*
    (((actual :Vec<String>) (:wat::kernel::RunResult/stdout result)))
    (:wat::core::if (:wat::core::= actual expected) -> :()
      ()
      (:wat::kernel::assertion-failed!
        "assert-stdout-is failed"
        (Some (:wat::core::string::join "\n" actual))
        (Some (:wat::core::string::join "\n" expected))))))

;; ─── assert-stderr-matches ────────────────────────────────────────────
;;
;; Regex match (unanchored) against each line of a RunResult's stderr.
;; Any line matching passes. Uses foldl over Vec<String> to OR the
;; matches — a straightforward "any" without a new primitive.
(:wat::core::define
  (:wat::test::any-line-matches
    (pattern :String)
    (lines :Vec<String>)
    -> :bool)
  (:wat::core::foldl lines false
    (:wat::core::lambda ((acc :bool) (line :String) -> :bool)
      (:wat::core::or acc (:wat::core::regex::matches? pattern line)))))

(:wat::core::define
  (:wat::test::assert-stderr-matches
    (result :wat::kernel::RunResult)
    (pattern :String)
    -> :())
  (:wat::core::let*
    (((stderr-lines :Vec<String>) (:wat::kernel::RunResult/stderr result)))
    (:wat::core::if (:wat::test::any-line-matches pattern stderr-lines) -> :()
      ()
      (:wat::kernel::assertion-failed!
        "assert-stderr-matches failed — no stderr line matched pattern"
        (Some (:wat::core::string::join "\n" stderr-lines))
        (Some pattern)))))

;; ─── run / run-in-scope ───────────────────────────────────────────────
;;
;; Thin ergonomic wrappers over :wat::kernel::run-sandboxed. `run` is
;; the common case — no filesystem access at all (InMemoryLoader).
;; `run-in-scope` sets up ScopedLoader when the test uses load! with
;; fixture files.
(:wat::core::define
  (:wat::test::run
    (src :String)
    (stdin :Vec<String>)
    -> :wat::kernel::RunResult)
  (:wat::kernel::run-sandboxed src stdin :None))

(:wat::core::define
  (:wat::test::run-in-scope
    (src :String)
    (stdin :Vec<String>)
    (scope :String)
    -> :wat::kernel::RunResult)
  (:wat::kernel::run-sandboxed src stdin (Some scope)))

;; ─── deftest — Clojure-style ergonomic shell (arc 007 slice 3b) ───────
;;
;; Registers a named zero-arg test function that returns RunResult.
;; The body runs inside a fresh sandboxed world with the caller's dims
;; + capacity-mode committed. When slice 4's test discoverer lands, it
;; iterates registered functions and invokes each.
;;
;; Shape:
;;
;;   (:wat::test::deftest :my::test::two-plus-two 1024 :error
;;     (:wat::test::assert-eq (:wat::core::i64::+ 2 2) 4))
;;
;; Expansion:
;;
;;   (:wat::core::define (:my::test::two-plus-two -> :wat::kernel::RunResult)
;;     (:wat::kernel::run-sandboxed-ast
;;       (:wat::core::vec :wat::WatAST
;;         (:wat::core::quote (:wat::config::set-dims! 1024))
;;         (:wat::core::quote (:wat::config::set-capacity-mode! :error))
;;         (:wat::core::quote (:wat::core::define (:user::main
;;                                                 (stdin  :wat::io::IOReader)
;;                                                 (stdout :wat::io::IOWriter)
;;                                                 (stderr :wat::io::IOWriter)
;;                                                 -> :())
;;                              <body>)))
;;       (:wat::core::vec :String)
;;       :None))
(:wat::core::defmacro
  (:wat::test::deftest
    (name :AST<()>)
    (dims :AST<i64>)
    (mode :AST<wat::core::keyword>)
    (body :AST<()>)
    -> :AST<()>)
  `(:wat::core::define (,name -> :wat::kernel::RunResult)
     (:wat::kernel::run-sandboxed-ast
       (:wat::core::vec :wat::WatAST
         (:wat::core::quote (:wat::config::set-dims! ,dims))
         (:wat::core::quote (:wat::config::set-capacity-mode! ,mode))
         (:wat::core::quote
           (:wat::core::define
             (:user::main
               (stdin  :wat::io::IOReader)
               (stdout :wat::io::IOWriter)
               (stderr :wat::io::IOWriter)
               -> :())
             ,body)))
       (:wat::core::vec :String)
       :None)))
