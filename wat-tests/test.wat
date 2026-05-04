;; wat-tests/test.wat — self-tests for wat/test.wat.
;;
;; The test harness tests itself. Every assertion primitive gets both
;; a pass-case deftest (the assertion succeeds → deftest returns a
;; clean RunResult) and a fail-case deftest (run an inner program
;; that invokes the assertion with mismatched args, then inspect the
;; inner RunResult's Failure slot to verify the right diagnostic
;; surfaced).
;;
;; Inner programs use :wat::test::run-ast + :wat::test::program — no
;; escaped-string ceremony. The one test still using the string-entry
;; :wat::test::run is intentional: it verifies the STRING path works
;; for callers who build programs from strings at runtime (fuzzers,
;; dynamically-generated tests).


;; ─── assert-eq — pass cases ───────────────────────────────────────────

(:wat::test::deftest :wat-tests::std::test::test-assert-eq-on-i64
  ()
  (:wat::test::assert-eq 42 42))

(:wat::test::deftest :wat-tests::std::test::test-assert-eq-on-strings
  ()
  (:wat::test::assert-eq "hello" "hello"))

(:wat::test::deftest :wat-tests::std::test::test-assert-eq-on-bools
  ()
  (:wat::test::assert-eq true true))

(:wat::test::deftest :wat-tests::std::test::test-assert-eq-on-vec
  ()
  (:wat::core::let*
    (((a :wat::core::Vector<wat::core::String>) (:wat::core::Vector :wat::core::String "x" "y"))
     ((b :wat::core::Vector<wat::core::String>) (:wat::core::Vector :wat::core::String "x" "y")))
    (:wat::test::assert-eq a b)))

;; ─── assert-eq — fail case surfaces message ───────────────────────────

(:wat::test::deftest :wat-tests::std::test::test-assert-eq-fail-populates-message
  ()
  ;; rune:complectens(embedded-program) — outer let* has 2 bindings (r, fail); bulk is embedded-program AST literal (test fixture, not composition)
  (:wat::core::let*
    (((r :wat::kernel::RunResult)
      (:wat::test::run-ast
        (:wat::test::program
          (:wat::core::define
            (:user::main
              (stdin  :wat::io::IOReader)
              (stdout :wat::io::IOWriter)
              (stderr :wat::io::IOWriter)
              -> :wat::core::unit)
            (:wat::test::assert-eq 42 43)))
        (:wat::core::Vector :wat::core::String)))
     ((fail :wat::core::Option<wat::kernel::Failure>) (:wat::kernel::RunResult/failure r)))
    (:wat::core::match fail -> :wat::core::unit
      ((:wat::core::Some f) (:wat::test::assert-eq
                  (:wat::kernel::Failure/message f)
                  "assert-eq failed"))
      (:wat::core::None (:wat::kernel::assertion-failed!
               "expected Failure, got :None"
               :wat::core::None :wat::core::None)))))

;; ─── assert-contains — pass + fail ────────────────────────────────────

(:wat::test::deftest :wat-tests::std::test::test-assert-contains-hit
  ()
  (:wat::test::assert-contains "the quick brown fox" "quick"))

(:wat::test::deftest :wat-tests::std::test::test-assert-contains-fail-populates-actual
  ()
  ;; rune:complectens(embedded-program) — outer let* has 2 bindings (r, fail); bulk is embedded-program AST literal (test fixture, not composition)
  (:wat::core::let*
    (((r :wat::kernel::RunResult)
      (:wat::test::run-ast
        (:wat::test::program
          (:wat::core::define
            (:user::main
              (stdin  :wat::io::IOReader)
              (stdout :wat::io::IOWriter)
              (stderr :wat::io::IOWriter)
              -> :wat::core::unit)
            (:wat::test::assert-contains "hello" "xyz")))
        (:wat::core::Vector :wat::core::String)))
     ((fail :wat::core::Option<wat::kernel::Failure>) (:wat::kernel::RunResult/failure r)))
    (:wat::core::match fail -> :wat::core::unit
      ((:wat::core::Some f)
        (:wat::core::let*
          (((actual :wat::core::Option<wat::core::String>) (:wat::kernel::Failure/actual f))
           ((expected :wat::core::Option<wat::core::String>) (:wat::kernel::Failure/expected f))
           ((_ :wat::core::unit)
            (:wat::core::match actual -> :wat::core::unit
              ((:wat::core::Some a) (:wat::test::assert-eq a "hello"))
              (:wat::core::None (:wat::kernel::assertion-failed!
                       "actual slot empty" :wat::core::None :wat::core::None)))))
          (:wat::core::match expected -> :wat::core::unit
            ((:wat::core::Some e) (:wat::test::assert-eq e "xyz"))
            (:wat::core::None (:wat::kernel::assertion-failed!
                     "expected slot empty" :wat::core::None :wat::core::None)))))
      (:wat::core::None (:wat::kernel::assertion-failed!
               "expected Failure, got :None" :wat::core::None :wat::core::None)))))

;; ─── assert-coincident — pass + fail-renders-explanation ─────────────

(:wat::test::deftest :wat-tests::std::test::test-assert-coincident-pass
  ()
  (:wat::test::assert-coincident
    (:wat::holon::Atom "alice")
    (:wat::holon::Atom "alice")))

;; The fail-side test exercises arc 069's wiring: when the assertion
;; fails, the rendered CoincidentExplanation lands in the failure
;; payload's `actual` slot. We grep for each named field; their
;; presence is what matters, not exact numeric values (those depend
;; on the encoder's d at run time).
(:wat::test::deftest :wat-tests::std::test::test-assert-coincident-fail-renders-explanation
  ()
  ;; rune:complectens(embedded-program) — outer let* has 2 bindings (r, fail); bulk is embedded-program AST literal (test fixture, not composition)
  (:wat::core::let*
    (((r :wat::kernel::RunResult)
      (:wat::test::run-ast
        (:wat::test::program
          (:wat::core::define
            (:user::main
              (stdin  :wat::io::IOReader)
              (stdout :wat::io::IOWriter)
              (stderr :wat::io::IOWriter)
              -> :wat::core::unit)
            (:wat::test::assert-coincident
              (:wat::holon::Atom "alice")
              (:wat::holon::Atom "charlie"))))
        (:wat::core::Vector :wat::core::String)))
     ((fail :wat::core::Option<wat::kernel::Failure>) (:wat::kernel::RunResult/failure r)))
    (:wat::core::match fail -> :wat::core::unit
      ((:wat::core::Some f)
        (:wat::core::let*
          (((actual :wat::core::Option<wat::core::String>) (:wat::kernel::Failure/actual f)))
          (:wat::core::match actual -> :wat::core::unit
            ((:wat::core::Some a)
              (:wat::core::let*
                (((_ :wat::core::unit) (:wat::test::assert-contains a "cosine"))
                 ((_ :wat::core::unit) (:wat::test::assert-contains a "floor"))
                 ((_ :wat::core::unit) (:wat::test::assert-contains a "dim"))
                 ((_ :wat::core::unit) (:wat::test::assert-contains a "sigma"))
                 ((_ :wat::core::unit) (:wat::test::assert-contains
                            a "min-sigma-to-pass")))
                ()))
            (:wat::core::None (:wat::kernel::assertion-failed!
                     "actual slot empty — explanation should populate it"
                     :wat::core::None :wat::core::None)))))
      (:wat::core::None (:wat::kernel::assertion-failed!
               "expected Failure, got :None" :wat::core::None :wat::core::None)))))

;; ─── assert-stdout-is — pass case ─────────────────────────────────────

(:wat::test::deftest :wat-tests::std::test::test-assert-stdout-is-matches
  ()
  (:wat::core::let*
    (((inner :wat::kernel::RunResult)
      (:wat::test::run-ast
        (:wat::test::program
          (:wat::core::define
            (:user::main
              (stdin  :wat::io::IOReader)
              (stdout :wat::io::IOWriter)
              (stderr :wat::io::IOWriter)
              -> :wat::core::unit)
            (:wat::core::let*
              (((_ :wat::core::unit) (:wat::io::IOWriter/println stdout "alpha"))
               ((_ :wat::core::unit) (:wat::io::IOWriter/println stdout "beta")))
              ())))
        (:wat::core::Vector :wat::core::String)))
     ((expected :wat::core::Vector<wat::core::String>) (:wat::core::Vector :wat::core::String "alpha" "beta")))
    (:wat::test::assert-stdout-is inner expected)))

;; ─── assert-stderr-matches — pass + fail-reports-pattern ──────────────

(:wat::test::deftest :wat-tests::std::test::test-assert-stderr-matches-pass
  ()
  (:wat::core::let*
    (((inner :wat::kernel::RunResult)
      (:wat::test::run-ast
        (:wat::test::program
          (:wat::core::define
            (:user::main
              (stdin  :wat::io::IOReader)
              (stdout :wat::io::IOWriter)
              (stderr :wat::io::IOWriter)
              -> :wat::core::unit)
            (:wat::io::IOWriter/println stderr "error: code 42")))
        (:wat::core::Vector :wat::core::String))))
    (:wat::test::assert-stderr-matches inner "code [0-9]+")))

(:wat::test::deftest :wat-tests::std::test::test-assert-stderr-matches-fail-reports-pattern
  ()
  ;; rune:complectens(embedded-program) — outer let* has 2 bindings (r, fail); bulk is a TWO-level nested embedded-program AST literal (fixture)
  ;; Two-level nested sandbox: outer program runs inner program that runs silent program. The middle layer calls assert-stderr-matches
  ;; against the silent program's empty stderr; that assertion fires; the middle program's RunResult.failure is populated with
  ;; expected = "my-pattern". The outer inspects the middle's failure.
  (:wat::core::let*
    (((r :wat::kernel::RunResult)
      (:wat::test::run-ast
        (:wat::test::program
          (:wat::core::define
            (:user::main
              (stdin  :wat::io::IOReader)
              (stdout :wat::io::IOWriter)
              (stderr :wat::io::IOWriter)
              -> :wat::core::unit)
            (:wat::core::let*
              (((silent :wat::kernel::RunResult)
                (:wat::test::run-ast
                  (:wat::test::program
                    (:wat::core::define
                      (:user::main
                        (stdin  :wat::io::IOReader)
                        (stdout :wat::io::IOWriter)
                        (stderr :wat::io::IOWriter)
                        -> :wat::core::unit)
                      ()))
                  (:wat::core::Vector :wat::core::String))))
              (:wat::test::assert-stderr-matches silent "my-pattern"))))
        (:wat::core::Vector :wat::core::String)))
     ((fail :wat::core::Option<wat::kernel::Failure>) (:wat::kernel::RunResult/failure r)))
    (:wat::core::match fail -> :wat::core::unit
      ((:wat::core::Some f)
        (:wat::core::let*
          (((expected :wat::core::Option<wat::core::String>) (:wat::kernel::Failure/expected f)))
          (:wat::core::match expected -> :wat::core::unit
            ((:wat::core::Some e) (:wat::test::assert-eq e "my-pattern"))
            (:wat::core::None (:wat::kernel::assertion-failed!
                     "expected slot empty" :wat::core::None :wat::core::None)))))
      (:wat::core::None (:wat::kernel::assertion-failed!
               "expected Failure, got :None" :wat::core::None :wat::core::None)))))

;; ─── :wat::test::run wrapper (string-entry path, kept for coverage) ───
;;
;; Programs built at runtime from strings — fuzzers, generated tests,
;; etc. — still use the string-entry run. This test verifies that
;; path continues to work alongside the AST-entry path used above.

(:wat::test::deftest :wat-tests::std::test::test-run-string-entry-path
  ()
  (:wat::core::let*
    (((r :wat::kernel::RunResult)
      (:wat::test::run
        "(:wat::config::set-capacity-mode! :error)
         (:wat::core::define (:user::main
                              (stdin  :wat::io::IOReader)
                              (stdout :wat::io::IOWriter)
                              (stderr :wat::io::IOWriter)
                              -> :wat::core::unit)
           (:wat::io::IOWriter/println stdout \"from-string\"))"
        (:wat::core::Vector :wat::core::String)))
     ((expected :wat::core::Vector<wat::core::String>) (:wat::core::Vector :wat::core::String "from-string")))
    (:wat::test::assert-stdout-is r expected)))

;; ─── :wat::test::run-ast — AST-entry path via :wat::test::program ────

(:wat::test::deftest :wat-tests::std::test::test-run-ast-via-program
  ()
  (:wat::core::let*
    (((r :wat::kernel::RunResult)
      (:wat::test::run-ast
        (:wat::test::program
          (:wat::core::define
            (:user::main
              (stdin  :wat::io::IOReader)
              (stdout :wat::io::IOWriter)
              (stderr :wat::io::IOWriter)
              -> :wat::core::unit)
            (:wat::io::IOWriter/println stdout "from-ast")))
        (:wat::core::Vector :wat::core::String)))
     ((expected :wat::core::Vector<wat::core::String>) (:wat::core::Vector :wat::core::String "from-ast")))
    (:wat::test::assert-stdout-is r expected)))

;; deftest's self-test is redundant here — every other passing deftest
;; in this file IS proof that deftest registered a callable zero-arg
;; :wat::kernel::RunResult-returning function, because `wat test`
;; discovered them by exactly that signature and invoked them
;; (signature-only discovery; the legacy `test-` last-segment filter
;; was dropped 2026-04-25). If deftest were broken, this whole file
;; would fail at discovery / startup, not one test.

;; ─── :wat::test::make-deftest — arc 029 slice 2 ──────────────────────
;;
;; Configured-deftest factory. The preamble registers an ambient
;; name; subsequent callsites are just name + body. Proves the
;; macro-generating-macro path end-to-end: outer make-deftest
;; expands to a defmacro registration, the generated defmacro
;; expands to a deftest call, the deftest expands to the full
;; run-sandboxed-ast scaffolding, and the test runs.

(:wat::test::make-deftest :wat-tests::std::test::cfg-deftest ())

(:wat-tests::std::test::cfg-deftest
  :wat-tests::std::test::test-make-deftest-runs
  (:wat::test::assert-eq (:wat::core::i64::+,2 2 2) 4))

(:wat-tests::std::test::cfg-deftest
  :wat-tests::std::test::test-make-deftest-second-test
  (:wat::test::assert-eq 10 (:wat::core::i64::*,2 5 2)))

;; ─── :wat::core::macroexpand / macroexpand-1 — arc 030 ────────────────
;;
;; The standard Lisp macro-debugging tool. Quote a form, hand it to
;; macroexpand(-1), inspect the returned AST. Lets users see what a
;; macro call produces without evaluating it.

(:wat::test::deftest :wat-tests::std::test::test-macroexpand-1-non-macro
  ()
  ;; A plain expression (no macro head) expands to itself. Verify by
  ;; evaluating the expanded AST and checking it produces Ok.
  (:wat::core::match
    (:wat::eval-ast!
      (:wat::core::macroexpand-1
        (:wat::core::quote (:wat::core::i64::+,2 2 2))))
    -> :wat::core::unit
    ((:wat::core::Ok _) (:wat::test::assert-eq true true))
    ((:wat::core::Err _) (:wat::test::assert-eq true false))))

(:wat::test::deftest :wat-tests::std::test::test-macroexpand-fixpoint-evaluates
  ()
  ;; macroexpand returns a :wat::WatAST; hand it to eval-ast!
  ;; to prove the expansion is evaluable.
  (:wat::core::match
    (:wat::eval-ast!
      (:wat::core::macroexpand
        (:wat::core::quote (:wat::core::i64::*,2 3 4))))
    -> :wat::core::unit
    ((:wat::core::Ok _) (:wat::test::assert-eq true true))
    ((:wat::core::Err _) (:wat::test::assert-eq true false))))
