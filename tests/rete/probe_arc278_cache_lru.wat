;; Co-located fixture for probe_arc278_cache_lru.rs — arc 278 Cache Stone 1 acceptance gate.
;;
;; Proves the baked `:wat::cache::Lru` primitive (src/rust_deps/cache.rs + wat/cache.wat — a
;; fresh, thread-owned bounded LRU over the `lru` crate) round-trips new/put/get/len AND that
;; the bound is real: a cap-2 cache holding {:a :b} evicts `:a` when `:c` arrives, handing the
;; evicted pair back as a NAMED `:wat::cache::Entry` (key/value), not a positional tuple.
;;
;; Keys are KEYWORDS and values are i64 — the generic `Lru :- [K V]` carries arbitrary hashable EDN,
;; it is not narrowed to String/i64.

(:wat::core::use! :rust::cache::Lru)

(:wat::test::deftest :user::cache_lru
  (:wat::core::let
    [cache (:wat::cache::Lru::new 2)
     e1    (:wat::cache::Lru::put cache :a 1)   ;; under cap  -> None
     e2    (:wat::cache::Lru::put cache :b 2)   ;; at cap     -> None
     e3    (:wat::cache::Lru::put cache :c 3)   ;; over cap   -> Some Entry{:a 1}
     got-b (:wat::cache::Lru::get cache :b)     ;; still present
     got-a (:wat::cache::Lru::get cache :a)     ;; evicted
     n     (:wat::cache::Lru::len cache)]

    (:wat::test::assert-eq e1 :wat::core::None)
    (:wat::test::assert-eq e2 :wat::core::None)
    (:wat::test::assert-eq e3 (:wat::core::Some (:wat::cache::Entry :key :a :value 1)))
    (:wat::test::assert-eq got-b (:wat::core::Some 2))
    (:wat::test::assert-eq got-a :wat::core::None)
    (:wat::test::assert-eq n 2)))

;; ─── put-overwrites-same-key — closes a Cache Stone 5 coverage gap ───────────────────────────
;;
;; The gate above only ever puts THREE DISTINCT keys (to prove eviction); nothing proved that
;; re-putting an EXISTING key updates in place. Ported from the dying `crates/wat-lru`'s
;; `test-local-cache-put-overwrites`: put "k" twice under a capacity that could never evict
;; (cap 16, one key). Per `wat/cache.wat`'s own `Lru::put` doc, the displaced-entry return
;; covers TWO cases under one `Some`: the bumped-out LRU entry on overflow, OR `k`'s PREVIOUS
;; binding when `k` was already present — so the overwrite itself correctly reports
;; `Some Entry{:key :k :value 1}` (the value being replaced), not `None`. The load-bearing
;; assertions are that the SECOND value wins on `get`, and `len` stays 1 — no duplicate slot.
(:wat::test::deftest :user::cache_lru_put_overwrites
  (:wat::core::let
    [cache (:wat::cache::Lru::new 16)
     e1    (:wat::cache::Lru::put cache :k 1)   ;; first write -> None (nothing displaced)
     e2    (:wat::cache::Lru::put cache :k 99)  ;; overwrite   -> Some Entry{:key :k :value 1}
     got   (:wat::cache::Lru::get cache :k)
     n     (:wat::cache::Lru::len cache)]

    (:wat::test::assert-eq e1 :wat::core::None)
    (:wat::test::assert-eq e2 (:wat::core::Some (:wat::cache::Entry :key :k :value 1)))
    (:wat::test::assert-eq got (:wat::core::Some 99))
    (:wat::test::assert-eq n 1)))

;; ─── HolonAST as an EXACT-match key — closes a Cache Stone 5 coverage gap ────────────────────
;;
;; The gate above only ever keys on Keyword. `:wat::cache::HolographicLru`'s internal
;; `(Lru :- [HolonAST nil])` field DOES instantiate `(Lru :- [K V])` at K=HolonAST, but only ever through
;; Hologram's own SIMILARITY match + recency bump — never a direct exact `Lru::get`/`put`
;; round-trip on a bare HolonAST key. Ported from the dying `crates/wat-lru`'s
;; `wat-tests/lru/HolonKey.wat` (arc 057 slice 3): the three properties any cache layer needs
;; to trust holon keys, proven directly against `(:wat::cache::Lru :- [K V])` this time, not the old
;; `:wat::lru::LocalCache`.
;;
;; The underlying `impl Hash + Eq for Value::holon__HolonAST` is ALSO covered at the Rust level
;; (`tests/value/probe_arc216_stone5a_value_hash.rs`) — these three gates are the WAT-level
;; proof that it flows correctly through `:rust::cache::Lru`'s `hashmap_key` dispatch.

;; round-trip — put a holon key + value, get back the value.
(:wat::test::deftest :user::cache_lru_holon_key_roundtrip
  (:wat::core::let
    [cache (:wat::cache::Lru::new 16)
     k     (:wat::holon::Atom (:wat::holon::to-holon (:wat::core::quote :the-form)))
     _put  (:wat::cache::Lru::put cache k 42)
     got   (:wat::cache::Lru::get cache k)]
    (:wat::test::assert-eq got (:wat::core::Some 42))))

;; distinguishes — structurally distinct holons land in distinct cache slots (no false
;; positives): storing under k1 and probing k2 must miss.
(:wat::test::deftest :user::cache_lru_holon_key_distinguishes
  (:wat::core::let
    [cache (:wat::cache::Lru::new 16)
     k1    (:wat::holon::Atom (:wat::holon::to-holon (:wat::core::quote :a)))
     k2    (:wat::holon::Atom (:wat::holon::to-holon (:wat::core::quote :b)))
     _put  (:wat::cache::Lru::put cache k1 1)
     got   (:wat::cache::Lru::get cache k2)]
    (:wat::test::assert-eq got :wat::core::None)))

;; structural-equal — two holons built INDEPENDENTLY but structurally equal MUST collide in the
;; cache (no false negatives — this is the load-bearing property memoization needs). If the
;; substrate's hash key included identity instead of structure, this would miss: the put under
;; k1 would never be visible under k2.
(:wat::test::deftest :user::cache_lru_holon_key_structural_equal
  (:wat::core::let
    [cache (:wat::cache::Lru::new 16)
     k1
      (:wat::holon::Bind
        (:wat::holon::Atom (:wat::holon::to-holon (:wat::core::quote :role)))
        (:wat::holon::Atom (:wat::holon::to-holon (:wat::core::quote :filler))))
     k2
      (:wat::holon::Bind
        (:wat::holon::Atom (:wat::holon::to-holon (:wat::core::quote :role)))
        (:wat::holon::Atom (:wat::holon::to-holon (:wat::core::quote :filler))))
     _put  (:wat::cache::Lru::put cache k1 99)
     got   (:wat::cache::Lru::get cache k2)]
    (:wat::test::assert-eq got (:wat::core::Some 99))))
