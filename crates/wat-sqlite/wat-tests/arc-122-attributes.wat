;; arc 122 end-to-end smoke test — verifies the per-test attribute
;; mechanism works through cargo test.
;;
;; Three deftests:
;;   - test-arc-122-plain: no annotation → runs normally
;;   - test-arc-122-ignored: preceded by (:wat::test::ignore ...) →
;;     emitted as #[test] #[ignore = "..."]; cargo test skips by default
;;   - test-arc-122-should-panic: preceded by
;;     (:wat::test::should-panic ...) → emitted as
;;     #[test] #[should_panic(expected = "...")]; the deftest's
;;     intentional assertion failure satisfies the should-panic
;;     contract.

(:wat::test::deftest :wat-tests::sqlite::arc-122::test-arc-122-plain
  ()
  (:wat::test::assert-eq 42 42))

(:wat::test::ignore "verifies #[ignore] attribute lands; runs only with cargo test --ignored")
(:wat::test::deftest :wat-tests::sqlite::arc-122::test-arc-122-ignored
  ()
  ;; If this ran without #[ignore], it would fail (1 != 2). Since
  ;; cargo skips it by default, the suite still passes.
  (:wat::test::assert-eq 1 2))

(:wat::test::should-panic "wat-arc-122-should-panic-marker")
(:wat::test::deftest :wat-tests::sqlite::arc-122::test-arc-122-should-panic
  ()
  ;; Deliberate panic with a message libtest's #[should_panic(expected
  ;; = "...")] will match. The substring matching means as long as the
  ;; assertion message contains the marker, the should-panic test is
  ;; reported as passing.
  (:wat::test::assert-eq "wat-arc-122-should-panic-marker" "different"))
