;; wat-tests/core/core-stream-materializers-differential.wat — stone 118.B5's DIFFERENTIAL: the
;; native `:wat::core::stream->vec` / `:wat::core::stream->pvec` (`src/collection/transform.rs`,
;; `eval_stream_to_vec` / `eval_stream_to_pvec`) must agree with their wat specifications
;; `:wat::core::stream->vec-spec` / `:wat::core::stream->pvec-spec` (`wat/seq.wat`) on every input.
;;
;; ★ THE SHAPE, same as `wat-tests/core/core-nth-differential.wat` (stone 118.B4-0) and the
;; recorded exemplar `:wat::rete::insert-all-spec` / `insert-all` (`wat/rete.wat:1508`):
;; `stream->vec`/`stream->pvec` are Rust intrinsics; `-spec` is the SAME thing written in wat as
;; obviously as possible — correct and slow on purpose. "the native kernel is the fast impl, the
;; spec keeps it honest."
;;
;; ⚠ `stream->vec-spec`/`stream->pvec-spec` MUST NEVER delegate to the native they specify — a
;; spec that calls its subject proves nothing (`wat/seq.wat`'s own ⚠ notes on both defns).
;; `[[feedback_a_green_test_can_prove_nothing]]` / `[[feedback_an_oracle_must_be_written_in_the_other_language]]`
;;
;; Two receivers ((Vector :- [T]) / (PersistentVector :- [T])) × three sizes (empty / one / many) = the
;; ACCEPTANCE table's six rows, plus two seeded-accumulator rows (`into`'s actual contract: `to`
;; is not always freshly empty) and one genuinely-lazy-producer row per receiver (not merely a
;; `map`-wrapped Vector) so the differential does not only ever see one Stream SHAPE.

;; ─── the lazy sources ──────────────────────────────────────────────────────────────────────────
;; `(map identity v)` — a genuine `(Stream :- [i64])` over an already-resident Vector, same idiom
;; `wat-tests/core/core-seq-walkers.wat` uses. `identity` keeps the walk itself under test, not
;; some other transform's correctness.

(:wat::core::defn :wat-tests::core::core-stream-materializers-differential::identity
  [x <- :wat::core::i64] -> :wat::core::i64 x)

(:wat::core::defn :wat-tests::core::core-stream-materializers-differential::stream-of
  [xs <- (:wat::core::Vector :- [:wat::core::i64])] -> (:wat::stream::Stream :- [:wat::core::i64])
  (:wat::core::map :wat-tests::core::core-stream-materializers-differential::identity xs))

;; A genuinely lazy producer — no backing container anywhere, each cell built on force. Used for
;; the "not just a map-wrapped Vector" rows below.
(:wat::core::defn :wat-tests::core::core-stream-materializers-differential::counter
  [i <- :wat::core::i64 limit <- :wat::core::i64] -> (:wat::stream::Stream :- [:wat::core::i64])
  (:wat::core::if (:wat::core::>= i limit)
    (:wat::stream::empty)
    (:wat::stream::lazy
      (:wat::stream::cons i
        (:wat-tests::core::core-stream-materializers-differential::counter (:wat::core::+ i 1) limit)))))

;; ═══ stream->vec — (Vector :- [T]) receiver, fresh-empty seed ═══════════════════════════════════════

(:wat::test::deftest :wat-tests::core::core-stream-materializers-differential::vec-agree-empty
  (:wat::test::assert-eq
    (:wat::core::stream->vec (:wat::core::Vector :wat::core::i64)
      (:wat-tests::core::core-stream-materializers-differential::stream-of (:wat::core::Vector :wat::core::i64)))
    (:wat::core::stream->vec-spec (:wat::core::Vector :wat::core::i64)
      (:wat-tests::core::core-stream-materializers-differential::stream-of (:wat::core::Vector :wat::core::i64)))))

(:wat::test::deftest :wat-tests::core::core-stream-materializers-differential::vec-agree-one
  (:wat::test::assert-eq
    (:wat::core::stream->vec (:wat::core::Vector :wat::core::i64)
      (:wat-tests::core::core-stream-materializers-differential::stream-of (:wat::core::Vector :wat::core::i64 42)))
    (:wat::core::stream->vec-spec (:wat::core::Vector :wat::core::i64)
      (:wat-tests::core::core-stream-materializers-differential::stream-of (:wat::core::Vector :wat::core::i64 42)))))

(:wat::test::deftest :wat-tests::core::core-stream-materializers-differential::vec-agree-many
  (:wat::test::assert-eq
    (:wat::core::stream->vec (:wat::core::Vector :wat::core::i64)
      (:wat-tests::core::core-stream-materializers-differential::stream-of
        (:wat::core::Vector :wat::core::i64 1 2 3 4 5 6 7 8 9 10)))
    (:wat::core::stream->vec-spec (:wat::core::Vector :wat::core::i64)
      (:wat-tests::core::core-stream-materializers-differential::stream-of
        (:wat::core::Vector :wat::core::i64 1 2 3 4 5 6 7 8 9 10)))))

;; ─── seeded accumulator — `into`'s actual contract: `to` is not always fresh-empty ─────────────

(:wat::test::deftest :wat-tests::core::core-stream-materializers-differential::vec-agree-seeded
  (:wat::test::assert-eq
    (:wat::core::stream->vec (:wat::core::Vector :wat::core::i64 100 200)
      (:wat-tests::core::core-stream-materializers-differential::stream-of
        (:wat::core::Vector :wat::core::i64 1 2 3)))
    (:wat::core::stream->vec-spec (:wat::core::Vector :wat::core::i64 100 200)
      (:wat-tests::core::core-stream-materializers-differential::stream-of
        (:wat::core::Vector :wat::core::i64 1 2 3)))))

;; ─── the genuinely lazy producer, not a re-wrapped Vector ──────────────────────────────────────

(:wat::test::deftest :wat-tests::core::core-stream-materializers-differential::vec-agree-lazy-producer
  (:wat::test::assert-eq
    (:wat::core::stream->vec (:wat::core::Vector :wat::core::i64)
      (:wat-tests::core::core-stream-materializers-differential::counter 0 25))
    (:wat::core::stream->vec-spec (:wat::core::Vector :wat::core::i64)
      (:wat-tests::core::core-stream-materializers-differential::counter 0 25))))

;; ═══ stream->pvec — (PersistentVector :- [T]) receiver, fresh-empty seed ═════════════════════════════

(:wat::test::deftest :wat-tests::core::core-stream-materializers-differential::pvec-agree-empty
  (:wat::test::assert-eq
    (:wat::core::stream->pvec (:wat::core::PersistentVector)
      (:wat-tests::core::core-stream-materializers-differential::stream-of (:wat::core::Vector :wat::core::i64)))
    (:wat::core::stream->pvec-spec (:wat::core::PersistentVector)
      (:wat-tests::core::core-stream-materializers-differential::stream-of (:wat::core::Vector :wat::core::i64)))))

(:wat::test::deftest :wat-tests::core::core-stream-materializers-differential::pvec-agree-one
  (:wat::test::assert-eq
    (:wat::core::stream->pvec (:wat::core::PersistentVector)
      (:wat-tests::core::core-stream-materializers-differential::stream-of (:wat::core::Vector :wat::core::i64 42)))
    (:wat::core::stream->pvec-spec (:wat::core::PersistentVector)
      (:wat-tests::core::core-stream-materializers-differential::stream-of (:wat::core::Vector :wat::core::i64 42)))))

(:wat::test::deftest :wat-tests::core::core-stream-materializers-differential::pvec-agree-many
  (:wat::test::assert-eq
    (:wat::core::stream->pvec (:wat::core::PersistentVector)
      (:wat-tests::core::core-stream-materializers-differential::stream-of
        (:wat::core::Vector :wat::core::i64 1 2 3 4 5 6 7 8 9 10)))
    (:wat::core::stream->pvec-spec (:wat::core::PersistentVector)
      (:wat-tests::core::core-stream-materializers-differential::stream-of
        (:wat::core::Vector :wat::core::i64 1 2 3 4 5 6 7 8 9 10)))))

(:wat::test::deftest :wat-tests::core::core-stream-materializers-differential::pvec-agree-seeded
  (:wat::test::assert-eq
    (:wat::core::stream->pvec (:wat::core::PersistentVector 9 8)
      (:wat-tests::core::core-stream-materializers-differential::stream-of
        (:wat::core::Vector :wat::core::i64 1 2 3)))
    (:wat::core::stream->pvec-spec (:wat::core::PersistentVector 9 8)
      (:wat-tests::core::core-stream-materializers-differential::stream-of
        (:wat::core::Vector :wat::core::i64 1 2 3)))))

(:wat::test::deftest :wat-tests::core::core-stream-materializers-differential::pvec-agree-lazy-producer
  (:wat::test::assert-eq
    (:wat::core::stream->pvec (:wat::core::PersistentVector)
      (:wat-tests::core::core-stream-materializers-differential::counter 0 25))
    (:wat::core::stream->pvec-spec (:wat::core::PersistentVector)
      (:wat-tests::core::core-stream-materializers-differential::counter 0 25))))

;; ═══ into itself — the call sites don't move; this proves the public verb still agrees end-to-end ═

(:wat::test::deftest :wat-tests::core::core-stream-materializers-differential::into-vec-matches-spec
  (:wat::test::assert-eq
    (:wat::core::into (:wat::core::Vector :wat::core::i64)
      (:wat-tests::core::core-stream-materializers-differential::stream-of
        (:wat::core::Vector :wat::core::i64 1 2 3 4 5)))
    (:wat::core::stream->vec-spec (:wat::core::Vector :wat::core::i64)
      (:wat-tests::core::core-stream-materializers-differential::stream-of
        (:wat::core::Vector :wat::core::i64 1 2 3 4 5)))))

(:wat::test::deftest :wat-tests::core::core-stream-materializers-differential::into-pvec-matches-spec
  (:wat::test::assert-eq
    (:wat::core::into (:wat::core::PersistentVector)
      (:wat-tests::core::core-stream-materializers-differential::stream-of
        (:wat::core::Vector :wat::core::i64 1 2 3 4 5)))
    (:wat::core::stream->pvec-spec (:wat::core::PersistentVector)
      (:wat-tests::core::core-stream-materializers-differential::stream-of
        (:wat::core::Vector :wat::core::i64 1 2 3 4 5)))))
