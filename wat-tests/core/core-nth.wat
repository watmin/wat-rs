;; wat-tests/core/core-nth.wat — stone 118.B4-i runtime coverage for widened `:wat::core::nth`.
;;
;; `nth` used to be `(Vector :- [T])`-only. Stone 118.B4-i turned it into a `defclause`: O(1) arms for
;; Vector/PersistentVector/List (unchanged body, once per container that has `get`) plus a
;; `(Seqable :- [T])` arm that walked a Stream with `:wat::stream::next`.
;;
;; ⛔ STONE 118.B4-iii — THE WALL (2026-08-18) removes the Stream receiver: `nth` on a Stream was
;; O(i) via that walk, identical syntax to the O(1) Vector case — a complexity lie the wall exists
;; to close. `nth`'s receiver set is now Vector / PersistentVector / List only. The three rows this
;; removed — `nth-stream-positions`, `nth-past-end-stream-raises`, and the ★ load-bearing
;; `nth-on-stream-visits-exactly-i-plus-1-cells` (which measured that the walk visits exactly i+1
;; cells, not a drain) — are GONE, not weakened: `(nth stream i)` no longer type-checks at all, so
;; there is nothing left for those assertions to measure. Confirmed at the wall: `(nth s i)` on a
;; Stream is refused with "a lazy Stream<T> has no O(1) nth — use (drop s i) then
;; :wat::stream::next" (`src/check.rs::infer_nth`). Every eager row below is UNCHANGED.
;;
;; Grounded on:
;;   - deftest idiom:            wat-tests/core/core-reduce.wat
;;   - container construction:   wat-tests/core/core-seqable.wat (Vector/PersistentVector/List/Stream)
;;   - death-expectation idiom:  wat-tests/core/core-seq-walkers.wat (spawn-peer process + Lost[Panic] match)
;;   - stdout-message-count idiom: wat-tests/test.wat test-assert-stdout-is-matches (chained recv)

;; ═══ row 1 — nth answers on all three eager containers, at 0 / middle / last ═══════════════

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
  (:wat::core::let [v (:wat::core::List 10 20 30)]
    (:wat::core::do
      (:wat::test::assert-eq (:wat::core::nth v 0) 10)
      (:wat::test::assert-eq (:wat::core::nth v 1) 20)
      (:wat::test::assert-eq (:wat::core::nth v 2) 30))))

;; ═══ row 2 — past-the-end raises BY NAME, on all three ═════════════════════════════════════
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
    (:wat::test::assert-true (:wat::regex::matches? "nth: index out of range" msg))))

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
    (:wat::test::assert-true (:wat::regex::matches? "nth: index out of range" msg))))

(:wat::test::deftest-hermetic :wat-tests::core::core-nth::nth-past-end-list-raises
  (:wat::core::let
    [p (:wat::test::spawn-peer (:wat::spawn::process)
         (:wat::core::forms
           (:wat::core::defn :user::main [] -> :wat::core::nil
             (:wat::kernel::println
               (:wat::core::nth (:wat::core::List 10 20 30) 99)))))
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
    (:wat::test::assert-true (:wat::regex::matches? "nth: index out of range" msg))))

;; ═══ row 3 — RETIRED by stone 118.B4-iii (THE WALL) ════════════════════════════════════════
;;
;; This used to be `nth-past-end-stream-raises` and, ★ load-bearing, `nth-on-stream-visits-
;; exactly-i-plus-1-cells` — the test that measured `(nth s i)` on a Stream forces exactly `i+1`
;; cells (a walk, not a drain). Both rows are GONE, not weakened: `(nth stream i)` no longer
;; type-checks, so nothing is left for either assertion to measure. The property they proved is
;; superseded, not lost — `wat-scripts/scratch-pad/probe-118B4-forces-per-element-by-walk-shape.wat`
;; (walk A) still proves a Stream is force-honest via `:wat::stream::next`, the one door left.
