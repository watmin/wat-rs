;; wat-tests/core/core-nth.wat — stone 118.B4-i runtime coverage for widened `:wat::core::nth`.
;;
;; `nth` used to be `Vector<T>`-only. This strike turns it into a `defclause`: O(1) arms for
;; Vector/PersistentVector/List (unchanged body, once per container that has `get`) plus one
;; `Seqable<T>` arm that walks a Stream with `:wat::stream::next` — the only receiver that arm
;; actually reaches, since the three eager containers all resolve to an earlier O(1) arm first.
;;
;; ★ THE LOAD-BEARING TEST is `nth-on-stream-visits-exactly-i-plus-1-cells` below. The
;; answer/raise tests above it would also pass on an implementation that realizes the whole
;; stream and then indexes it — which would reintroduce the O(n) retention stone B3 just
;; deleted. Only the force count distinguishes a walk from a drain.
;;
;; Grounded on:
;;   - deftest idiom:            wat-tests/core/core-reduce.wat
;;   - container construction:   wat-tests/core/core-seqable.wat (Vector/PersistentVector/List/Stream)
;;   - death-expectation idiom:  wat-tests/core/core-seq-walkers.wat (spawn-peer process + Lost[Panic] match)
;;   - stdout-message-count idiom: wat-tests/test.wat test-assert-stdout-is-matches (chained recv)
;;   - force-shape generator:    wat-scripts/scratch-pad/probe-118B4-forces-per-element-by-walk-shape.wat

;; ═══ row 1 — nth answers on all four containers, at 0 / middle / last ══════════════════════

(:wat::test::deftest :wat-tests::core::core-nth::nth-vector-positions
  (:wat::core::let [v (:wat::core::Vector :wat::core::i64 10 20 30)]
    (:wat::core::do
      (:wat::test::assert-eq (:wat::core::nth v 0) 10)
      (:wat::test::assert-eq (:wat::core::nth v 1) 20)
      (:wat::test::assert-eq (:wat::core::nth v 2) 30))))

(:wat::test::deftest :wat-tests::core::core-nth::nth-persistentvector-positions
  (:wat::core::let [v (:wat::core::PersistentVector 10 20 30)]
    (:wat::core::do
      (:wat::test::assert-eq (:wat::core::nth v 0) 10)
      (:wat::test::assert-eq (:wat::core::nth v 1) 20)
      (:wat::test::assert-eq (:wat::core::nth v 2) 30))))

(:wat::test::deftest :wat-tests::core::core-nth::nth-list-positions
  (:wat::core::let [v (:wat::core::List/of 10 20 30)]
    (:wat::core::do
      (:wat::test::assert-eq (:wat::core::nth v 0) 10)
      (:wat::test::assert-eq (:wat::core::nth v 1) 20)
      (:wat::test::assert-eq (:wat::core::nth v 2) 30))))

(:wat::test::deftest :wat-tests::core::core-nth::nth-stream-positions
  (:wat::core::let
    [s (:wat::stream::cons 10
         (:wat::stream::lazy (:wat::stream::cons 20
           (:wat::stream::lazy (:wat::stream::cons 30
             (:wat::stream::lazy (:wat::stream::empty)))))))]
    (:wat::core::do
      (:wat::test::assert-eq (:wat::core::nth s 0) 10)
      (:wat::test::assert-eq (:wat::core::nth s 1) 20)
      (:wat::test::assert-eq (:wat::core::nth s 2) 30))))

;; ═══ row 2 — past-the-end raises BY NAME, on all four ══════════════════════════════════════
;;
;; `wat/test.wat` has no assert-raises verb, so these use the corpus's own death-expectation
;; idiom (`wat-tests/core/core-seq-walkers.wat`): spawn a peer, let it die, match the panic
;; MESSAGE. Matching the message — not merely "it died" — is what proves the NAMED error rather
;; than any failure at all.

(:wat::test::deftest-hermetic :wat-tests::core::core-nth::nth-past-end-vector-raises
  (:wat::core::let
    [p (:wat::test::spawn-peer (:wat::spawn::process)
         (:wat::core::forms
           (:wat::core::defn :user::main [] -> :wat::core::nil
             (:wat::kernel::println
               (:wat::core::nth (:wat::core::Vector :wat::core::i64 10 20 30) 99)))))
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

(:wat::test::deftest-hermetic :wat-tests::core::core-nth::nth-past-end-persistentvector-raises
  (:wat::core::let
    [p (:wat::test::spawn-peer (:wat::spawn::process)
         (:wat::core::forms
           (:wat::core::defn :user::main [] -> :wat::core::nil
             (:wat::kernel::println
               (:wat::core::nth (:wat::core::PersistentVector 10 20 30) 99)))))
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

(:wat::test::deftest-hermetic :wat-tests::core::core-nth::nth-past-end-list-raises
  (:wat::core::let
    [p (:wat::test::spawn-peer (:wat::spawn::process)
         (:wat::core::forms
           (:wat::core::defn :user::main [] -> :wat::core::nil
             (:wat::kernel::println
               (:wat::core::nth (:wat::core::List/of 10 20 30) 99)))))
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

(:wat::test::deftest-hermetic :wat-tests::core::core-nth::nth-past-end-stream-raises
  (:wat::core::let
    [p (:wat::test::spawn-peer (:wat::spawn::process)
         (:wat::core::forms
           (:wat::core::defn :user::main [] -> :wat::core::nil
             (:wat::kernel::println
               (:wat::core::nth
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

;; ═══ row 3 (★ load-bearing) — nth on a Stream visits exactly i+1 cells ═════════════════════
;;
;; Built from `wat-scripts/scratch-pad/probe-118B4-forces-per-element-by-walk-shape.wat`'s
;; `:user::gen`, which prints "FORCED" once per cell realization (no memo — 118.B3 deleted both,
;; so every independent force of the same cell prints again). `nth s 4` on an infinite counting
;; stream starting at 0 must force cells 0..4 inclusive (5 = i+1) and answer 4 — never more,
;; never fewer. `expect-forced` recv's exactly 5 "FORCED" messages; if the walk forced fewer, the
;; 5th expectation would receive the answer "4" instead and fail; if it forced more, the final
;; assert-eq below would receive "FORCED" instead of "4" and fail. Only the exact count passes
;; both.

;; ⚠ Fully inline, no sibling/nested `defn` helper: a `deftest-hermetic` body is shipped to a
;; forked child as its own closure — sibling top-level file `defn`s (proven with the file-scope
;; `recv-str`/`expect-forced` helpers first drafted here) came back `UnresolvedReferences`, and
;; hoisting them into the body's own `do`-prefix (the documented declaration-lift path) still
;; failed the same way, because `split_body_prelude` matches the RAW pre-macroexpansion head
;; keyword against `is_declaration_form`, which lists `:wat::core::def` but not
;; `:wat::core::defn` — so a leading `defn` is never recognized as a liftable declaration and
;; is never registered in the child. Six chained `recv`s, matching the house pattern
;; (`wat-tests/test.wat` `test-assert-stdout-is-matches`, `wat-tests/core/core-seq-walkers.wat`
;; death idiom), is the proven-safe shape.
(:wat::test::deftest-hermetic :wat-tests::core::core-nth::nth-on-stream-visits-exactly-i-plus-1-cells
  (:wat::core::let
    [p (:wat::test::spawn-peer (:wat::spawn::process)
         (:wat::core::forms
           (:wat::core::defn :user::gen [n <- :wat::core::i64] -> :wat::stream::Stream<wat::core::i64>
             (:wat::stream::lazy
               (:wat::core::do
                 (:wat::kernel::println "FORCED")
                 (:wat::stream::cons n (:user::gen (:wat::core::+ n 1))))))
           (:wat::core::defn :user::main [] -> :wat::core::nil
             (:wat::core::do
               (:wat::kernel::println (:wat::core::str (:wat::core::nth (:user::gen 0) 4)))
               nil))))
     m0 (:wat::core::match (:wat::kernel::recv p)
          ((:wat::kernel::RecvOutcome::Message m) m)
          ((:wat::kernel::RecvOutcome::Lost cause)
            (:wat::kernel::assertion-failed! (:wat::kernel::LociDiedError/message cause) :wat::core::None :wat::core::None))
          (:wat::kernel::RecvOutcome::Stopped
            (:wat::kernel::assertion-failed! "force-count: stopped before message 0" :wat::core::None :wat::core::None))
          (:wat::kernel::RecvOutcome::Closed
            (:wat::kernel::assertion-failed! "force-count: closed before message 0" :wat::core::None :wat::core::None)))
     _c0 (:wat::test::assert-eq m0 "FORCED")
     m1 (:wat::core::match (:wat::kernel::recv p)
          ((:wat::kernel::RecvOutcome::Message m) m)
          ((:wat::kernel::RecvOutcome::Lost cause)
            (:wat::kernel::assertion-failed! (:wat::kernel::LociDiedError/message cause) :wat::core::None :wat::core::None))
          (:wat::kernel::RecvOutcome::Stopped
            (:wat::kernel::assertion-failed! "force-count: stopped before message 1" :wat::core::None :wat::core::None))
          (:wat::kernel::RecvOutcome::Closed
            (:wat::kernel::assertion-failed! "force-count: closed before message 1" :wat::core::None :wat::core::None)))
     _c1 (:wat::test::assert-eq m1 "FORCED")
     m2 (:wat::core::match (:wat::kernel::recv p)
          ((:wat::kernel::RecvOutcome::Message m) m)
          ((:wat::kernel::RecvOutcome::Lost cause)
            (:wat::kernel::assertion-failed! (:wat::kernel::LociDiedError/message cause) :wat::core::None :wat::core::None))
          (:wat::kernel::RecvOutcome::Stopped
            (:wat::kernel::assertion-failed! "force-count: stopped before message 2" :wat::core::None :wat::core::None))
          (:wat::kernel::RecvOutcome::Closed
            (:wat::kernel::assertion-failed! "force-count: closed before message 2" :wat::core::None :wat::core::None)))
     _c2 (:wat::test::assert-eq m2 "FORCED")
     m3 (:wat::core::match (:wat::kernel::recv p)
          ((:wat::kernel::RecvOutcome::Message m) m)
          ((:wat::kernel::RecvOutcome::Lost cause)
            (:wat::kernel::assertion-failed! (:wat::kernel::LociDiedError/message cause) :wat::core::None :wat::core::None))
          (:wat::kernel::RecvOutcome::Stopped
            (:wat::kernel::assertion-failed! "force-count: stopped before message 3" :wat::core::None :wat::core::None))
          (:wat::kernel::RecvOutcome::Closed
            (:wat::kernel::assertion-failed! "force-count: closed before message 3" :wat::core::None :wat::core::None)))
     _c3 (:wat::test::assert-eq m3 "FORCED")
     m4 (:wat::core::match (:wat::kernel::recv p)
          ((:wat::kernel::RecvOutcome::Message m) m)
          ((:wat::kernel::RecvOutcome::Lost cause)
            (:wat::kernel::assertion-failed! (:wat::kernel::LociDiedError/message cause) :wat::core::None :wat::core::None))
          (:wat::kernel::RecvOutcome::Stopped
            (:wat::kernel::assertion-failed! "force-count: stopped before message 4" :wat::core::None :wat::core::None))
          (:wat::kernel::RecvOutcome::Closed
            (:wat::kernel::assertion-failed! "force-count: closed before message 4" :wat::core::None :wat::core::None)))
     _c4 (:wat::test::assert-eq m4 "FORCED")
     ;; the 6th recv must be the ANSWER, not a 6th "FORCED" — that is the "no more than i+1" half.
     m5 (:wat::core::match (:wat::kernel::recv p)
          ((:wat::kernel::RecvOutcome::Message m) m)
          ((:wat::kernel::RecvOutcome::Lost cause)
            (:wat::kernel::assertion-failed! (:wat::kernel::LociDiedError/message cause) :wat::core::None :wat::core::None))
          (:wat::kernel::RecvOutcome::Stopped
            (:wat::kernel::assertion-failed! "force-count: stopped before the answer" :wat::core::None :wat::core::None))
          (:wat::kernel::RecvOutcome::Closed
            (:wat::kernel::assertion-failed! "force-count: closed before the answer" :wat::core::None :wat::core::None)))]
    (:wat::test::assert-eq m5 "4")))
