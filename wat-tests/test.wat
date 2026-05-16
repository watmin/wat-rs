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
  (:wat::core::let
    [a (:wat::core::Vector :wat::core::String "x" "y")
     b (:wat::core::Vector :wat::core::String "x" "y")]
    (:wat::test::assert-eq a b)))

;; ─── assert-eq — fail case surfaces message ───────────────────────────

(:wat::test::deftest :wat-tests::std::test::test-assert-eq-fail-populates-message
  ()
  ;; rune:complectens(embedded-program) — outer let has 2 bindings (r, fail); bulk is embedded-program AST literal (test fixture, not composition)
  (:wat::core::let
    [r
      (:wat::test::run-thread
        (:wat::test::assert-eq 42 43))
     fail (:wat::kernel::RunResult/failure r)]
    (:wat::core::match fail -> :wat::core::nil
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
  ;; rune:complectens(embedded-program) — outer let has 2 bindings (r, fail); bulk is embedded-program AST literal (test fixture, not composition)
  (:wat::core::let
    [r
      (:wat::test::run-thread
        (:wat::test::assert-contains "hello" "xyz"))
     fail (:wat::kernel::RunResult/failure r)]
    (:wat::core::match fail -> :wat::core::nil
      ((:wat::core::Some f)
        (:wat::core::let
          [actual (:wat::kernel::Failure/actual f)
           expected (:wat::kernel::Failure/expected f)
           _
            (:wat::core::match actual -> :wat::core::nil
              ((:wat::core::Some a) (:wat::test::assert-eq a "hello"))
              (:wat::core::None (:wat::kernel::assertion-failed!
                       "actual slot empty" :wat::core::None :wat::core::None)))]
          (:wat::core::match expected -> :wat::core::nil
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
  ;; rune:complectens(embedded-program) — outer let has 2 bindings (r, fail); bulk is embedded-program AST literal (test fixture, not composition)
  (:wat::core::let
    [r
      (:wat::test::run-thread
        (:wat::test::assert-coincident
          (:wat::holon::Atom "alice")
          (:wat::holon::Atom "charlie")))
     fail (:wat::kernel::RunResult/failure r)]
    (:wat::core::match fail -> :wat::core::nil
      ((:wat::core::Some f)
        (:wat::core::let
          [actual (:wat::kernel::Failure/actual f)]
          (:wat::core::match actual -> :wat::core::nil
            ((:wat::core::Some a)
              (:wat::core::do
                (:wat::test::assert-contains a "cosine")
                (:wat::test::assert-contains a "floor")
                (:wat::test::assert-contains a "dim")
                (:wat::test::assert-contains a "sigma")
                (:wat::test::assert-contains
                            a "min-sigma-to-pass")
                ()))
            (:wat::core::None (:wat::kernel::assertion-failed!
                     "actual slot empty — explanation should populate it"
                     :wat::core::None :wat::core::None)))))
      (:wat::core::None (:wat::kernel::assertion-failed!
               "expected Failure, got :None" :wat::core::None :wat::core::None)))))

;; ─── assert-stdout-is — pass case ─────────────────────────────────────

(:wat::test::deftest-hermetic :wat-tests::std::test::test-assert-stdout-is-matches
  ()
  (:wat::core::let
    [inner
      (:wat::test::run-hermetic
        (:wat::core::do
          (:wat::kernel::println "alpha")
          (:wat::kernel::println "beta")
          ()))
     expected (:wat::core::Vector :wat::core::String "\"alpha\"" "\"beta\"")]
    (:wat::test::assert-stdout-is inner expected)))

;; ─── assert-stderr-matches — pass + fail-reports-pattern ──────────────

(:wat::test::deftest-hermetic :wat-tests::std::test::test-assert-stderr-matches-pass
  ()
  (:wat::core::let
    [inner
      (:wat::test::run-hermetic
        (:wat::kernel::eprintln "error: code 42"))]
    (:wat::test::assert-stderr-matches inner "code [0-9]+")))

(:wat::test::deftest-hermetic :wat-tests::std::test::test-assert-stderr-matches-fail-reports-pattern
  ()
  ;; Verifies assert-stderr-matches's failure-reporting shape on REAL non-matching stderr.
  ;; Inner produces actual stderr content that doesn't match the pattern; the matcher
  ;; loop runs against that content and fires; the failure carries `expected = "my-pattern"`
  ;; and `actual = (Vec ... captured stderr lines ...)`.
  ;;
  ;; Architectural change (arc 170 slice 4a-γ-decorate): inner spawn was previously
  ;; :wat::test::run-thread with empty body — the test passed via empty-input edge case
  ;; without exercising the pattern-matching machinery. The rearchitecture uses
  ;; :wat::test::run-hermetic with a non-matching stderr line so the matcher loop
  ;; actually runs.
  (:wat::core::let
    [r
      (:wat::test::run-hermetic
        (:wat::core::let
          [silent
            (:wat::test::run-hermetic
              (:wat::kernel::eprintln "different content"))]
          (:wat::test::assert-stderr-matches silent "my-pattern")))
     fail (:wat::kernel::RunResult/failure r)]
    (:wat::core::match fail -> :wat::core::nil
      ((:wat::core::Some f)
        (:wat::core::let
          [expected (:wat::kernel::Failure/expected f)]
          (:wat::core::match expected -> :wat::core::nil
            ((:wat::core::Some e) (:wat::test::assert-eq e "my-pattern"))
            (:wat::core::None (:wat::kernel::assertion-failed!
                     "expected slot empty" :wat::core::None :wat::core::None)))))
      (:wat::core::None (:wat::kernel::assertion-failed!
               "expected Failure, got :None" :wat::core::None :wat::core::None)))))

;; ─── :wat::test::run-thread — legacy string-entry path migrated ──────
;;
;; Arc 170 slice 4a-β: the legacy :wat::test::run took a runtime
;; source string; the modern surface takes a body AST directly via
;; :wat::test::run-thread. Multi-form sources wrap in
;; (:wat::core::do ...). The dynamic-source-string path is no longer
;; exercised here — the modern surface is body-AST only.

;; Duplicate of :wat-tests::std::test::test-assert-stdout-is-matches at line 132 —
;; same hermetic-print-and-capture pattern with different fixture string. Preserved
;; per accumulate-tests-defer-cleanup policy (test cleanup is post-109; coverage
;; tooling needed to verify safe deletion). Original test purpose
;; ("test the legacy STRING-entry path") retired during arc 170 slice 4a-β
;; when the legacy :wat::test::run path was swept to canonical macros.
(:wat::test::deftest-hermetic :wat-tests::std::test::test-run-string-entry-path
  ()
  ;; Arc 170 slice 4a-β: this test originally exercised the legacy
  ;; :wat::test::run STRING-parsing path; the inner source carried a
  ;; (:wat::config::set-capacity-mode! :error) form that the legacy
  ;; substrate config-collected. The modern body-AST shape has no
  ;; analogue — config-setters are file-level, not body-runtime forms.
  ;; The test now verifies the simpler post-migration shape: hermetic
  ;; child prints, parent captures stdout. The original "STRING-path
  ;; tested" intent retires with the legacy :wat::test::run define.
  (:wat::core::let
    [r
      (:wat::test::run-hermetic
        (:wat::kernel::println "from-string"))
     expected (:wat::core::Vector :wat::core::String "\"from-string\"")]
    (:wat::test::assert-stdout-is r expected)))

;; ─── :wat::test::run-ast — AST-entry path via :wat::test::program ────

;; Duplicate of :wat-tests::std::test::test-assert-stdout-is-matches at line 132 —
;; same hermetic-print-and-capture pattern with different fixture string. Preserved
;; per accumulate-tests-defer-cleanup policy (test cleanup is post-109; coverage
;; tooling needed to verify safe deletion). Original test purpose
;; ("test the legacy AST-via-program path") retired during arc 170 slice 4a-β
;; when the legacy :wat::test::run-ast path was swept to canonical macros.
(:wat::test::deftest-hermetic :wat-tests::std::test::test-run-ast-via-program
  ()
  (:wat::core::let
    [r
      (:wat::test::run-hermetic
        (:wat::kernel::println "from-ast"))
     expected (:wat::core::Vector :wat::core::String "\"from-ast\"")]
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
  (:wat::test::assert-eq (:wat::core::i64::+'2 2 2) 4))

(:wat-tests::std::test::cfg-deftest
  :wat-tests::std::test::test-make-deftest-second-test
  (:wat::test::assert-eq 10 (:wat::core::i64::*'2 5 2)))

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
        (:wat::core::quote (:wat::core::i64::+'2 2 2))))
    -> :wat::core::nil
    ((:wat::core::Ok _) (:wat::test::assert-eq true true))
    ((:wat::core::Err _) (:wat::test::assert-eq true false))))

(:wat::test::deftest :wat-tests::std::test::test-macroexpand-fixpoint-evaluates
  ()
  ;; macroexpand returns a :wat::WatAST; hand it to eval-ast!
  ;; to prove the expansion is evaluable.
  (:wat::core::match
    (:wat::eval-ast!
      (:wat::core::macroexpand
        (:wat::core::quote (:wat::core::i64::*'2 3 4))))
    -> :wat::core::nil
    ((:wat::core::Ok _) (:wat::test::assert-eq true true))
    ((:wat::core::Err _) (:wat::test::assert-eq true false))))
