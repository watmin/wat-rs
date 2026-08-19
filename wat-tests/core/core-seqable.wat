;; wat-tests/core/core-seqable.wat — stone 118.B1 runtime coverage for `:wat::core::Seqable<T>`.
;;
;; `Seqable<T>` is the type the seven `<verb>-stream` twins were a workaround for. Builder,
;; 2026-07-31: *"The twins are a workaround for the missing type, not a pattern."* Route B was
;; ruled 2026-08-17 (`docs/arc/2026/04/118-lazy-seqs-vs-threaded-streams/DECISIONS-118.B-four-questioned.md`).
;;
;; ★ THE LOAD-BEARING TEST IS `generic-fn-over-seqable-accepts-all-four`. Everything above it
;; only proves the four `extend-type`s registered. What 118.3-B actually had to fix was that a
;; concrete builtin would not unify against a PARAMETRIC surface parameter — so a test that
;; DECLARES a `Seqable<T>`-typed fn without CALLING it proves nothing. That exact mistake was made
;; on 2026-08-17 and reported as "the full design type-checks"; adding call sites made it RED four
;; times over. This test calls it with all four containers.
;;
;; Grounded on:
;;   - deftest idiom:      wat-tests/core/core-arithmetic.wat
;;   - the surface + impls: wat/seq.wat (top of file)
;;   - the four heads:      src/collection/infer.rs:665 `extract_lazyable_elem`

;; ─── the surface resolves on each of the four containers, order-preserving ──────────────────

(:wat::test::deftest :wat-tests::core::core-seqable::seq-of-vector
  (:wat::core::let [out (:wat::core::into [] (:wat::core::Seqable/seq
                          (:wat::core::Vector :wat::core::i64 1 2 3)))]
    (:wat::test::assert-eq (:wat::core::string::join "," out) "1,2,3")))

(:wat::test::deftest :wat-tests::core::core-seqable::seq-of-persistentvector
  (:wat::core::let [out (:wat::core::into [] (:wat::core::Seqable/seq
                          (:wat::core::PersistentVector 1 2 3 4)))]
    (:wat::test::assert-eq (:wat::core::string::join "," out) "1,2,3,4")))

(:wat::test::deftest :wat-tests::core::core-seqable::seq-of-list
  (:wat::core::let [out (:wat::core::into [] (:wat::core::Seqable/seq
                          (:wat::core::List/of 1 2 3 4 5)))]
    (:wat::test::assert-eq (:wat::core::string::join "," out) "1,2,3,4,5")))

(:wat::test::deftest :wat-tests::core::core-seqable::seq-of-stream
  (:wat::core::let [out (:wat::core::into [] (:wat::core::Seqable/seq
                          (:wat::stream::cons 7
                            (:wat::stream::lazy
                              (:wat::stream::cons 8
                                (:wat::stream::lazy (:wat::stream::empty)))))))]
    (:wat::test::assert-eq (:wat::core::string::join "," out) "7,8")))

;; ─── ★ THE STONE — one generic fn over ANY Seqable<T>, CALLED with all four ─────────────────
;;
;; This is the shape route B exists for: after B2 every sequence verb in the stdlib looks like
;; this, and so does a user's. Under the old world it would need five `defclause` arms plus a
;; `-stream` twin.

(:wat::core::defn :wat-tests::core::core-seqable::count-via-seq<T>
  [s <- :wat::core::Seqable<T>] -> :wat::core::i64
  (:wat::core::length (:wat::core::into [] (:wat::core::Seqable/seq s))))

(:wat::test::deftest :wat-tests::core::core-seqable::generic-fn-over-seqable-accepts-all-four
  (:wat::core::do
    (:wat::test::assert-eq
      (:wat-tests::core::core-seqable::count-via-seq (:wat::core::Vector :wat::core::i64 1 2 3)) 3)
    (:wat::test::assert-eq
      (:wat-tests::core::core-seqable::count-via-seq (:wat::core::PersistentVector 1 2 3 4)) 4)
    (:wat::test::assert-eq
      (:wat-tests::core::core-seqable::count-via-seq (:wat::core::List/of 1 2 3 4 5)) 5)
    (:wat::test::assert-eq
      (:wat-tests::core::core-seqable::count-via-seq
        (:wat::stream::cons 1
          (:wat::stream::lazy
            (:wat::stream::cons 2
              (:wat::stream::lazy (:wat::stream::empty)))))) 2)))

;; ─── laziness: `seq` must not force the chain ───────────────────────────────────────────────
;;
;; The source is INFINITE. A materialising `seq` would never return, so this test passing at all
;; is the assertion — `assert-eq` is only there to pin the values it yielded.

(:wat::core::defn :wat-tests::core::core-seqable::nat
  [i <- :wat::core::i64] -> :wat::stream::Stream<wat::core::i64>
  (:wat::stream::lazy
    (:wat::stream::cons i (:wat-tests::core::core-seqable::nat (:wat::core::+ i 1)))))

(:wat::test::deftest :wat-tests::core::core-seqable::seq-of-infinite-stream-stays-lazy
  (:wat::core::let [out (:wat::core::into []
                          (:wat::core::take
                            (:wat::core::Seqable/seq (:wat-tests::core::core-seqable::nat 0)) 3))]
    (:wat::test::assert-eq (:wat::core::string::join "," out) "0,1,2")))
