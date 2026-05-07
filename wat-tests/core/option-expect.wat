;; wat-tests/core/option-expect.wat — arc 108 unit tests for
;; `:wat::core::Option/expect`.
;;
;; Form: (:wat::core::Option/expect -> :T <opt> <msg>) — type
;; declared at HEAD position before any value producer (parallels
;; `match`'s `-> :T` placement, but the VALUE-producing role of the
;; opt-expr puts the type ahead of it). On `(Some v)` returns `v`;
;; on `:None` panics with the msg.
;;
;; Pass cases: deftests that exercise the Some-arm.
;; Fail cases: run the panic path inside `:wat::test::run-ast` so
;; the surrounding catch_unwind surfaces the AssertionPayload as a
;; `Failure` on the inner RunResult; the outer deftest matches on it.


;; ─── Some happy path — i64 ────────────────────────────────────────────

(:wat::test::deftest :wat-tests::core::option-expect::some-i64
  ()
  (:wat::core::let
    ((opt (:wat::core::Some 42))
     (v
      (:wat::core::Option/expect -> :wat::core::i64
        opt
        "should be Some")))
    (:wat::test::assert-eq v 42)))


;; ─── Some happy path — String ─────────────────────────────────────────

(:wat::test::deftest :wat-tests::core::option-expect::some-string
  ()
  (:wat::core::let
    ((opt (:wat::core::Some "hello"))
     (v
      (:wat::core::Option/expect -> :wat::core::String
        opt
        "should be Some")))
    (:wat::test::assert-eq v "hello")))


;; ─── Some happy path — nested :wat::core::Option<wat::core::Option<wat::core::i64>> ────────────────────

(:wat::test::deftest :wat-tests::core::option-expect::some-nested-option
  ()
  (:wat::core::let
    ((opt (:wat::core::Some (:wat::core::Some 7)))
     (inner
      (:wat::core::Option/expect -> :wat::core::Option<wat::core::i64>
        opt
        "outer should be Some"))
     (v
      (:wat::core::Option/expect -> :wat::core::i64
        inner
        "inner should be Some")))
    (:wat::test::assert-eq v 7)))


;; ─── :None panics with the supplied message ──────────────────────────

(:wat::test::deftest :wat-tests::core::option-expect::none-panics-with-message
  ()
  (:wat::core::let
    ((r
      (:wat::test::run-ast
        (:wat::test::program
          (:wat::core::define
            (:user::main
              (stdin  :wat::io::IOReader)
              (stdout :wat::io::IOWriter)
              (stderr :wat::io::IOWriter)
              -> :wat::core::nil)
            (:wat::core::let
              ((opt :wat::core::None)
               (_v
                (:wat::core::Option/expect -> :wat::core::i64
                  opt
                  "broker disconnected")))
              ())))
        (:wat::core::Vector :wat::core::String)))
     (fail (:wat::kernel::RunResult/failure r)))
    (:wat::core::match fail -> :wat::core::nil
      ((:wat::core::Some f)
        (:wat::test::assert-eq
          (:wat::kernel::Failure/message f)
          "broker disconnected"))
      (:wat::core::None
        (:wat::kernel::assertion-failed!
          "expected Failure on :None panic, got :None"
          :wat::core::None :wat::core::None)))))
