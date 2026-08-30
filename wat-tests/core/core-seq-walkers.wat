;; wat-tests/core/core-seq-walkers.wat — stone 118.B2b runtime coverage for the five verbs whose
;; three-call Stream walk this stone removed: `remove`, `take-while`, `drop-while`, `take-nth`,
;; `reductions`.
;;
;; ⛔ WHY THIS FILE EXISTS. Before this stone these five verbs had almost NO runtime coverage —
;; `remove` / `drop-while` / `reductions` had none at all, and `take-while` had exactly one test
;; (`tests/types/probe_arc118_2z_takewhile_lazy.rs`, a laziness probe). Their bodies were rewritten
;; from a `first`/`rest`/`empty?` walk to a single-force `:wat::stream::next` pull, and a green
;; floor would have said nothing about whether they still behaved. The baseline was captured first
;; (`wat-scripts/scratch-pad/probe-118B-six-walkers-baseline.wat`, run against HEAD BEFORE the
;; migration) and every expectation below is that measured baseline, pinned so it cannot drift.
;;
;; ★ `take-nth-0-repeats-the-head` IS THE LOAD-BEARING ONE. It is the only test in the corpus that
;; can catch the trap this stone was one line away from shipping — see the verb's comment in
;; wat/seq.wat.
;;
;; Grounded on: wat-tests/core/core-seqable.wat (the deftest idiom + the four-container shape).

;; ─── an infinite source, for the laziness rows ─────────────────────────────────────────────────
(:wat::core::defn :wat-tests::core::core-seq-walkers::nat
  [i <- :wat::core::i64] -> (:wat::stream::Stream :- [:wat::core::i64])
  (:wat::stream::lazy
    (:wat::stream::cons i (:wat-tests::core::core-seq-walkers::nat (:wat::core::+ i 1)))))

;; ─── the lazy sources ──────────────────────────────────────────────────────────────────────────
;; The fourth "container" each verb is exercised over is a REAL lazy stage, not a re-wrapped
;; Vector. `(map identity v)` returns a genuine `(Stream :- [i64])`, so these rows test the composition
;; that actually matters — a lazy verb consuming another lazy verb's output, which is precisely
;; where a re-forcing walker would run the upstream's user code more than once.
;;
;; ⚠ NOT `(Seqable/seq v)`. That was the obvious spelling and it does NOT type-check: `Seqable/seq`
;; is declared `[self <- (Seqable :- [T])] -> (Stream :- [T])`, and calling it on a concrete `(Vector :- [i64])`
;; yields `(Stream :- [T])` with T UNBOUND — the surface method drops the instantiation, so the result
;; will not satisfy a concrete `(Seqable :- [i64])` downstream. Recorded in
;; `NOTE-118.B2b-two-doors-the-checker-opened-and-the-runtime-did-not.md`; it is pre-existing (B1
;; minted the surface; nothing had yet fed a surface-method RESULT into a concrete consumer).

(:wat::core::defn :wat-tests::core::core-seq-walkers::identity
  [x <- :wat::core::i64] -> :wat::core::i64 x)

(:wat::core::defn :wat-tests::core::core-seq-walkers::lazy-six [] -> (:wat::stream::Stream :- [:wat::core::i64])
  (:wat::core::map :wat-tests::core::core-seq-walkers::identity
    (:wat::core::Vector :- [:wat::core::i64] 1 2 3 4 5 6)))

(:wat::core::defn :wat-tests::core::core-seq-walkers::lazy-1234-12 [] -> (:wat::stream::Stream :- [:wat::core::i64])
  (:wat::core::map :wat-tests::core::core-seq-walkers::identity
    (:wat::core::Vector :- [:wat::core::i64] 1 2 3 4 1 2)))

(:wat::core::defn :wat-tests::core::core-seq-walkers::lazy-four [] -> (:wat::stream::Stream :- [:wat::core::i64])
  (:wat::core::map :wat-tests::core::core-seq-walkers::identity
    (:wat::core::Vector :- [:wat::core::i64] 1 2 3 4)))

;; ─── remove — all four containers, plus the bare PersistentVector the old 5th arm served ───────

(:wat::test::deftest :wat-tests::core::core-seq-walkers::remove-over-every-container
  (:wat::core::let
    [pred (:wat::core::fn [x <- :wat::core::i64] -> :wat::core::bool
            (:wat::core::= 0 (:wat::core::mod x 2)))]
    (:wat::core::do
      (:wat::test::assert-eq
        (:wat::string::join ","
          (:wat::core::into [] (:wat::core::remove pred (:wat::core::Vector :- [:wat::core::i64] 1 2 3 4 5 6))))
        "1,3,5")
      (:wat::test::assert-eq
        (:wat::string::join ","
          (:wat::core::into [] (:wat::core::remove pred (:wat::core::List 1 2 3 4 5 6))))
        "1,3,5")
      (:wat::test::assert-eq
        (:wat::string::join ","
          (:wat::core::into [] (:wat::core::remove pred (:wat::core::PersistentVector 1 2 3 4 5 6))))
        "1,3,5")
      (:wat::test::assert-eq
        (:wat::string::join ","
          (:wat::core::into []
            (:wat::core::remove pred (:wat-tests::core::core-seq-walkers::lazy-six))))
        "1,3,5"))))

;; `remove` must not realize past what the consumer pulls — the source here is INFINITE, so this
;; test terminating at all is the assertion.
(:wat::test::deftest :wat-tests::core::core-seq-walkers::remove-stays-lazy-over-an-infinite-source
  (:wat::test::assert-eq
    (:wat::string::join ","
      (:wat::core::into []
        (:wat::core::take
          (:wat::core::remove
            (:wat::core::fn [x <- :wat::core::i64] -> :wat::core::bool
              (:wat::core::= 0 (:wat::core::mod x 2)))
            (:wat-tests::core::core-seq-walkers::nat 0))
          4)))
    "1,3,5,7"))

;; ─── take-while ────────────────────────────────────────────────────────────────────────────────

(:wat::test::deftest :wat-tests::core::core-seq-walkers::take-while-over-every-container
  (:wat::core::let
    [pred (:wat::core::fn [x <- :wat::core::i64] -> :wat::core::bool (:wat::core::< x 4))]
    (:wat::core::do
      (:wat::test::assert-eq
        (:wat::string::join ","
          (:wat::core::into [] (:wat::core::take-while pred (:wat::core::Vector :- [:wat::core::i64] 1 2 3 4 1 2))))
        "1,2,3")
      (:wat::test::assert-eq
        (:wat::string::join ","
          (:wat::core::into [] (:wat::core::take-while pred (:wat::core::List 1 2 3 4 1 2))))
        "1,2,3")
      (:wat::test::assert-eq
        (:wat::string::join ","
          (:wat::core::into [] (:wat::core::take-while pred (:wat::core::PersistentVector 1 2 3 4 1 2))))
        "1,2,3")
      (:wat::test::assert-eq
        (:wat::string::join ","
          (:wat::core::into []
            (:wat::core::take-while pred (:wat-tests::core::core-seq-walkers::lazy-1234-12))))
        "1,2,3"))))

;; take-while over an INFINITE source: it must stop at the first false without ever forcing the
;; cell after it. (The stronger form of this — the skipped cell DIVIDES BY ZERO — is
;; `tests/types/probe_arc118_2z_takewhile_lazy.rs`.)
(:wat::test::deftest :wat-tests::core::core-seq-walkers::take-while-terminates-on-an-infinite-source
  (:wat::test::assert-eq
    (:wat::string::join ","
      (:wat::core::into []
        (:wat::core::take-while
          (:wat::core::fn [x <- :wat::core::i64] -> :wat::core::bool (:wat::core::< x 3))
          (:wat-tests::core::core-seq-walkers::nat 0))))
    "0,1,2"))

;; ─── drop-while ────────────────────────────────────────────────────────────────────────────────
;; The remainder must come back UNCHANGED — including elements that would fail `pred` again later
;; (`1,2` at the tail). A walker that kept filtering instead of stopping would return "4" alone.

(:wat::test::deftest :wat-tests::core::core-seq-walkers::drop-while-over-every-container
  (:wat::core::let
    [pred (:wat::core::fn [x <- :wat::core::i64] -> :wat::core::bool (:wat::core::< x 4))]
    (:wat::core::do
      (:wat::test::assert-eq
        (:wat::string::join ","
          (:wat::core::into [] (:wat::core::drop-while pred (:wat::core::Vector :- [:wat::core::i64] 1 2 3 4 1 2))))
        "4,1,2")
      (:wat::test::assert-eq
        (:wat::string::join ","
          (:wat::core::into [] (:wat::core::drop-while pred (:wat::core::List 1 2 3 4 1 2))))
        "4,1,2")
      (:wat::test::assert-eq
        (:wat::string::join ","
          (:wat::core::into [] (:wat::core::drop-while pred (:wat::core::PersistentVector 1 2 3 4 1 2))))
        "4,1,2")
      (:wat::test::assert-eq
        (:wat::string::join ","
          (:wat::core::into []
            (:wat::core::drop-while pred (:wat-tests::core::core-seq-walkers::lazy-1234-12))))
        "4,1,2"))))

;; ─── take-nth ──────────────────────────────────────────────────────────────────────────────────

(:wat::test::deftest :wat-tests::core::core-seq-walkers::take-nth-over-every-container
  (:wat::core::do
    (:wat::test::assert-eq
      (:wat::string::join ","
        (:wat::core::into [] (:wat::core::take-nth 2 (:wat::core::Vector :- [:wat::core::i64] 1 2 3 4 5 6))))
      "1,3,5")
    (:wat::test::assert-eq
      (:wat::string::join ","
        (:wat::core::into [] (:wat::core::take-nth 2 (:wat::core::List 1 2 3 4 5 6))))
      "1,3,5")
    (:wat::test::assert-eq
      (:wat::string::join ","
        (:wat::core::into [] (:wat::core::take-nth 2 (:wat::core::PersistentVector 1 2 3 4 5 6))))
      "1,3,5")
    (:wat::test::assert-eq
      (:wat::string::join ","
        (:wat::core::into []
          (:wat::core::take-nth 2 (:wat-tests::core::core-seq-walkers::lazy-six))))
      "1,3,5")
    ;; n = 1 is every element — the control that separates "take-nth works" from "n=0 is special".
    (:wat::test::assert-eq
      (:wat::string::join ","
        (:wat::core::into [] (:wat::core::take-nth 1 (:wat::core::Vector :- [:wat::core::i64] 1 2 3))))
      "1,2,3")))

;; ★★ THE TRAP, PINNED. `(take-nth 0 coll)` is an INFINITE repeat of the head — clojure's own
;; behaviour, and what wat did before this stone (measured against HEAD, not assumed). The obvious
;; `next`-based rewrite — emit `value`, recurse on `(drop rest (- n 1))` — silently turns this into
;; "1,2,3", and NOTHING else in the corpus would notice: `take-nth` has no caller outside a scratch
;; probe. The `take` is what keeps this test finite; without it the stream never ends.
(:wat::test::deftest :wat-tests::core::core-seq-walkers::take-nth-0-repeats-the-head
  (:wat::test::assert-eq
    (:wat::string::join ","
      (:wat::core::into []
        (:wat::core::take (:wat::core::take-nth 0 (:wat::core::Vector :- [:wat::core::i64] 1 2 3)) 5)))
    "1,1,1,1,1"))

(:wat::test::deftest :wat-tests::core::core-seq-walkers::take-nth-stays-lazy-over-an-infinite-source
  (:wat::test::assert-eq
    (:wat::string::join ","
      (:wat::core::into []
        (:wat::core::take (:wat::core::take-nth 3 (:wat-tests::core::core-seq-walkers::nat 0)) 4)))
    "0,3,6,9"))

;; ─── reductions — both arities, every container ────────────────────────────────────────────────

(:wat::test::deftest :wat-tests::core::core-seq-walkers::reductions-3arity-over-every-container
  (:wat::core::let
    [f (:wat::core::fn [a <- :wat::core::i64 b <- :wat::core::i64] -> :wat::core::i64
         (:wat::core::+ a b))]
    (:wat::core::do
      (:wat::test::assert-eq
        (:wat::string::join ","
          (:wat::core::into [] (:wat::core::reductions f 0 (:wat::core::Vector :- [:wat::core::i64] 1 2 3 4))))
        "0,1,3,6,10")
      (:wat::test::assert-eq
        (:wat::string::join ","
          (:wat::core::into [] (:wat::core::reductions f 0 (:wat::core::List 1 2 3 4))))
        "0,1,3,6,10")
      (:wat::test::assert-eq
        (:wat::string::join ","
          (:wat::core::into [] (:wat::core::reductions f 0 (:wat::core::PersistentVector 1 2 3 4))))
        "0,1,3,6,10")
      (:wat::test::assert-eq
        (:wat::string::join ","
          (:wat::core::into []
            (:wat::core::reductions f 0 (:wat-tests::core::core-seq-walkers::lazy-four))))
        "0,1,3,6,10"))))

(:wat::test::deftest :wat-tests::core::core-seq-walkers::reductions-2arity-over-every-container
  (:wat::core::let
    [f (:wat::core::fn [a <- :wat::core::i64 b <- :wat::core::i64] -> :wat::core::i64
         (:wat::core::+ a b))]
    (:wat::core::do
      (:wat::test::assert-eq
        (:wat::string::join ","
          (:wat::core::into [] (:wat::core::reductions f (:wat::core::Vector :- [:wat::core::i64] 1 2 3 4))))
        "1,3,6,10")
      (:wat::test::assert-eq
        (:wat::string::join ","
          (:wat::core::into [] (:wat::core::reductions f (:wat::core::List 1 2 3 4))))
        "1,3,6,10")
      (:wat::test::assert-eq
        (:wat::string::join ","
          (:wat::core::into [] (:wat::core::reductions f (:wat::core::PersistentVector 1 2 3 4))))
        "1,3,6,10")
      (:wat::test::assert-eq
        (:wat::string::join ","
          (:wat::core::into []
            (:wat::core::reductions f (:wat-tests::core::core-seq-walkers::lazy-four))))
        "1,3,6,10"))))

;; `reductions` is a LAZY producer — an infinite source must yield a prefix without diverging.
;; (This is the row that would go red if the walker ever became eager.)
(:wat::test::deftest :wat-tests::core::core-seq-walkers::reductions-stays-lazy-over-an-infinite-source
  (:wat::test::assert-eq
    (:wat::string::join ","
      (:wat::core::into []
        (:wat::core::take
          (:wat::core::reductions
            (:wat::core::fn [a <- :wat::core::i64 b <- :wat::core::i64] -> :wat::core::i64
              (:wat::core::+ a b))
            0
            (:wat-tests::core::core-seq-walkers::nat 1))
          5)))
    "0,1,3,6,10"))

;; ─── ★★ THE NEGATIVE CONTROL — reductions/2 on EMPTY raises, BY NAME, for both kinds ───────────
;;
;; Before this stone `reductions`' own comment claimed an empty collection "raises via `first`'s
;; out-of-range failure". Measured against HEAD that was true for a Vector and FALSE for a Stream:
;; `first` on an exhausted Stream returns a bare `nil` (the tracked B5 hole), so the 2-arity Stream
;; arm silently produced a one-element stream containing nil — a caller folding an empty pipeline
;; got a WRONG ANSWER instead of an error.
;;
;; `wat/test.wat` has no assert-raises verb, so these use the corpus's own death-expectation idiom
;; (grounded on `wat-tests/test.wat` test-assert-stderr-matches-pass): spawn a peer, let it die, and
;; match the panic MESSAGE. Matching the message — not merely "it died" — is what makes these prove
;; the NAMED error rather than any failure at all.
;;
;; The Vector row is the arm that already raised, so its green says the migration did not LOSE an
;; existing guarantee. The Stream row is the arm that did not, so its green is the fix. Non-vacuity
;; for both lives in `reductions-2arity-over-every-container` above — a `reductions` that raised
;; unconditionally would satisfy the two rows below and fail that one.

(:wat::test::deftest-hermetic :wat-tests::core::core-seq-walkers::reductions-2arity-on-empty-vector-raises
  (:wat::core::let
    [p (:wat::test::spawn-peer (:wat::spawn::process)
         (:wat::core::forms
           (:wat::core::defn :user::main [] -> :wat::core::nil
             (:wat::kernel::println
               (:wat::core::length
                 (:wat::core::into []
                   (:wat::core::reductions
                     (:wat::core::fn [a <- :wat::core::i64 b <- :wat::core::i64] -> :wat::core::i64
                       (:wat::core::+ a b))
                     (:wat::core::Vector :- [:wat::core::i64]))))))))
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
    (:wat::test::assert-true
      (:wat::regex::matches? "reductions: the 2-arity form needs at least one element" msg))))

(:wat::test::deftest-hermetic :wat-tests::core::core-seq-walkers::reductions-2arity-on-empty-stream-raises
  (:wat::core::let
    [p (:wat::test::spawn-peer (:wat::spawn::process)
         (:wat::core::forms
           (:wat::core::defn :user::main [] -> :wat::core::nil
             (:wat::kernel::println
               (:wat::core::length
                 (:wat::core::into []
                   (:wat::core::reductions
                     (:wat::core::fn [a <- :wat::core::i64 b <- :wat::core::i64] -> :wat::core::i64
                       (:wat::core::+ a b))
                     (:wat::stream::empty))))))))
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
    (:wat::test::assert-true
      (:wat::regex::matches? "reductions: the 2-arity form needs at least one element" msg))))
