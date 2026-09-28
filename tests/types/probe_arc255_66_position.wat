;; Stone 255.66 — position decides. Both spellings construct; a type slot stays a type.
(:wat::core::defn :user::lit-both [] -> wat.type/bool
  (:wat::core::if
    (:wat::core::= [1 2 3] (wat.type/Vector :- [wat.type/i64] 1 2 3))
    (:wat::core::= [1 2 3] (wat.type/Vector :- [wat.type/i64] 1 2 3))
    false))

(:wat::core::defn :user::empty-both [] -> wat.type/bool
  (:wat::core::= (wat.core/length (wat.type/Vector :- [wat.type/i64]))
                 (wat.core/length (wat.type/Vector :- [wat.type/i64]))))

(:wat::core::defn :user::takes [xs :- (wat.type/Vector :- [wat.type/i64])] -> wat.type/i64
  (:wat::core::nth xs 0))

(:wat::core::defn :user::in-type [] -> wat.type/i64
  (:user::takes (wat.type/Vector :- [wat.type/i64] 7)))

(:wat::core::defn :user::conj-nth [] -> wat.type/i64
  (:wat::core::let [v (:wat::core::conj (wat.type/Vector :- [wat.type/i64]) 4)]
    (:wat::core::nth v 0)))

(:wat::core::defn :user::fold [] -> wat.type/i64
  (:wat::core::foldl
    (:wat::core::fn [a :- wat.type/i64 b :- wat.type/i64] -> wat.type/i64
      (:wat::core::+ a b))
    0
    (wat.type/Vector :- [wat.type/i64] 1 2)))

(:wat::core::defn :user::mapv-it [] -> wat.type/bool
  (:wat::core::= [1 2]
    (:wat::core::mapv
      (:wat::core::fn [n :- wat.type/i64] -> wat.type/i64 n)
      (wat.type/Vector :- [wat.type/i64] 1 2))))

(:wat::core::defn :user::hmap [] -> wat.type/bool
  (:wat::core::=
    (wat.type/HashMap :- [wat.type/keyword wat.type/i64] :a 1)
    (wat.type/HashMap :- [wat.type/keyword wat.type/i64] :a 1)))

(:wat::core::defn :user::hset [] -> wat.type/bool
  (:wat::core::=
    (wat.type/HashSet :- [wat.type/i64] 1)
    (wat.type/HashSet :- [wat.type/i64] 1)))

(:wat::core::defn :user::pvec [] -> wat.type/bool
  (:wat::core::=
    (wat.type/PersistentVector :- [wat.type/i64] 1 2)
    (wat.type/PersistentVector :- [wat.type/i64] 1 2)))

;; List does not take a type bracket (infer_linked_list_constructor). The type
;; form is the annotation; the value is the elements.
(:wat::core::defn :user::plist [] -> wat.type/bool
  (:wat::core::=
    (wat.type/List :- [wat.type/i64] 1 2)
    (wat.type/List :- [wat.type/i64] 1 2)))

(:wat::core::defn :user::takes-list [xs :- (wat.type/List :- [wat.type/i64])] -> wat.type/i64
  1)

(:wat::core::defn :user::ptuple [] -> wat.type/bool
  (:wat::core::=
    (wat.type/Tuple :- [wat.type/i64 wat.type/i64] 1 2)
    (wat.type/Tuple :- [wat.type/i64 wat.type/i64] 1 2)))

(:wat::core::defn :user::pmap [] -> wat.type/bool
  (:wat::core::let [a (wat.type/PersistentMap :- [wat.type/keyword wat.type/i64] :a 1)
                    b (wat.type/PersistentMap :- [wat.type/keyword wat.type/i64] :a 1)]
    (:wat::core::= (:wat::map::get a :a) (:wat::map::get b :a))))

(:wat::core::defmacro :user::touch-vec [] -> wat.type/AST
  (:wat::core::let [_ (wat.type/Vector :- [wat.type/i64])]
    (:wat::core::quote 1)))

(:wat::core::defn :user::macro-ran [] -> wat.type/i64
  (:user::touch-vec))
