;; wat-tests/core/result-expect.wat — arc 108 unit tests for
;; `:wat::core::result::expect`.
;;
;; Form: (:wat::core::result::expect -> :T <res> <msg>). On
;; `(Ok v)` returns `v`; on `(Err _)` panics with the msg (the
;; carried Err value is discarded — the message names the
;; contract).


;; ─── Ok happy path — i64 ──────────────────────────────────────────────

(:wat::test::deftest :wat-tests::core::result-expect::ok-i64
  ()
  (:wat::core::let*
    (((res :Result<i64,String>) (Ok 99))
     ((v :wat::core::i64)
      (:wat::core::result::expect -> :wat::core::i64
        res
        "should be Ok")))
    (:wat::test::assert-eq v 99)))


;; ─── Ok happy path — String ───────────────────────────────────────────

(:wat::test::deftest :wat-tests::core::result-expect::ok-string
  ()
  (:wat::core::let*
    (((res :Result<String,i64>) (Ok "yes"))
     ((v :wat::core::String)
      (:wat::core::result::expect -> :wat::core::String
        res
        "should be Ok")))
    (:wat::test::assert-eq v "yes")))


;; ─── Err panics with the supplied message ────────────────────────────

(:wat::test::deftest :wat-tests::core::result-expect::err-panics-with-message
  ()
  (:wat::core::let*
    (((r :wat::kernel::RunResult)
      (:wat::test::run-ast
        (:wat::test::program
          (:wat::core::define
            (:user::main
              (stdin  :wat::io::IOReader)
              (stdout :wat::io::IOWriter)
              (stderr :wat::io::IOWriter)
              -> :())
            (:wat::core::let*
              (((res :Result<i64,String>) (Err "rundb crashed"))
               ((_v :wat::core::i64)
                (:wat::core::result::expect -> :wat::core::i64
                  res
                  "expected Ok value")))
              ())))
        (:wat::core::vec :wat::core::String)))
     ((fail :Option<wat::kernel::Failure>) (:wat::kernel::RunResult/failure r)))
    (:wat::core::match fail -> :()
      ((Some f)
        (:wat::test::assert-eq
          (:wat::kernel::Failure/message f)
          "expected Ok value"))
      (:None
        (:wat::kernel::assertion-failed!
          "expected Failure on Err panic, got :None"
          :None :None)))))
