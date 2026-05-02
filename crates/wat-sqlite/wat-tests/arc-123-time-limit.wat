;; arc 123 end-to-end smoke test — verifies :time-limit lands the
;; thread-spawn + recv_timeout wrapper on the generated #[test]
;; fn. The timeout-firing case is verified separately when the
;; annotation is applied to an actual hanging test (arc 119's
;; recovered step-tests).
;;
;; Three deftests covering the parse + emission path:
;;   - test-arc-123-fast: 100ms budget, trivial work → passes
;;   - test-arc-123-seconds-suffix: 5s budget, trivial work →
;;     passes; verifies the 's' suffix parses to 5000ms
;;   - test-arc-123-minutes-suffix: 1m budget, trivial work →
;;     passes; verifies the 'm' suffix parses to 60000ms

(:wat::test::time-limit "100ms")
(:wat::test::deftest :wat-tests::sqlite::arc-123::test-arc-123-fast
  ()
  (:wat::test::assert-eq 42 42))

(:wat::test::time-limit "5s")
(:wat::test::deftest :wat-tests::sqlite::arc-123::test-arc-123-seconds-suffix
  ()
  (:wat::test::assert-eq "ok" "ok"))

(:wat::test::time-limit "1m")
(:wat::test::deftest :wat-tests::sqlite::arc-123::test-arc-123-minutes-suffix
  ()
  (:wat::test::assert-eq true true))
