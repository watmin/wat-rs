;; wat-tests/std/test.wat — self-tests for wat/std/test.wat.
;;
;; The test harness tests itself. Every assertion primitive gets both
;; a pass-case deftest (the assertion succeeds → deftest returns a
;; clean RunResult) and a fail-case deftest (run an inner program via
;; :wat::test::run that invokes the assertion with mismatched args,
;; then inspect the inner RunResult's Failure slot to verify the
;; right diagnostic surfaced).
;;
;; deftest itself is proven by a fixture deftest the other tests call:
;; if deftest registers a callable zero-arg :wat::kernel::RunResult
;; function, calling :example-fixture returns a valid RunResult.

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
      (:wat::test::run
        "(:wat::config::set-dims! 1024)
         (:wat::config::set-capacity-mode! :error)
         (:wat::core::define (:user::main
                              (stdin  :wat::io::IOReader)
                              (stdout :wat::io::IOWriter)
                              (stderr :wat::io::IOWriter)
                              -> :())
           (:wat::test::assert-eq 42 43))"
        (:wat::core::vec :String)))
     ((fail :Option<wat::kernel::Failure>)
      (:wat::kernel::RunResult/failure r)))
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
      (:wat::test::run
        "(:wat::config::set-dims! 1024)
         (:wat::config::set-capacity-mode! :error)
         (:wat::core::define (:user::main
                              (stdin  :wat::io::IOReader)
                              (stdout :wat::io::IOWriter)
                              (stderr :wat::io::IOWriter)
                              -> :())
           (:wat::test::assert-contains \"hello\" \"xyz\"))"
        (:wat::core::vec :String)))
     ((fail :Option<wat::kernel::Failure>)
      (:wat::kernel::RunResult/failure r)))
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
      (:wat::test::run
        "(:wat::config::set-dims! 1024)
         (:wat::config::set-capacity-mode! :error)
         (:wat::core::define (:user::main
                              (stdin  :wat::io::IOReader)
                              (stdout :wat::io::IOWriter)
                              (stderr :wat::io::IOWriter)
                              -> :())
           (:wat::core::let*
             (((_ :()) (:wat::io::IOWriter/println stdout \"alpha\"))
              ((_ :()) (:wat::io::IOWriter/println stdout \"beta\")))
             ()))"
        (:wat::core::vec :String)))
     ((expected :Vec<String>) (:wat::core::vec :String "alpha" "beta")))
    (:wat::test::assert-stdout-is inner expected)))

;; ─── assert-stderr-matches — pass + fail-reports-pattern ──────────────

(:wat::test::deftest :wat-tests::std::test::test-assert-stderr-matches-pass 1024 :error
  (:wat::core::let*
    (((inner :wat::kernel::RunResult)
      (:wat::test::run
        "(:wat::config::set-dims! 1024)
         (:wat::config::set-capacity-mode! :error)
         (:wat::core::define (:user::main
                              (stdin  :wat::io::IOReader)
                              (stdout :wat::io::IOWriter)
                              (stderr :wat::io::IOWriter)
                              -> :())
           (:wat::io::IOWriter/println stderr \"error: code 42\"))"
        (:wat::core::vec :String))))
    (:wat::test::assert-stderr-matches inner "code [0-9]+")))

(:wat::test::deftest :wat-tests::std::test::test-assert-stderr-matches-fail-reports-pattern 1024 :error
  ;; Inner writes nothing to stderr; assert-stderr-matches fails; outer
  ;; inspects Failure.expected (should be the pattern passed in).
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
           (:wat::core::let*
             (((silent :wat::kernel::RunResult)
               (:wat::test::run
                 \"(:wat::config::set-dims! 1024)
                  (:wat::config::set-capacity-mode! :error)
                  (:wat::core::define (:user::main
                                       (stdin  :wat::io::IOReader)
                                       (stdout :wat::io::IOWriter)
                                       (stderr :wat::io::IOWriter)
                                       -> :())
                    ())\"
                 (:wat::core::vec :String))))
             (:wat::test::assert-stderr-matches silent \"my-pattern\")))"
        (:wat::core::vec :String)))
     ((fail :Option<wat::kernel::Failure>)
      (:wat::kernel::RunResult/failure r)))
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

;; ─── :wat::test::run wrapper ──────────────────────────────────────────

(:wat::test::deftest :wat-tests::std::test::test-run-wraps-sandbox 1024 :error
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
           (:wat::io::IOWriter/println stdout \"captured\"))"
        (:wat::core::vec :String)))
     ((expected :Vec<String>) (:wat::core::vec :String "captured")))
    (:wat::test::assert-stdout-is r expected)))

;; deftest's self-test is redundant here — every other passing deftest
;; in this file IS proof that deftest registered a callable zero-arg
;; :wat::kernel::RunResult-returning function, because `wat test`
;; discovered them by exactly that signature + name convention and
;; invoked them. If deftest were broken, this whole file would fail
;; at discovery / startup, not one test.
;;
;; (Previously attempted: a test-deftest-registers-callable that
;; called a sibling passing-fixture deftest and inspected its result.
;; Impossible — each deftest body runs in its own fresh sandbox via
;; run-sandboxed-ast, so cross-deftest references within the same
;; file don't resolve. The deftest IS the sandbox boundary.)
