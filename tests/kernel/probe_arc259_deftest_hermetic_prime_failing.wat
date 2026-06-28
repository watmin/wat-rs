;; Co-located fixture for probe_arc259_deftest_hermetic_prime.rs — deftest_hermetic_prime_failing_raises_with_message.
;; A failing deftest-hermetic': body raises; message must surface via process Err channel.

(:wat::test::deftest-hermetic' :user::failing ()
  (:wat::kernel::assertion-failed! "HERMETIC-FAIL-SENTINEL" :wat::core::None :wat::core::None))

(:wat::core::defn :user::main [] -> :wat::core::nil nil)
