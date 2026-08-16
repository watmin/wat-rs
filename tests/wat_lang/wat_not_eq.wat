;; tests/wat_lang/wat_not_eq.wat — co-located fixture.
;; Arc 056 carry-along — :wat::core::not= + Enum equality.

;; test1: not= 3 5 → true (different)
(:wat::core::defn :t::test1-not-eq-true [] -> :wat::core::bool
  (:wat::core::not= 3 5))

;; test2: not= 7 7 → false (same)
(:wat::core::defn :t::test2-not-eq-false [] -> :wat::core::bool
  (:wat::core::not= 7 7))

;; test3: cross-numeric not=/= are category-aware (arc 300 Stone C5, C5b
;; 1f1873e1). Superseded 237.8a: cross-numeric coercion for equality was
;; DELETED; C5 deliberately reversed that to match eval/clj semantics.
;; (:wat::core::not= 3 3.0) => true; (:wat::core::= 3 3.0) => false.
(:wat::core::defn :t::test3-not-eq-cross-numeric [] -> :wat::core::bool
  (:wat::core::not= 3 3.0))

(:wat::core::defn :t::test3b-eq-cross-numeric [] -> :wat::core::bool
  (:wat::core::= 3 3.0))

;; test4: enum unit variant equality via = and not=
(:wat::core::defenum :my::Color :wat::enum::Pure :Red :Blue :Green)

(:wat::core::defn :t::test4-enum-eq [] -> :wat::core::bool
  (:wat::core::let
    [a :my::Color::Red
     b :my::Color::Red
     c :my::Color::Blue]
    (:wat::core::and
      (:wat::core::= a b)
      (:wat::core::not= a c))))
