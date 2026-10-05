;; Stone 255.95 — a symbol and a keyword are different values, and one
;; keyword pair has one spelling.
(:wat::core::defn :my::test::symbol-is-not-the-keyword [] -> wat.type/bool
  (:wat::core::if (:wat::core::= 'a.b/c :a.b/c)
    false
    true))

(:wat::core::defn :my::test::keyword-spellings-are-one-pair [] -> wat.type/bool
  (:wat::core::let
    [one (wat.type/HashSet :- [wat.type/keyword] :a::b::c :a::b/c :a.b/c)]
    (:wat::core::if (:wat::core::= :a::b::c :a::b/c)
      (:wat::core::if (:wat::core::= :a::b/c :a.b/c)
        (:wat::core::if (:wat::core::= (wat.core/count one) 1)
          (:wat::core::if (:wat::core::= 'k :k) false true)
          false)
        false)
      false)))
