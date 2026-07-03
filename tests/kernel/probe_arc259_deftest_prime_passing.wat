;; Co-located fixture for probe_arc259_deftest_prime.rs — deftest_prime_passing_returns.
;; A passing deftest': body evaluates correctly; fn must RETURN (not raise).

(:wat::test::deftest' :user::passing ()
  (:wat::test::assert-eq 4 (:wat::core::+ 2 2)))

