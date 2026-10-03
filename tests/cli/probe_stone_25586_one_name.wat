;; Stone 255.86 amend 4 — the one-name replacements, pinned as values.
;; Equatable results are :wat::test::assert-eq against the value written as a value.
;; PersistentMap is not a member of :wat::core::Equatable (wat/class.wat), so the three
;; map results are :probe::pm-got against :probe::pm-want, compared by Value equality
;; in the beside test. HashMap and HashSet of several entries are the collections.
(wat.core/defrecord probe/PinRec [sk :- wat.type/i64])

(wat.core/defn probe/hold [] :- wat.type/nil
  (wat.core/let
    [hm (wat.type/HashMap :- [wat.type/String wat.type/i64] "a" 1)
     hm0 (wat.type/HashMap :- [wat.type/String wat.type/i64])
     pm (wat.type/PersistentMap :- [wat.type/String wat.type/i64] "a" 1)
     pm0 (wat.type/PersistentMap :- [wat.type/String wat.type/i64])
     v (wat.type/Vector :- [wat.type/i64] 1 2)
     v0 (wat.type/Vector :- [wat.type/i64])
     v1 (wat.type/Vector :- [wat.type/i64] 1)
     pv (wat.type/PersistentVector :- [wat.type/i64] 1 2)
     pv0 (wat.type/PersistentVector :- [wat.type/i64])
     pv1 (wat.type/PersistentVector :- [wat.type/i64] 1)
     hs (wat.type/HashSet :- [wat.type/i64] 1)
     hs0 (wat.type/HashSet :- [wat.type/i64])
     ls (wat.type/List :- [wat.type/i64] 2)
     ls0 (wat.type/List :- [wat.type/i64])
     rec (probe/PinRec :sk 1)]
    (wat.core/do
      (wat.test/assert-eq (wat.core/length hm) 1)
      (wat.test/assert-eq (wat.core/empty? hm0) true)
      (wat.test/assert-eq (wat.core/empty? hm) false)
      (wat.test/assert-eq (wat.core/contains? hm "a") true)
      (wat.test/assert-eq (wat.core/contains? hm "z") false)
      (wat.test/assert-eq (wat.core/get hm "a") (wat.core/Option.Some {:value 1}))
      (wat.test/assert-eq (wat.core/get hm "z") (wat.core/Option.None {}))
      (wat.test/assert-eq
        (wat.core/assoc hm "b" 2)
        (wat.type/HashMap :- [wat.type/String wat.type/i64] "a" 1 "b" 2))
      (wat.test/assert-eq
        (wat.core/dissoc hm "a")
        (wat.type/HashMap :- [wat.type/String wat.type/i64]))
      (wat.test/assert-eq
        (wat.core/dissoc hm "z")
        (wat.type/HashMap :- [wat.type/String wat.type/i64] "a" 1))
      (wat.test/assert-eq
        (wat.core/keys hm)
        (wat.type/Vector :- [wat.type/String] "a"))
      (wat.test/assert-eq
        (wat.core/values hm)
        (wat.type/Vector :- [wat.type/i64] 1))
      (wat.test/assert-eq (wat.core/length pm) 1)
      (wat.test/assert-eq (wat.core/empty? pm0) true)
      (wat.test/assert-eq (wat.core/empty? pm) false)
      (wat.test/assert-eq (wat.core/contains? pm "a") true)
      (wat.test/assert-eq (wat.core/contains? pm "z") false)
      (wat.test/assert-eq (wat.core/get pm "a") (wat.core/Option.Some {:value 1}))
      (wat.test/assert-eq (wat.core/get pm "z") (wat.core/Option.None {}))
      (wat.test/assert-eq
        (wat.core/keys pm)
        (wat.type/Vector :- [wat.type/String] "a"))
      (wat.test/assert-eq
        (wat.core/values pm)
        (wat.type/Vector :- [wat.type/i64] 1))
      (wat.test/assert-eq (wat.core/length v) 2)
      (wat.test/assert-eq (wat.core/empty? v0) true)
      (wat.test/assert-eq (wat.core/empty? v) false)
      (wat.test/assert-eq (wat.core/contains? v 1) true)
      (wat.test/assert-eq (wat.core/contains? v 9) false)
      (wat.test/assert-eq (wat.core/get v 0) (wat.core/Option.Some {:value 1}))
      (wat.test/assert-eq
        (wat.core/conj v 3)
        (wat.type/Vector :- [wat.type/i64] 1 2 3))
      (wat.test/assert-eq
        (wat.core/concat v1 (wat.type/Vector :- [wat.type/i64] 2))
        (wat.type/Vector :- [wat.type/i64] 1 2))
      (wat.test/assert-eq
        (wat.core/into v1 (wat.type/Vector :- [wat.type/i64] 2))
        (wat.type/Vector :- [wat.type/i64] 1 2))
      (wat.test/assert-eq
        (wat.core/into v1 (wat.type/PersistentVector :- [wat.type/i64] 2))
        (wat.type/Vector :- [wat.type/i64] 1 2))
      (wat.test/assert-eq (wat.core/length pv) 2)
      (wat.test/assert-eq (wat.core/empty? pv0) true)
      (wat.test/assert-eq (wat.core/empty? pv) false)
      (wat.test/assert-eq (wat.core/contains? pv 1) true)
      (wat.test/assert-eq (wat.core/contains? pv 9) false)
      (wat.test/assert-eq (wat.core/get pv 0) (wat.core/Option.Some {:value 1}))
      (wat.test/assert-eq
        (wat.core/conj pv 3)
        (wat.type/PersistentVector :- [wat.type/i64] 1 2 3))
      (wat.test/assert-eq
        (wat.core/into pv1 (wat.type/Vector :- [wat.type/i64] 2))
        (wat.type/PersistentVector :- [wat.type/i64] 1 2))
      (wat.test/assert-eq
        (wat.core/into pv1 (wat.type/PersistentVector :- [wat.type/i64] 2))
        (wat.type/PersistentVector :- [wat.type/i64] 1 2))
      (wat.test/assert-eq (wat.core/length hs) 1)
      (wat.test/assert-eq (wat.core/empty? hs0) true)
      (wat.test/assert-eq (wat.core/empty? hs) false)
      (wat.test/assert-eq (wat.core/contains? hs 1) true)
      (wat.test/assert-eq (wat.core/contains? hs 9) false)
      (wat.test/assert-eq
        (wat.core/conj hs 2)
        (wat.type/HashSet :- [wat.type/i64] 1 2))
      (wat.test/assert-eq (wat.core/length ls) 1)
      (wat.test/assert-eq (wat.core/empty? ls0) true)
      (wat.test/assert-eq (wat.core/empty? ls) false)
      (wat.test/assert-eq (wat.core/contains? ls 2) true)
      (wat.test/assert-eq (wat.core/contains? ls 9) false)
      (wat.test/assert-eq (wat.core/get ls 0) (wat.core/Option.Some {:value 2}))
      (wat.test/assert-eq
        (wat.core/conj ls 1)
        (wat.type/List :- [wat.type/i64] 1 2))
      (wat.test/assert-eq
        (wat.type/List :- [wat.type/i64] 1 2)
        (wat.core/conj ls 1))
      (wat.test/assert-eq (wat.type/char "a") \a)
      (wat.test/assert-eq
        (wat.bytes/to-hex (wat.type/Vector :- [wat.type/u8] (wat.type/u8 255) (wat.type/u8 0) (wat.type/u8 16)))
        "ff0010")
      (wat.test/assert-eq
        (wat.bytes/from-hex "ff0010")
        (wat.core/Option.Some {:value (wat.type/Vector :- [wat.type/u8] (wat.type/u8 255) (wat.type/u8 0) (wat.type/u8 16))}))
      (wat.test/assert-eq (wat.record/field-at (probe/PinRec :sk 9) 0) 9)
      (wat.test/assert-eq
        (wat.record/same-data? (probe/PinRec :sk 9) (probe/PinRec :sk 9))
        true)
      (wat.test/assert-eq
        (wat.record/same-data? (probe/PinRec :sk 9) (probe/PinRec :sk 8))
        false)
      (wat.test/assert-eq
        (wat.core/assoc rec :sk 9)
        (probe/PinRec :sk 9)))))

(wat.core/defn user/main [] :- wat.type/nil
  (probe/hold))

(wat.core/defn probe/pm-got [] :- (wat.type/Vector :- [(wat.type/PersistentMap :- [wat.type/String wat.type/i64])])
  (wat.core/let
    [pm (wat.type/PersistentMap :- [wat.type/String wat.type/i64] "a" 1)]
    (wat.type/Vector :- [(wat.type/PersistentMap :- [wat.type/String wat.type/i64])]
      (wat.core/assoc pm "b" 2)
      (wat.core/dissoc pm "a")
      (wat.core/dissoc pm "z"))))

(wat.core/defn probe/pm-want [] :- (wat.type/Vector :- [(wat.type/PersistentMap :- [wat.type/String wat.type/i64])])
  (wat.type/Vector :- [(wat.type/PersistentMap :- [wat.type/String wat.type/i64])]
    (wat.type/PersistentMap :- [wat.type/String wat.type/i64] "a" 1 "b" 2)
    (wat.type/PersistentMap :- [wat.type/String wat.type/i64])
    (wat.type/PersistentMap :- [wat.type/String wat.type/i64] "a" 1)))
