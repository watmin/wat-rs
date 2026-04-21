;; wat-tests/std/test.wat — self-tests for wat/std/test.wat.
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

(:wat::config::set-dims! 1024)
(:wat::config::set-capacity-mode! :error)

;; ─── assert-eq — pass cases ───────────────────────────────────────────

(:wat::test::deftest :wat-tests::std::test::test-assert-eq-on-i64 1024 :error
  (:wat::test::assert-eq 42 42))

(:wat::test::deftest :wat-tests::std::test::test-assert-eq-on-strings 1024 :error
  (:wat::test::assert-eq "hello" "hello"))

(:wat::test::deftest :wat-tests::std::test::test-assert-eq-on-bools 1024 :error
  (:wat::test::assert-eq true true))

(:wat::test::deftest :wat-tests::std::test::test-assert-eq-on-vec 1024 :error
  (:wat::core::let*
    (((a :Vec<String>) (:wat::core::vec :String "x" "y"))
     ((b :Vec<String>) (:wat::core::vec :String "x" "y")))
    (:wat::test::assert-eq a b)))

;; ─── assert-eq — fail case surfaces message ───────────────────────────

(:wat::test::deftest :wat-tests::std::test::test-assert-eq-fail-populates-message 1024 :error
  (:wat::core::let*
    (((r :wat::kernel::RunResult)
      (:wat::test::run-ast
        (:wat::test::program
          (:wat::config::set-dims! 1024)
          (:wat::config::set-capacity-mode! :error)
          (:wat::core::define
            (:user::main
              (stdin  :wat::io::IOReader)
              (stdout :wat::io::IOWriter)
              (stderr :wat::io::IOWriter)
              -> :())
            (:wat::test::assert-eq 42 43)))
        (:wat::core::vec :String)))
     ((fail :Option<wat::kernel::Failure>) (:wat::kernel::RunResult/failure r)))
    (:wat::core::match fail -> :()
      ((Some f) (:wat::test::assert-eq
                  (:wat::kernel::Failure/message f)
                  "assert-eq failed"))
      (:None (:wat::kernel::assertion-failed!
               "expected Failure, got :None"
               :None :None)))))

;; ─── assert-contains — pass + fail ────────────────────────────────────

(:wat::test::deftest :wat-tests::std::test::test-assert-contains-hit 1024 :error
  (:wat::test::assert-contains "the quick brown fox" "quick"))

(:wat::test::deftest :wat-tests::std::test::test-assert-contains-fail-populates-actual 1024 :error
  (:wat::core::let*
    (((r :wat::kernel::RunResult)
      (:wat::test::run-ast
        (:wat::test::program
          (:wat::config::set-dims! 1024)
          (:wat::config::set-capacity-mode! :error)
          (:wat::core::define
            (:user::main
              (stdin  :wat::io::IOReader)
              (stdout :wat::io::IOWriter)
              (stderr :wat::io::IOWriter)
              -> :())
            (:wat::test::assert-contains "hello" "xyz")))
        (:wat::core::vec :String)))
     ((fail :Option<wat::kernel::Failure>) (:wat::kernel::RunResult/failure r)))
    (:wat::core::match fail -> :()
      ((Some f)
        (:wat::core::let*
          (((actual :Option<String>) (:wat::kernel::Failure/actual f))
           ((expected :Option<String>) (:wat::kernel::Failure/expected f))
           ((_ :())
            (:wat::core::match actual -> :()
              ((Some a) (:wat::test::assert-eq a "hello"))
              (:None (:wat::kernel::assertion-failed!
                       "actual slot empty" :None :None)))))
          (:wat::core::match expected -> :()
            ((Some e) (:wat::test::assert-eq e "xyz"))
            (:None (:wat::kernel::assertion-failed!
                     "expected slot empty" :None :None)))))
      (:None (:wat::kernel::assertion-failed!
               "expected Failure, got :None" :None :None)))))

;; ─── assert-stdout-is — pass case ─────────────────────────────────────

(:wat::test::deftest :wat-tests::std::test::test-assert-stdout-is-matches 1024 :error
  (:wat::core::let*
    (((inner :wat::kernel::RunResult)
      (:wat::test::run-ast
        (:wat::test::program
          (:wat::config::set-dims! 1024)
          (:wat::config::set-capacity-mode! :error)
          (:wat::core::define
            (:user::main
              (stdin  :wat::io::IOReader)
              (stdout :wat::io::IOWriter)
              (stderr :wat::io::IOWriter)
              -> :())
            (:wat::core::let*
              (((_ :()) (:wat::io::IOWriter/println stdout "alpha"))
               ((_ :()) (:wat::io::IOWriter/println stdout "beta")))
              ())))
        (:wat::core::vec :String)))
     ((expected :Vec<String>) (:wat::core::vec :String "alpha" "beta")))
    (:wat::test::assert-stdout-is inner expected)))

;; ─── assert-stderr-matches — pass + fail-reports-pattern ──────────────

(:wat::test::deftest :wat-tests::std::test::test-assert-stderr-matches-pass 1024 :error
  (:wat::core::let*
    (((inner :wat::kernel::RunResult)
      (:wat::test::run-ast
        (:wat::test::program
          (:wat::config::set-dims! 1024)
          (:wat::config::set-capacity-mode! :error)
          (:wat::core::define
            (:user::main
              (stdin  :wat::io::IOReader)
              (stdout :wat::io::IOWriter)
              (stderr :wat::io::IOWriter)
              -> :())
            (:wat::io::IOWriter/println stderr "error: code 42")))
        (:wat::core::vec :String))))
    (:wat::test::assert-stderr-matches inner "code [0-9]+")))

(:wat::test::deftest :wat-tests::std::test::test-assert-stderr-matches-fail-reports-pattern 1024 :error
  ;; Two-level nested sandbox: outer program runs inner program that
  ;; runs silent program. The middle layer calls assert-stderr-matches
  ;; against the silent program's empty stderr; that assertion fires;
  ;; the middle program's RunResult.failure is populated with
  ;; expected = "my-pattern". The outer inspects the middle's failure.
  (:wat::core::let*
    (((r :wat::kernel::RunResult)
      (:wat::test::run-ast
        (:wat::test::program
          (:wat::config::set-dims! 1024)
          (:wat::config::set-capacity-mode! :error)
          (:wat::core::define
            (:user::main
              (stdin  :wat::io::IOReader)
              (stdout :wat::io::IOWriter)
              (stderr :wat::io::IOWriter)
              -> :())
            (:wat::core::let*
              (((silent :wat::kernel::RunResult)
                (:wat::test::run-ast
                  (:wat::test::program
                    (:wat::config::set-dims! 1024)
                    (:wat::config::set-capacity-mode! :error)
                    (:wat::core::define
                      (:user::main
                        (stdin  :wat::io::IOReader)
                        (stdout :wat::io::IOWriter)
                        (stderr :wat::io::IOWriter)
                        -> :())
                      ()))
                  (:wat::core::vec :String))))
              (:wat::test::assert-stderr-matches silent "my-pattern"))))
        (:wat::core::vec :String)))
     ((fail :Option<wat::kernel::Failure>) (:wat::kernel::RunResult/failure r)))
    (:wat::core::match fail -> :()
      ((Some f)
        (:wat::core::let*
          (((expected :Option<String>) (:wat::kernel::Failure/expected f)))
          (:wat::core::match expected -> :()
            ((Some e) (:wat::test::assert-eq e "my-pattern"))
            (:None (:wat::kernel::assertion-failed!
                     "expected slot empty" :None :None)))))
      (:None (:wat::kernel::assertion-failed!
               "expected Failure, got :None" :None :None)))))

;; ─── :wat::test::run wrapper (string-entry path, kept for coverage) ───
;;
;; Programs built at runtime from strings — fuzzers, generated tests,
;; etc. — still use the string-entry run. This test verifies that
;; path continues to work alongside the AST-entry path used above.

(:wat::test::deftest :wat-tests::std::test::test-run-string-entry-path 1024 :error
  (:wat::core::let*
    (((r :wat::kernel::RunResult)
      (:wat::test::run
        "(:wat::config::set-dims! 1024)
         (:wat::config::set-capacity-mode! :error)
         (:wat::core::define (:user::main
                              (stdin  :wat::io::IOReader)
                              (stdout :wat::io::IOWriter)
                              (stderr :wat::io::IOWriter)
                              -> :())
           (:wat::io::IOWriter/println stdout \"from-string\"))"
        (:wat::core::vec :String)))
     ((expected :Vec<String>) (:wat::core::vec :String "from-string")))
    (:wat::test::assert-stdout-is r expected)))

;; ─── :wat::test::run-ast — AST-entry path via :wat::test::program ────

(:wat::test::deftest :wat-tests::std::test::test-run-ast-via-program 1024 :error
  (:wat::core::let*
    (((r :wat::kernel::RunResult)
      (:wat::test::run-ast
        (:wat::test::program
          (:wat::config::set-dims! 1024)
          (:wat::config::set-capacity-mode! :error)
          (:wat::core::define
            (:user::main
              (stdin  :wat::io::IOReader)
              (stdout :wat::io::IOWriter)
              (stderr :wat::io::IOWriter)
              -> :())
            (:wat::io::IOWriter/println stdout "from-ast")))
        (:wat::core::vec :String)))
     ((expected :Vec<String>) (:wat::core::vec :String "from-ast")))
    (:wat::test::assert-stdout-is r expected)))

;; deftest's self-test is redundant here — every other passing deftest
;; in this file IS proof that deftest registered a callable zero-arg
;; :wat::kernel::RunResult-returning function, because `wat test`
;; discovered them by exactly that signature + name convention and
;; invoked them. If deftest were broken, this whole file would fail
;; at discovery / startup, not one test.
