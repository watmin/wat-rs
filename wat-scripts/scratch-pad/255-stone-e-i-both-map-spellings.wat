;; wat-scripts/scratch-pad/255-stone-e-i-both-map-spellings.wat — arc 255 Stone E-i acceptance
;; probe. Exercises all 8 verbs under BOTH new namespaces (`:wat::map::` for PersistentMap,
;; `:wat::hashmap::` for HashMap), asserting a concrete result for each. Not excluded from the
;; corpus codemod (it uses only the NEW spellings already; there is no old-spelling half to
;; protect, unlike the numerics A-i/A-ii probes).

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::let
    [hm0 (:wat::core::HashMap :wat::core::keyword :wat::core::i64)
     hm1 (:wat::hashmap::assoc hm0 :a 1)
     pm0 (:wat::core::PersistentMap :b 2)
     pm1 (:wat::map::assoc pm0 :a 1)]
    (:wat::core::do
      (:wat::test::assert-eq (:wat::hashmap::length hm1) 1)
      (:wat::test::assert-eq (:wat::map::length pm1) 2)
      (:wat::test::assert-eq (:wat::hashmap::empty? hm0) true)
      (:wat::test::assert-eq (:wat::map::empty? (:wat::core::PersistentMap)) true)
      (:wat::test::assert-eq (:wat::hashmap::contains-key? hm1 :a) true)
      (:wat::test::assert-eq (:wat::map::contains-key? pm1 :a) true)
      (:wat::test::assert-eq (:wat::hashmap::get hm1 :a) (:wat::core::Some 1))
      (:wat::test::assert-eq (:wat::map::get pm1 :a) (:wat::core::Some 1))
      (:wat::test::assert-eq (:wat::core::length (:wat::hashmap::keys hm1)) 1)
      (:wat::test::assert-eq (:wat::core::length (:wat::map::keys pm1)) 2)
      (:wat::test::assert-eq (:wat::core::length (:wat::hashmap::values hm1)) 1)
      (:wat::test::assert-eq (:wat::core::length (:wat::map::values pm1)) 2)
      (:wat::test::assert-eq (:wat::hashmap::empty? (:wat::hashmap::dissoc hm1 :a)) true)
      (:wat::test::assert-eq (:wat::map::empty? (:wat::map::dissoc (:wat::map::dissoc pm1 :a) :b)) true)
      (:wat::kernel::println "STONE-E-i: all 8 verbs, both new namespaces: OK")
      nil)))
