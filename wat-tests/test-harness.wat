;; wat-tests/test_harness.wat — tests for :wat::test::* itself.
;;
;; Written in wat. Each deftest registers a named test function that,
;; when invoked, returns a RunResult. The :user::main below invokes
;; each, inspects the Failure slot, writes "<name>:PASS" or
;; "<name>:FAIL" to stdout. A Rust runner (tests/wat_tests_dir.rs)
;; asserts every line ends in :PASS.
;;
;; When wat test lands (arc 007 slice 4), this same file becomes
;; input to `wat test wat-tests/` — the CLI discovers each
;; deftest by iterating registered functions rather than requiring
;; the hand-written :user::main at the bottom.

(:wat::config::set-dims! 1024)
(:wat::config::set-capacity-mode! :error)

;; ─── Tests for assert-eq ──────────────────────────────────────────────

(:wat::test::deftest :wat-tests::harness::assert-eq-passes 1024 :error
  (:wat::test::assert-eq 42 42))

(:wat::test::deftest :wat-tests::harness::assert-eq-on-strings 1024 :error
  (:wat::test::assert-eq "hello" "hello"))

(:wat::test::deftest :wat-tests::harness::assert-eq-on-bools 1024 :error
  (:wat::test::assert-eq true true))

;; ─── Tests for assert-contains ────────────────────────────────────────

(:wat::test::deftest :wat-tests::harness::assert-contains-hit 1024 :error
  (:wat::test::assert-contains "the quick brown fox" "quick"))

(:wat::test::deftest :wat-tests::harness::assert-contains-start 1024 :error
  (:wat::test::assert-contains "prefix-match" "prefix"))

(:wat::test::deftest :wat-tests::harness::assert-contains-end 1024 :error
  (:wat::test::assert-contains "ends-with-suffix" "suffix"))

;; ─── Tests for assert-stdout-is ───────────────────────────────────────
;;
;; The inner program writes two lines; assert-stdout-is compares.

(:wat::test::deftest :wat-tests::harness::stdout-is-two-lines 1024 :error
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
     ((expected :Vec<String>)
      (:wat::core::conj
        (:wat::core::conj (:wat::core::vec :String) "alpha")
        "beta")))
    (:wat::test::assert-stdout-is inner expected)))

;; ─── Structural equality via :wat::core::= ────────────────────────────
;;
;; Exercises the slice-3 substrate gap we fixed: `=` on Vec<String>.

(:wat::test::deftest :wat-tests::harness::vec-equality 1024 :error
  (:wat::core::let*
    (((a :Vec<String>)
      (:wat::core::conj
        (:wat::core::conj (:wat::core::vec :String) "x")
        "y"))
     ((b :Vec<String>)
      (:wat::core::conj
        (:wat::core::conj (:wat::core::vec :String) "x")
        "y")))
    (:wat::test::assert-eq a b)))

;; ─── :user::main — invokes every test, writes line per result ─────────

(:wat::core::define
  (:wat-tests::harness::report-one
    (name :String)
    (result :wat::kernel::RunResult)
    (stdout :wat::io::IOWriter)
    -> :())
  (:wat::core::let*
    (((fail :Option<wat::kernel::Failure>)
      (:wat::kernel::RunResult/failure result)))
    (:wat::core::match fail -> :()
      ((Some _) (:wat::io::IOWriter/println stdout
                  (:wat::core::string::join ":"
                    (:wat::core::conj
                      (:wat::core::conj (:wat::core::vec :String) name)
                      "FAIL"))))
      (:None    (:wat::io::IOWriter/println stdout
                  (:wat::core::string::join ":"
                    (:wat::core::conj
                      (:wat::core::conj (:wat::core::vec :String) name)
                      "PASS")))))))

(:wat::core::define
  (:user::main
    (stdin  :wat::io::IOReader)
    (stdout :wat::io::IOWriter)
    (stderr :wat::io::IOWriter)
    -> :())
  (:wat::core::let*
    (((_ :()) (:wat-tests::harness::report-one
                "assert-eq-passes"
                (:wat-tests::harness::assert-eq-passes) stdout))
     ((_ :()) (:wat-tests::harness::report-one
                "assert-eq-on-strings"
                (:wat-tests::harness::assert-eq-on-strings) stdout))
     ((_ :()) (:wat-tests::harness::report-one
                "assert-eq-on-bools"
                (:wat-tests::harness::assert-eq-on-bools) stdout))
     ((_ :()) (:wat-tests::harness::report-one
                "assert-contains-hit"
                (:wat-tests::harness::assert-contains-hit) stdout))
     ((_ :()) (:wat-tests::harness::report-one
                "assert-contains-start"
                (:wat-tests::harness::assert-contains-start) stdout))
     ((_ :()) (:wat-tests::harness::report-one
                "assert-contains-end"
                (:wat-tests::harness::assert-contains-end) stdout))
     ((_ :()) (:wat-tests::harness::report-one
                "stdout-is-two-lines"
                (:wat-tests::harness::stdout-is-two-lines) stdout)))
    (:wat-tests::harness::report-one
      "vec-equality"
      (:wat-tests::harness::vec-equality) stdout)))
