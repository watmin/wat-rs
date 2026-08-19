;; wat-tests/core/core-nth-differential.wat — stone 118.B4-0's DIFFERENTIAL: the native
;; `:wat::core::nth` (`src/runtime.rs`, `eval_nth`) must agree with its wat specification
;; `:wat::core::nth-spec` (`wat/core.wat`) on every input.
;;
;; ★ THE SHAPE, same as `wat-tests/core/core-foldl-spec.wat` (stone 118.B6) and the recorded
;; exemplar `:wat::rete::insert-all-spec` / `insert-all` (`wat/rete.wat:1508`): `nth` is a Rust
;; intrinsic; `nth-spec` is the SAME thing written in wat as obviously as possible — correct and
;; slow on purpose. "the native kernel is the fast impl, the spec keeps it honest."
;;
;; ⚠ `nth-spec` MUST NEVER delegate to `nth`. A spec that calls its subject proves nothing.
;; `[[feedback_a_green_test_can_prove_nothing]]` / `[[feedback_an_oracle_must_be_written_in_the_other_language]]`
;;
;; ═══ in-range agreement — Vector / PersistentVector / List / Stream, at 0 / mid / last ═══════
;;
;; Direct value comparison: `nth` and `nth-spec` both succeed on these inputs, so `assert-eq`
;; between the two calls IS the differential.

(:wat::test::deftest :wat-tests::core::core-nth-differential::agree-on-vector
  (:wat::core::let [v (:wat::core::Vector :wat::core::i64 10 20 30 40 50)]
    (:wat::core::do
      (:wat::test::assert-eq (:wat::core::nth v 0) (:wat::core::nth-spec v 0))
      (:wat::test::assert-eq (:wat::core::nth v 2) (:wat::core::nth-spec v 2))
      (:wat::test::assert-eq (:wat::core::nth v 4) (:wat::core::nth-spec v 4)))))

(:wat::test::deftest :wat-tests::core::core-nth-differential::agree-on-persistentvector
  (:wat::core::let [v (:wat::core::PersistentVector 10 20 30 40 50)]
    (:wat::core::do
      (:wat::test::assert-eq (:wat::core::nth v 0) (:wat::core::nth-spec v 0))
      (:wat::test::assert-eq (:wat::core::nth v 2) (:wat::core::nth-spec v 2))
      (:wat::test::assert-eq (:wat::core::nth v 4) (:wat::core::nth-spec v 4)))))

(:wat::test::deftest :wat-tests::core::core-nth-differential::agree-on-list
  (:wat::core::let [v (:wat::core::List/of 10 20 30 40 50)]
    (:wat::core::do
      (:wat::test::assert-eq (:wat::core::nth v 0) (:wat::core::nth-spec v 0))
      (:wat::test::assert-eq (:wat::core::nth v 2) (:wat::core::nth-spec v 2))
      (:wat::test::assert-eq (:wat::core::nth v 4) (:wat::core::nth-spec v 4)))))

(:wat::test::deftest :wat-tests::core::core-nth-differential::agree-on-stream
  (:wat::core::let
    [mk (:wat::core::fn [] -> :wat::stream::Stream<wat::core::i64>
          (:wat::stream::cons 10
            (:wat::stream::lazy (:wat::stream::cons 20
              (:wat::stream::lazy (:wat::stream::cons 30
                (:wat::stream::lazy (:wat::stream::cons 40
                  (:wat::stream::lazy (:wat::stream::cons 50
                    (:wat::stream::lazy (:wat::stream::empty))))))))))))]
    (:wat::core::do
      (:wat::test::assert-eq (:wat::core::nth (mk) 0) (:wat::core::nth-spec (mk) 0))
      (:wat::test::assert-eq (:wat::core::nth (mk) 2) (:wat::core::nth-spec (mk) 2))
      (:wat::test::assert-eq (:wat::core::nth (mk) 4) (:wat::core::nth-spec (mk) 4)))))

;; ═══ out-of-range agreement — both raise the SAME message ═════════════════════════════════════
;;
;; `wat-tests/core/core-nth.wat`'s `nth-past-end-*-raises` rows already establish that the native
;; `nth` raises exactly "nth: index out of range" on Vector/PersistentVector/List/Stream. These
;; four rows establish the other half: `nth-spec` raises the SAME literal message on the SAME
;; four receivers — together the two files prove native and oracle agree on the raise, not just
;; on the happy path. Death-expectation idiom per `wat-tests/core/core-seq-walkers.wat`.

(:wat::test::deftest-hermetic :wat-tests::core::core-nth-differential::nth-spec-past-end-vector-raises
  (:wat::core::let
    [p (:wat::test::spawn-peer (:wat::spawn::process)
         (:wat::core::forms
           (:wat::core::defn :user::main [] -> :wat::core::nil
             (:wat::kernel::println
               (:wat::core::nth-spec (:wat::core::Vector :wat::core::i64 10 20 30) 99)))))
     msg (:wat::core::match (:wat::kernel::recv p)
           ((:wat::kernel::RecvOutcome::Message _m)
             (:wat::kernel::assertion-failed! "expected Lost[Panic], got Message" :wat::core::None :wat::core::None))
           ((:wat::kernel::RecvOutcome::Lost cause)
             (:wat::core::match cause
               ((:wat::kernel::LociDiedError::Panic message _failure) message)
               (_ (:wat::kernel::assertion-failed! "expected Lost[Panic], got other Lost" :wat::core::None :wat::core::None))))
           (:wat::kernel::RecvOutcome::Stopped
             (:wat::kernel::assertion-failed! "expected Lost[Panic], got Stopped" :wat::core::None :wat::core::None))
           (:wat::kernel::RecvOutcome::Closed
             (:wat::kernel::assertion-failed! "expected Lost[Panic], got Closed" :wat::core::None :wat::core::None)))]
    (:wat::test::assert-true (:wat::core::regex::matches? "nth: index out of range" msg))))

(:wat::test::deftest-hermetic :wat-tests::core::core-nth-differential::nth-spec-past-end-persistentvector-raises
  (:wat::core::let
    [p (:wat::test::spawn-peer (:wat::spawn::process)
         (:wat::core::forms
           (:wat::core::defn :user::main [] -> :wat::core::nil
             (:wat::kernel::println
               (:wat::core::nth-spec (:wat::core::PersistentVector 10 20 30) 99)))))
     msg (:wat::core::match (:wat::kernel::recv p)
           ((:wat::kernel::RecvOutcome::Message _m)
             (:wat::kernel::assertion-failed! "expected Lost[Panic], got Message" :wat::core::None :wat::core::None))
           ((:wat::kernel::RecvOutcome::Lost cause)
             (:wat::core::match cause
               ((:wat::kernel::LociDiedError::Panic message _failure) message)
               (_ (:wat::kernel::assertion-failed! "expected Lost[Panic], got other Lost" :wat::core::None :wat::core::None))))
           (:wat::kernel::RecvOutcome::Stopped
             (:wat::kernel::assertion-failed! "expected Lost[Panic], got Stopped" :wat::core::None :wat::core::None))
           (:wat::kernel::RecvOutcome::Closed
             (:wat::kernel::assertion-failed! "expected Lost[Panic], got Closed" :wat::core::None :wat::core::None)))]
    (:wat::test::assert-true (:wat::core::regex::matches? "nth: index out of range" msg))))

(:wat::test::deftest-hermetic :wat-tests::core::core-nth-differential::nth-spec-past-end-list-raises
  (:wat::core::let
    [p (:wat::test::spawn-peer (:wat::spawn::process)
         (:wat::core::forms
           (:wat::core::defn :user::main [] -> :wat::core::nil
             (:wat::kernel::println
               (:wat::core::nth-spec (:wat::core::List/of 10 20 30) 99)))))
     msg (:wat::core::match (:wat::kernel::recv p)
           ((:wat::kernel::RecvOutcome::Message _m)
             (:wat::kernel::assertion-failed! "expected Lost[Panic], got Message" :wat::core::None :wat::core::None))
           ((:wat::kernel::RecvOutcome::Lost cause)
             (:wat::core::match cause
               ((:wat::kernel::LociDiedError::Panic message _failure) message)
               (_ (:wat::kernel::assertion-failed! "expected Lost[Panic], got other Lost" :wat::core::None :wat::core::None))))
           (:wat::kernel::RecvOutcome::Stopped
             (:wat::kernel::assertion-failed! "expected Lost[Panic], got Stopped" :wat::core::None :wat::core::None))
           (:wat::kernel::RecvOutcome::Closed
             (:wat::kernel::assertion-failed! "expected Lost[Panic], got Closed" :wat::core::None :wat::core::None)))]
    (:wat::test::assert-true (:wat::core::regex::matches? "nth: index out of range" msg))))

(:wat::test::deftest-hermetic :wat-tests::core::core-nth-differential::nth-spec-past-end-stream-raises
  (:wat::core::let
    [p (:wat::test::spawn-peer (:wat::spawn::process)
         (:wat::core::forms
           (:wat::core::defn :user::main [] -> :wat::core::nil
             (:wat::kernel::println
               (:wat::core::nth-spec
                 (:wat::stream::cons 10
                   (:wat::stream::lazy (:wat::stream::cons 20
                     (:wat::stream::lazy (:wat::stream::cons 30
                       (:wat::stream::lazy (:wat::stream::empty)))))))
                 99)))))
     msg (:wat::core::match (:wat::kernel::recv p)
           ((:wat::kernel::RecvOutcome::Message _m)
             (:wat::kernel::assertion-failed! "expected Lost[Panic], got Message" :wat::core::None :wat::core::None))
           ((:wat::kernel::RecvOutcome::Lost cause)
             (:wat::core::match cause
               ((:wat::kernel::LociDiedError::Panic message _failure) message)
               (_ (:wat::kernel::assertion-failed! "expected Lost[Panic], got other Lost" :wat::core::None :wat::core::None))))
           (:wat::kernel::RecvOutcome::Stopped
             (:wat::kernel::assertion-failed! "expected Lost[Panic], got Stopped" :wat::core::None :wat::core::None))
           (:wat::kernel::RecvOutcome::Closed
             (:wat::kernel::assertion-failed! "expected Lost[Panic], got Closed" :wat::core::None :wat::core::None)))]
    (:wat::test::assert-true (:wat::core::regex::matches? "nth: index out of range" msg))))
