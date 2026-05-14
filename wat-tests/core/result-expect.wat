;; wat-tests/core/result-expect.wat — arc 108 unit tests for
;; `:wat::core::Result/expect`.
;;
;; Form: (:wat::core::Result/expect -> :T <res> <msg>). On
;; `(Ok v)` returns `v`; on `(Err _)` panics with the msg (the
;; carried Err value is discarded — the message names the
;; contract).


;; ─── Ok happy path — i64 ──────────────────────────────────────────────

(:wat::test::deftest :wat-tests::core::result-expect::ok-i64
  ()
  (:wat::core::let
    [res (:wat::core::Ok 99)
     v
      (:wat::core::Result/expect -> :wat::core::i64
        res
        "should be Ok")]
    (:wat::test::assert-eq v 99)))


;; ─── Ok happy path — String ───────────────────────────────────────────

(:wat::test::deftest :wat-tests::core::result-expect::ok-string
  ()
  (:wat::core::let
    [res (:wat::core::Ok "yes")
     v
      (:wat::core::Result/expect -> :wat::core::String
        res
        "should be Ok")]
    (:wat::test::assert-eq v "yes")))


;; ─── Err panics with the supplied message ────────────────────────────

(:wat::test::deftest :wat-tests::core::result-expect::err-panics-with-message
  ()
  (:wat::core::let
    [r
      (:wat::test::run-thread
        (:wat::core::let
          [res (:wat::core::Err "rundb crashed")
           _v
            (:wat::core::Result/expect -> :wat::core::i64
              res
              "expected Ok value")]
          ()))
     fail (:wat::kernel::RunResult/failure r)]
    (:wat::core::match fail -> :wat::core::nil
      ((:wat::core::Some f)
        (:wat::test::assert-eq
          (:wat::kernel::Failure/message f)
          "expected Ok value"))
      (:wat::core::None
        (:wat::kernel::assertion-failed!
          "expected Failure on Err panic, got :None"
          :wat::core::None :wat::core::None)))))
