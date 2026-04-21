;; wat-tests/test-harness.wat — tests for :wat::test::* itself.
;;
;; Written in wat. Each deftest registers a named test function whose
;; last ::-segment starts with `test-`; `wat test wat-tests/` auto-
;; discovers every matching zero-arg `:wat::kernel::RunResult`-returning
;; function, invokes each in random order, reports cargo-test-style.

(:wat::config::set-dims! 1024)
(:wat::config::set-capacity-mode! :error)

;; ─── Tests for assert-eq ──────────────────────────────────────────────

(:wat::test::deftest :wat-tests::harness::test-assert-eq-passes 1024 :error
  (:wat::test::assert-eq 42 42))

(:wat::test::deftest :wat-tests::harness::test-assert-eq-on-strings 1024 :error
  (:wat::test::assert-eq "hello" "hello"))

(:wat::test::deftest :wat-tests::harness::test-assert-eq-on-bools 1024 :error
  (:wat::test::assert-eq true true))

;; ─── Tests for assert-contains ────────────────────────────────────────

(:wat::test::deftest :wat-tests::harness::test-assert-contains-hit 1024 :error
  (:wat::test::assert-contains "the quick brown fox" "quick"))

(:wat::test::deftest :wat-tests::harness::test-assert-contains-start 1024 :error
  (:wat::test::assert-contains "prefix-match" "prefix"))

(:wat::test::deftest :wat-tests::harness::test-assert-contains-end 1024 :error
  (:wat::test::assert-contains "ends-with-suffix" "suffix"))

;; ─── Tests for assert-stdout-is ───────────────────────────────────────
;;
;; The inner program writes two lines; assert-stdout-is compares.

(:wat::test::deftest :wat-tests::harness::test-stdout-is-two-lines 1024 :error
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
;; Exercises the slice-3 substrate gap: `=` on Vec<String>.

(:wat::test::deftest :wat-tests::harness::test-vec-equality 1024 :error
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
