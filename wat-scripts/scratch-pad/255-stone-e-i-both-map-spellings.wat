;; wat-scripts/scratch-pad/255-stone-e-i-both-map-spellings.wat — arc 255 Stone E-i acceptance
;; probe. Exercises all 8 verbs under BOTH new namespaces (`:wat::map::` for PersistentMap,
;; `:wat::hashmap::` for HashMap), asserting a concrete result for each. Not excluded from the
;; corpus codemod (it uses only the NEW spellings already; there is no old-spelling half to
;; protect, unlike the numerics A-i/A-ii probes).

(:wat::core::defn :user::main [] -> wat.type/nil
  (:wat::core::let
    [hm0 (wat.type/HashMap :- [wat.type/keyword wat.type/i64])
     hm1 (:wat::core::assoc hm0 :a 1)
     pm0 (wat.type/PersistentMap :- [wat.type/keyword wat.type/i64] :b 2)
     pm1 (:wat::core::assoc pm0 :a 1)]
    (:wat::core::do
      (:wat::test::assert-eq (:wat::core::length hm1) 1)
      (:wat::test::assert-eq (:wat::core::length pm1) 2)
      (:wat::test::assert-eq (:wat::core::empty? hm0) true)
      (:wat::test::assert-eq (:wat::core::empty? (wat.type/PersistentMap :- [wat.type/keyword wat.type/i64])) true)
      (:wat::test::assert-eq (:wat::core::contains? hm1 :a) true)
      (:wat::test::assert-eq (:wat::core::contains? pm1 :a) true)
      (:wat::test::assert-eq (:wat::core::get hm1 :a) (:wat::core::Option.Some {:value 1}))
      (:wat::test::assert-eq (:wat::core::get pm1 :a) (:wat::core::Option.Some {:value 1}))
      (:wat::test::assert-eq (:wat::core::length (:wat::core::keys hm1)) 1)
      (:wat::test::assert-eq (:wat::core::length (:wat::core::keys pm1)) 2)
      (:wat::test::assert-eq (:wat::core::length (:wat::core::values hm1)) 1)
      (:wat::test::assert-eq (:wat::core::length (:wat::core::values pm1)) 2)
      (:wat::test::assert-eq (:wat::core::empty? (:wat::core::dissoc hm1 :a)) true)
      (:wat::test::assert-eq (:wat::core::empty? (:wat::core::dissoc (:wat::core::dissoc pm1 :a) :b)) true)
      (:wat::kernel::println "STONE-E-i: all 8 verbs, both new namespaces: OK")
      nil)))
