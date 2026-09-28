;; Stone 255.66 — position decides. Both spellings construct; a type slot stays a type.
(:wat::core::defn :user::lit-both [] -> :wat::core::bool
  (:wat::core::if
    (:wat::core::= [1 2 3] (wat.type/Vector :- [wat.type/i64] 1 2 3))
    (:wat::core::= [1 2 3] (:wat::core::Vector :- [:wat::core::i64] 1 2 3))
    false))

(:wat::core::defn :user::empty-both [] -> :wat::core::bool
  (:wat::core::= (wat.core/length (wat.type/Vector :- [wat.type/i64]))
                 (wat.core/length (:wat::core::Vector :- [:wat::core::i64]))))

(:wat::core::defn :user::takes [xs :- (wat.type/Vector :- [wat.type/i64])] -> :wat::core::i64
  (:wat::core::nth xs 0))

(:wat::core::defn :user::in-type [] -> :wat::core::i64
  (:user::takes (wat.type/Vector :- [wat.type/i64] 7)))

(:wat::core::defn :user::conj-nth [] -> :wat::core::i64
  (:wat::core::let [v (:wat::core::conj (wat.type/Vector :- [wat.type/i64]) 4)]
    (:wat::core::nth v 0)))

(:wat::core::defn :user::fold [] -> :wat::core::i64
  (:wat::core::foldl
    (:wat::core::fn [a :- :wat::core::i64 b :- wat.type/i64] -> :wat::core::i64
      (:wat::core::+ a b))
    0
    (wat.type/Vector :- [wat.type/i64] 1 2)))

(:wat::core::defn :user::mapv-it [] -> :wat::core::bool
  (:wat::core::= [1 2]
    (:wat::core::mapv
      (:wat::core::fn [n :- wat.type/i64] -> wat.type/i64 n)
      (wat.type/Vector :- [wat.type/i64] 1 2))))

(:wat::core::defn :user::hmap [] -> :wat::core::bool
  (:wat::core::=
    (wat.type/HashMap :- [wat.type/keyword wat.type/i64] :a 1)
    (:wat::core::HashMap :- [:wat::core::keyword :wat::core::i64] :a 1)))

(:wat::core::defn :user::hset [] -> :wat::core::bool
  (:wat::core::=
    (wat.type/HashSet :- [wat.type/i64] 1)
    (:wat::core::HashSet :- [:wat::core::i64] 1)))

(:wat::core::defn :user::pvec [] -> :wat::core::bool
  (:wat::core::=
    (wat.type/PersistentVector :- [wat.type/i64] 1 2)
    (:wat::core::PersistentVector :- [:wat::core::i64] 1 2)))

;; List does not take a type bracket (infer_linked_list_constructor). The type
;; form is the annotation; the value is the elements.
(:wat::core::defn :user::plist [] -> :wat::core::bool
  (:wat::core::=
    (wat.type/List 1 2)
    (:wat::core::List 1 2)))

(:wat::core::defn :user::takes-list [xs :- (wat.type/List :- [wat.type/i64])] -> :wat::core::i64
  1)

(:wat::core::defn :user::ptuple [] -> :wat::core::bool
  (:wat::core::=
    (wat.type/Tuple :- [wat.type/i64 wat.type/i64] 1 2)
    (:wat::core::Tuple 1 2)))

(:wat::core::defn :user::pmap [] -> :wat::core::bool
  (:wat::core::let [a (wat.type/PersistentMap :- [wat.type/keyword wat.type/i64] :a 1)
                    b (:wat::core::PersistentMap :- [:wat::core::keyword :wat::core::i64] :a 1)]
    (:wat::core::= (:wat::map::get a :a) (:wat::map::get b :a))))

(:wat::core::defmacro :user::touch-vec [] -> :wat::WatAST
  (:wat::core::let [_ (wat.type/Vector :- [wat.type/i64])]
    (:wat::core::quote 1)))

(:wat::core::defn :user::macro-ran [] -> :wat::core::i64
  (:user::touch-vec))
