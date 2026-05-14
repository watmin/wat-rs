;; wat-tests/run-thread.wat — Layer 1 verification for :wat::test::run-thread.
;;
;; Arc 170 slice 4a-α (task #308). Cheap-thread counterpart to the
;; existing :wat::test::run-hermetic. The macro spawns a thread via
;; :wat::kernel::spawn-thread, joins via :wat::kernel::Thread/join-result,
;; and surfaces panics as a structured Failure in RunResult.failure.
;;
;; Two paths exercised:
;;
;;   Ok-path  — body runs a passing assertion; outer asserts
;;              RunResult.failure is :None.
;;
;;   Err-path — body runs a FAILING assertion; outer asserts
;;              RunResult.failure is :Some(_). This is the load-bearing
;;              proof that ThreadDiedError -> Failure conversion works
;;              through Thread/join-result's chain branch. Without this,
;;              the next stone (4a-β sweep) can't trust panic
;;              propagation through the new macro.
;;
;; Each deftest body executes inside its own deftest sandbox (currently
;; run-hermetic per test.wat:294-303). The INNER program is what
;; exercises run-thread.

;; ─── Ok-path: passing assertion inside run-thread ─────────────────────

(:wat::test::deftest :wat-tests::std::test::run-thread-ok-path
  ()
  (:wat::core::let
    [result (:wat::test::run-thread
              (:wat::test::assert-eq 4 (:wat::core::i64::+'2 2 2)))]
    (:wat::core::match (:wat::kernel::RunResult/failure result)
      -> :wat::core::nil
      (:wat::core::None :wat::core::nil)
      ((:wat::core::Some _f)
       (:wat::kernel::assertion-failed!
         "Ok-path: expected :None but got :Some — passing assertion was misclassified as failure"
         :wat::core::None :wat::core::None)))))

;; ─── Err-path: failing assertion inside run-thread ────────────────────

(:wat::test::deftest :wat-tests::std::test::run-thread-err-path
  ()
  (:wat::core::let
    [result (:wat::test::run-thread
              (:wat::test::assert-eq 99 (:wat::core::i64::+'2 2 2)))]
    (:wat::core::match (:wat::kernel::RunResult/failure result)
      -> :wat::core::nil
      ((:wat::core::Some _f) :wat::core::nil)
      (:wat::core::None
       (:wat::kernel::assertion-failed!
         "Err-path: expected :Some failure but got :None — chain handling broken"
         :wat::core::None :wat::core::None)))))
