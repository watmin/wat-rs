;; Co-located fixture for probe_arc259_deftest_hermetic_prime.rs — deftest_hermetic_prime_passing_returns.
;; A passing deftest-hermetic': body evaluates correctly; fn must RETURN (not raise).

(:wat::test::deftest-hermetic' :user::passing ()
  (:wat::test::assert-eq 4 (:wat::core::+ 2 2)))

