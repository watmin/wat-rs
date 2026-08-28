;; tests/collection/probe_arc216_stone5b_hashset_native_storage.wat — co-located fixture.
;; Arc 216 Stone 216.5b — Value::wat__std__HashSet native storage refactor.

;; ─── Probe 1 — Construction with primitive elements ──────────────────────────

;; p1a: i64 set length 3
(:wat::core::defn :t::p1a-i64-set-len [] -> :wat::core::i64
  (:wat::hashset::length
    (:wat::core::HashSet :wat::core::i64 1 2 3)))

;; p1b: String set length 3
(:wat::core::defn :t::p1b-str-set-len [] -> :wat::core::i64
  (:wat::hashset::length
    (:wat::core::HashSet :wat::core::String "a" "b" "c")))

;; p1c: bool set length 2
(:wat::core::defn :t::p1c-bool-set-len [] -> :wat::core::i64
  (:wat::hashset::length
    (:wat::core::HashSet :wat::core::bool true false)))

;; p1d: keyword set length 3
(:wat::core::defn :t::p1d-kw-set-len [] -> :wat::core::i64
  (:wat::hashset::length
    (:wat::core::HashSet :wat::core::keyword :foo :bar :baz)))

;; ─── Probe 2 — contains? ────────────────────────────────────────────────────

;; p2a: i64 hit
(:wat::core::defn :t::p2a-contains-i64-hit [] -> :wat::core::bool
  (:wat::core::let
    [s (:wat::core::HashSet :wat::core::i64 10 20 30)]
    (:wat::core::contains? s 20)))

;; p2b: i64 miss
(:wat::core::defn :t::p2b-contains-i64-miss [] -> :wat::core::bool
  (:wat::core::let
    [s (:wat::core::HashSet :wat::core::i64 10 20 30)]
    (:wat::core::contains? s 99)))

;; p2c: String hit
(:wat::core::defn :t::p2c-contains-str-hit [] -> :wat::core::bool
  (:wat::core::let
    [s (:wat::core::HashSet :wat::core::String "apple" "banana")]
    (:wat::core::contains? s "apple")))

;; p2d: String miss
(:wat::core::defn :t::p2d-contains-str-miss [] -> :wat::core::bool
  (:wat::core::let
    [s (:wat::core::HashSet :wat::core::String "apple" "banana")]
    (:wat::core::contains? s "cherry")))

;; p2e: keyword hit
(:wat::core::defn :t::p2e-contains-kw-hit [] -> :wat::core::bool
  (:wat::core::let
    [s (:wat::core::HashSet :wat::core::keyword :x :y)]
    (:wat::core::contains? s :x)))

;; ─── Probe 3 — HashSet/length ────────────────────────────────────────────────

(:wat::core::defn :t::p3-length [] -> :wat::core::i64
  (:wat::hashset::length
    (:wat::core::HashSet :wat::core::i64 1 2 3 4 5)))

;; ─── Probe 4 — HashSet/empty? ────────────────────────────────────────────────

;; p4a: non-empty is false
(:wat::core::defn :t::p4a-nonempty [] -> :wat::core::bool
  (:wat::hashset::empty?
    (:wat::core::HashSet :wat::core::i64 1)))

;; p4b: deduped to one still non-empty
(:wat::core::defn :t::p4b-dedup-nonempty [] -> :wat::core::bool
  (:wat::hashset::empty?
    (:wat::core::HashSet :wat::core::i64 42 42 42)))

;; ─── Probe 5 — HashSet/conj ──────────────────────────────────────────────────

;; p5a: conj adds new element
(:wat::core::defn :t::p5a-conj-add [] -> :wat::core::bool
  (:wat::core::let
    [s0 (:wat::core::HashSet :wat::core::i64 1 2)
     s1 (:wat::core::conj s0 3)]
    (:wat::core::contains? s1 3)))

;; p5b: conj with existing element idempotent — length unchanged
(:wat::core::defn :t::p5b-conj-dup [] -> :wat::core::i64
  (:wat::core::let
    [s0 (:wat::core::HashSet :wat::core::i64 1 2)
     s1 (:wat::core::conj s0 1)]
    (:wat::hashset::length s1)))

;; p5c: conj functional — original unchanged
(:wat::core::defn :t::p5c-conj-immutable [] -> :wat::core::bool
  (:wat::core::let
    [s0 (:wat::core::HashSet :wat::core::i64 1 2)
     _  (:wat::core::conj s0 3)]
    (:wat::core::contains? s0 3)))

;; ─── Probe 6 — conj for bool elements ───────────────────────────────────────

;; p6a: conj false into set-with-true
(:wat::core::defn :t::p6a-conj-bool-false [] -> :wat::core::bool
  (:wat::core::let
    [s0 (:wat::core::HashSet :wat::core::bool true)
     s1 (:wat::core::conj s0 false)]
    (:wat::core::contains? s1 false)))

;; p6b: conj of already-present bool element — length stays 2
(:wat::core::defn :t::p6b-conj-bool-dedup [] -> :wat::core::i64
  (:wat::core::let
    [s0 (:wat::core::HashSet :wat::core::bool true false)
     s1 (:wat::core::conj s0 true)]
    (:wat::hashset::length s1)))

;; ─── Probe 7 — Nested HashSet ────────────────────────────────────────────────

;; p7a: outer length 2 (two distinct inner sets)
(:wat::core::defn :t::p7a-nested-len [] -> :wat::core::i64
  (:wat::core::let
    [inner1 (:wat::core::HashSet :wat::core::i64 1 2)
     inner2 (:wat::core::HashSet :wat::core::i64 3 4)
     outer  (:wat::core::HashSet :wat::type::Infer inner1 inner2)]
    (:wat::hashset::length outer)))

;; p7b: inner HashSet found by value equality
(:wat::core::defn :t::p7b-nested-contains [] -> :wat::core::bool
  (:wat::core::let
    [inner1 (:wat::core::HashSet :wat::core::i64 1 2)
     inner2 (:wat::core::HashSet :wat::core::i64 3 4)
     outer  (:wat::core::HashSet :wat::type::Infer inner1 inner2)
     probe  (:wat::core::HashSet :wat::core::i64 1 2)]
    (:wat::core::contains? outer probe)))

;; p7c: duplicate inner HashSet deduped
(:wat::core::defn :t::p7c-nested-dedup [] -> :wat::core::i64
  (:wat::core::let
    [inner (:wat::core::HashSet :wat::core::i64 1 2)
     outer (:wat::core::HashSet :wat::type::Infer inner inner)]
    (:wat::hashset::length outer)))

;; ─── Probe 8 — Round-trip to-holon + from-holon ─────────────────────────────

;; p8a: round-trip preserves length 3
(:wat::core::defn :t::p8a-rt-len [] -> :wat::core::i64
  (:wat::core::let
    [s    (:wat::core::HashSet :wat::core::i64 10 20 30)
     atom (:wat::holon::to-holon s)
     back (:wat::holon::from-holon atom)]
    (:wat::hashset::length back)))

;; p8b: round-trip preserves membership
(:wat::core::defn :t::p8b-rt-contains [] -> :wat::core::bool
  (:wat::core::let
    [s    (:wat::core::HashSet :wat::core::i64 10 20 30)
     atom (:wat::holon::to-holon s)
     back (:wat::holon::from-holon atom)]
    (:wat::core::contains? back 20)))

;; ─── Probe 9 — HashSet as VALUE inside a HashMap ─────────────────────────────

(:wat::core::defn :t::p9-hashset-as-hm-val [] -> :wat::core::bool
  (:wat::core::let
    [inner   (:wat::core::HashSet :wat::core::i64 1 2 3)
     m       (:wat::core::HashMap :wat::core::keyword :wat::type::Infer :my-set inner)
     fetched (:wat::core::match (:wat::core::get m :my-set) 
                ((:wat::core::Some v) (:wat::core::contains? v 2))
                (:wat::core::None     false))]
    fetched))

;; ─── Probe 10 — HashSet as KEY inside a HashMap ──────────────────────────────

(:wat::core::defn :t::p10-hashset-as-hm-key [] -> :wat::core::bool
  (:wat::core::let
    [key   (:wat::core::HashSet :wat::core::i64 7 8 9)
     m     (:wat::core::HashMap :wat::type::Infer :wat::core::String key "found-it")
     probe (:wat::core::HashSet :wat::core::i64 7 8 9)]
    (:wat::hashmap::contains-key? m probe)))
