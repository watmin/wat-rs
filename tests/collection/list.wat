;; tests/collection/list.wat — co-located fixture for the sibling probe (.rs),
;; slurped via startup_beside(file!()). Named defns for each WAT-backed test.

(:wat::core::defn :list::length-of-3 [] -> wat.type/i64
  (:wat::core::length (wat.type/List :- [wat.type/i64] 1 2 3)))

(:wat::core::defn :list::length-of-2 [] -> wat.type/i64
  (:wat::core::length (wat.type/List :- [wat.type/i64] 1 2)))

(:wat::core::defn :list::empty-q-of-empty [] -> wat.type/bool
  (:wat::core::empty? (wat.type/List :- [wat.type/i64])))

(:wat::core::defn :list::length-3 [] -> wat.type/i64
  (:wat::core::length (wat.type/List :- [wat.type/i64] 10 20 30)))

(:wat::core::defn :list::length-0 [] -> wat.type/i64
  (:wat::core::length (wat.type/List :- [wat.type/i64])))

(:wat::core::defn :list::empty-q-true [] -> wat.type/bool
  (:wat::core::empty? (wat.type/List :- [wat.type/i64])))

(:wat::core::defn :list::empty-q-false [] -> wat.type/bool
  (:wat::core::empty? (wat.type/List :- [wat.type/i64] 1)))

(:wat::core::defn :list::first-some [] -> wat.type/bool
  (:wat::core::= (:wat::core::first (wat.type/List :- [wat.type/i64] 10 20 30)) 10))

(:wat::core::defn :list::rest-tail-len [] -> wat.type/i64
  (:wat::core::length (:wat::core::rest (wat.type/List :- [wat.type/i64] 1 2 3))))

(:wat::core::defn :list::conj-prepends [] -> wat.type/bool
  (:wat::core::= (:wat::core::first (:wat::core::conj (wat.type/List :- [wat.type/i64] 2 3) 1)) 1))

(:wat::core::defn :list::vec-conj-appends [] -> wat.type/bool
  (:wat::core::= (:wat::core::first (:wat::core::conj [2 3] 1)) 2))

(:wat::core::defn :list::contains-found [] -> wat.type/bool
  (:wat::core::contains? (wat.type/List :- [wat.type/i64] 1 2 3) 2))

(:wat::core::defn :list::contains-not-found [] -> wat.type/bool
  (:wat::core::contains? (wat.type/List :- [wat.type/i64] 1 2 3) 99))

(:wat::core::defn :list::get-found [] -> wat.type/bool
  (:wat::core::match (:wat::core::get (wat.type/List :- [wat.type/i64] 10 20 30) 1)
    
    [:wat::core::Option.Some {:value x} (:wat::core::= x 20)]
    [:wat::core::Option.None {} false]))

(:wat::core::defn :list::get-oob [] -> wat.type/bool
  (:wat::core::match (:wat::core::get (wat.type/List :- [wat.type/i64] 10 20 30) 99)
    
    [:wat::core::Option.Some {:value _} false]
    [:wat::core::Option.None {} true]))
