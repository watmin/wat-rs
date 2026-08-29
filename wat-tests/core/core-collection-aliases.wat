;; wat-tests/core/core-collection-aliases.wat — corpus witnesses for the
;; short-name collection alias surface defined in wat/core.wat lines 16-19:
;;
;;   (:wat::core::defalias :wat::core::dissoc  :wat::core::HashMap/dissoc)
;;   (:wat::core::defalias :wat::core::keys    :wat::core::HashMap/keys)
;;   (:wat::core::defalias :wat::core::values  :wat::core::HashMap/values)
;;   (:wat::core::defalias :wat::core::concat  :wat::core::Vector/concat)
;;
;; These witnesses exist so the short-name alias surface is EXERCISED by
;; the corpus, not just defined.  register_defalias installs a silent
;; nil-stub when the target is missing and defers the error to call-time,
;; so a broken alias target can go undetected while the suite stays green.
;; Calling each alias through a passing test closes that gap.
;;
;; Grounded on:
;;   - alias definitions:  wat/core.wat lines 16-19
;;   - type signatures:    src/check.rs lines 16797-16852
;;   - deftest idiom:      wat-tests/core/core-arithmetic.wat
;;   - HashMap construction: (:wat::core::HashMap :K :V) = empty map;
;;                           (:wat::core::assoc m k v) = map with k→v added
;;   - probare caller-absence finding, arc 249 ward-close

;; ─── dissoc: remove a key, length drops by one ────────────────────────────
;;
;; Build a 2-key map {a→1, b→2}, dissoc "a", assert keys length = 1.
;; Uses the SHORT name :wat::core::dissoc (alias for HashMap/dissoc).

(:wat::test::deftest :wat-tests::core::core-collection-aliases::dissoc-removes-key
  
  (:wat::core::let
    [m0 (:wat::core::HashMap :- [:wat::core::String :wat::core::i64])
     m1 (:wat::core::assoc m0 "a" 1)
     m2 (:wat::core::assoc m1 "b" 2)
     ;; short-name alias under test
     m3 (:wat::core::dissoc m2 "a")
     ks (:wat::hashmap::keys m3)]
    (:wat::test::assert-eq (:wat::core::length ks) 1)))

;; ─── dissoc: result is a HashMap, not a tombstoned original ──────────────
;;
;; dissoc the only key from a 1-key map → empty map (length 0).

(:wat::test::deftest :wat-tests::core::core-collection-aliases::dissoc-to-empty
  
  (:wat::core::let
    [m0 (:wat::core::HashMap :- [:wat::core::String :wat::core::i64])
     m1 (:wat::core::assoc m0 "only" 42)
     m2 (:wat::core::dissoc m1 "only")
     ks (:wat::hashmap::keys m2)]
    (:wat::test::assert-eq (:wat::core::length ks) 0)))

;; ─── keys: returns a vector of all keys ───────────────────────────────────
;;
;; Build a 1-key map {x→99}, call short-name :wat::core::keys, assert length 1.
;; Does NOT assert the key value (HashMap order is non-deterministic for
;; multi-key maps; a 1-key map is unambiguous).

(:wat::test::deftest :wat-tests::core::core-collection-aliases::keys-single-entry
  
  (:wat::core::let
    [m0 (:wat::core::HashMap :- [:wat::core::String :wat::core::i64])
     m1 (:wat::core::assoc m0 "x" 99)
     ;; short-name alias under test
     ks (:wat::core::keys m1)]
    (:wat::test::assert-eq (:wat::core::length ks) 1)))

;; ─── keys: two-key map has two keys ──────────────────────────────────────

(:wat::test::deftest :wat-tests::core::core-collection-aliases::keys-two-entries
  
  (:wat::core::let
    [m0 (:wat::core::HashMap :- [:wat::core::String :wat::core::i64])
     m1 (:wat::core::assoc m0 "p" 10)
     m2 (:wat::core::assoc m1 "q" 20)
     ks (:wat::core::keys m2)]
    (:wat::test::assert-eq (:wat::core::length ks) 2)))

;; ─── values: returns a vector of all values ───────────────────────────────
;;
;; Build a 1-key map {k→7}, call short-name :wat::core::values, assert
;; the result has length 1 (value content would need sort for >1-key maps
;; due to non-determinism; 1-key is deterministic).

(:wat::test::deftest :wat-tests::core::core-collection-aliases::values-single-entry
  
  (:wat::core::let
    [m0 (:wat::core::HashMap :- [:wat::core::String :wat::core::i64])
     m1 (:wat::core::assoc m0 "k" 7)
     ;; short-name alias under test
     vs (:wat::core::values m1)]
    (:wat::test::assert-eq (:wat::core::length vs) 1)))

;; ─── values: content check on 1-key map (deterministic) ─────────────────
;;
;; A 1-key map has exactly one value; assert its contents equal [7].
;; Uses the = intrinsic on (Vec :- [i64]) (arc 237.8d equality grid — Vecs
;; compare element-wise).

(:wat::test::deftest :wat-tests::core::core-collection-aliases::values-content-one-key
  
  (:wat::core::let
    [m0 (:wat::core::HashMap :- [:wat::core::String :wat::core::i64])
     m1 (:wat::core::assoc m0 "k" 7)
     vs (:wat::core::values m1)
     expected (:wat::core::Vector :- [:wat::core::i64] 7)]
    (:wat::test::assert-eq vs expected)))

;; ─── concat: concatenates two vectors ────────────────────────────────────
;;
;; [1 2] ++ [3 4] = [1 2 3 4] via short-name :wat::core::concat.
;; Alias for Vector/concat; 2-arg, same-type (i64) vectors.

(:wat::test::deftest :wat-tests::core::core-collection-aliases::concat-two-vectors
  
  (:wat::core::let
    [left     (:wat::core::Vector :- [:wat::core::i64] 1 2)
     right    (:wat::core::Vector :- [:wat::core::i64] 3 4)
     ;; short-name alias under test
     combined (:wat::core::concat left right)
     expected (:wat::core::Vector :- [:wat::core::i64] 1 2 3 4)]
    (:wat::test::assert-eq combined expected)))

;; ─── concat: empty ++ non-empty = non-empty ───────────────────────────────

(:wat::test::deftest :wat-tests::core::core-collection-aliases::concat-empty-left
  
  (:wat::core::let
    [empty    (:wat::core::Vector :- [:wat::core::i64])
     right    (:wat::core::Vector :- [:wat::core::i64] 5 6 7)
     combined (:wat::core::concat empty right)
     expected (:wat::core::Vector :- [:wat::core::i64] 5 6 7)]
    (:wat::test::assert-eq combined expected)))

;; ─── concat: length of concatenated result ────────────────────────────────

(:wat::test::deftest :wat-tests::core::core-collection-aliases::concat-length
  
  (:wat::core::let
    [left     (:wat::core::Vector :- [:wat::core::i64] 1 2 3)
     right    (:wat::core::Vector :- [:wat::core::i64] 4 5)
     combined (:wat::core::concat left right)]
    (:wat::test::assert-eq (:wat::core::length combined) 5)))
