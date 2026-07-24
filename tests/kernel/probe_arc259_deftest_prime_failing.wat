;; Co-located fixture for probe_arc259_deftest_prime.rs — deftest_prime_failing_raises_with_message.
;; A failing deftest': body raises; message must surface via pipe (S3.5a-0 IPC fix).

(:wat::test::deftest :user::failing 
  (:wat::kernel::assertion-failed! "DEFTEST-FAIL-SENTINEL" :wat::core::None :wat::core::None))

