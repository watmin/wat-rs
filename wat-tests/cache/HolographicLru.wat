;; wat-tests/cache/HolographicLru.wat — arc 278 Cache Stone 3 acceptance gate.
;;
;; `:wat::cache::HolographicLru` composes a `Hologram` (the similarity index, holds VALUES) with
;; Stone 1's `(Lru :- [HolonAST nil])` (the recency/bound index, holds KEYS ONLY — the `nil` value slot
;; means the LRU stores nothing of its own; the values live in the Hologram). The LRU exists so
;; the Hologram cannot grow without limit.
;;
;; THE LOAD-BEARING INVARIANT — dual eviction: when `put` overflows the LRU, the displaced key
;; must ALSO be removed from the Hologram. `test-dual-eviction` below is the sharp assertion — it
;; proves the evicted key is a MISS from the Hologram, not merely absent from the LRU. Verified
;; empirically this session: sabotaging `HolographicLru::put` to skip the `Hologram/remove` call
;; sends `test-dual-eviction` AND `test-get-bumps-recency` red, and `test-len-agrees-with-bound`
;; reports 4 instead of 3 (the Hologram growing unbounded while the LRU believes it is bounded —
;; exactly the failure mode this gate exists to catch).

;; ─── similarity, not equality — a coincident but DIFFERENT probe hits ───────────────────────
;;
;; Put under a Thermometer key at 50.0; probe with 50.01 — a structurally different HolonAST
;; (different literal value), coincident by cosine. If this only ever probed with the exact
;; stored key, it would prove nothing about the "Holographic" half of the name.
(:wat::test::deftest :wat-tests::cache::HolographicLru::test-similarity-not-equality
  (:wat::core::let
    [store
      (:wat::cache::HolographicLru::new (:wat::holon::filter-coincident) 10)
     k (:wat::holon::Thermometer 50.0 0.0 100.0)
     v (:wat::holon::leaf :answer-for-fifty)
     _ (:wat::cache::HolographicLru::put store k v)
     probe (:wat::holon::Thermometer 50.01 0.0 100.0)
     got (:wat::cache::HolographicLru::get store probe)]
    (:wat::test::assert-eq got (:wat::core::Some v))))

;; ─── ★ dual eviction — the one that catches a real bug ──────────────────────────────────────
;;
;; Capacity 2; insert 3 distinct keys — `:c` overflows the LRU and evicts `:a`. Assert `:a` is
;; gone from the HOLOGRAM (a `get` miss), not merely absent from the LRU's own bookkeeping. `:b`
;; and `:c` must still be present.
(:wat::test::deftest :wat-tests::cache::HolographicLru::test-dual-eviction
  (:wat::core::let
    [store (:wat::cache::HolographicLru::new (:wat::holon::filter-coincident) 2)
     a (:wat::holon::leaf :a)
     b (:wat::holon::leaf :b)
     c (:wat::holon::leaf :c)
     _ (:wat::cache::HolographicLru::put store a (:wat::holon::leaf :val-a))
     _ (:wat::cache::HolographicLru::put store b (:wat::holon::leaf :val-b))
     _ (:wat::cache::HolographicLru::put store c (:wat::holon::leaf :val-c))
     got-a (:wat::cache::HolographicLru::get store a)
     got-b (:wat::cache::HolographicLru::get store b)
     got-c (:wat::cache::HolographicLru::get store c)]
    (:wat::test::assert-eq got-a :wat::core::None)
    (:wat::test::assert-eq got-b (:wat::core::Some (:wat::holon::leaf :val-b)))
    (:wat::test::assert-eq got-c (:wat::core::Some (:wat::holon::leaf :val-c)))))

;; ─── get bumps recency — a hit feeds the LRU's ordering, not just the Hologram's ────────────
;;
;; Put A then B at cap 2; `get A` (bumping it to MRU); put C — B is evicted, not A, proving
;; `Hologram/find`'s matched-key return actually drives the LRU bump inside `get`.
(:wat::test::deftest :wat-tests::cache::HolographicLru::test-get-bumps-recency
  (:wat::core::let
    [store (:wat::cache::HolographicLru::new (:wat::holon::filter-coincident) 2)
     a (:wat::holon::leaf :a)
     b (:wat::holon::leaf :b)
     c (:wat::holon::leaf :c)
     _ (:wat::cache::HolographicLru::put store a (:wat::holon::leaf :val-a))
     _ (:wat::cache::HolographicLru::put store b (:wat::holon::leaf :val-b))
     _ (:wat::cache::HolographicLru::get store a)
     _ (:wat::cache::HolographicLru::put store c (:wat::holon::leaf :val-c))
     got-a (:wat::cache::HolographicLru::get store a)
     got-b (:wat::cache::HolographicLru::get store b)
     got-c (:wat::cache::HolographicLru::get store c)]
    (:wat::test::assert-eq got-a (:wat::core::Some (:wat::holon::leaf :val-a)))
    (:wat::test::assert-eq got-b :wat::core::None)
    (:wat::test::assert-eq got-c (:wat::core::Some (:wat::holon::leaf :val-c)))))

;; ─── the Match record itself — read by name, not position ─────────────────────────────────
;;
;; The four tests above exercise `Hologram/find` only THROUGH `HolographicLru::get`, which
;; immediately destructures it and discards the record. This test is the one direct gate on
;; `:wat::holon::Match` itself: probe with a coincident-but-different value and read the
;; matched key back by NAME (`Match/key`) — proving it is the STORED key, not the probe — and
;; the value by name (`Match/value`). A tuple could not express this assertion legibly; it is
;; the reason the record exists. Targets `Hologram/find` directly.
(:wat::test::deftest :wat-tests::cache::HolographicLru::test-find-returns-match-record
  (:wat::core::let
    [store (:wat::cache::HolographicLru::new (:wat::holon::filter-coincident) 10)
     k (:wat::holon::Thermometer 50.0 0.0 100.0)
     v (:wat::holon::leaf :answer-for-fifty)
     _ (:wat::cache::HolographicLru::put store k v)
     probe (:wat::holon::Thermometer 50.01 0.0 100.0)
     hologram (:wat::cache::HolographicLru/hologram store)]
    (:wat::core::match (:wat::holon::Hologram/find hologram probe)
      ((:wat::core::Some m)
        (:wat::core::let
          [matched-key (:wat::holon::Match/key m)
           matched-val (:wat::holon::Match/value m)]
          (:wat::test::assert-eq matched-key k)
          (:wat::test::assert-eq matched-val v)))
      (:wat::core::None (:wat::test::assert-eq :expected-a-match :got-none)))))

;; ─── len agrees with the bound after overflow ───────────────────────────────────────────────
;;
;; Cap 3, insert 4 distinct keys — `len` (read via the Hologram, the value-holding half) must
;; still report 3, proving the Hologram's population tracks the LRU's bound, not the raw put
;; count.
(:wat::test::deftest :wat-tests::cache::HolographicLru::test-len-agrees-with-bound
  (:wat::core::let
    [store (:wat::cache::HolographicLru::new (:wat::holon::filter-coincident) 3)
     _ (:wat::cache::HolographicLru::put store (:wat::holon::leaf :k1) (:wat::holon::leaf :v1))
     _ (:wat::cache::HolographicLru::put store (:wat::holon::leaf :k2) (:wat::holon::leaf :v2))
     _ (:wat::cache::HolographicLru::put store (:wat::holon::leaf :k3) (:wat::holon::leaf :v3))
     _ (:wat::cache::HolographicLru::put store (:wat::holon::leaf :k4) (:wat::holon::leaf :v4))
     n (:wat::cache::HolographicLru::len store)]
    (:wat::test::assert-eq n 3)))
