;; tests/collection/probe_seq_container_registry.wat — co-located fixture for the sibling probe (.rs),
;; slurped via startup_beside(file!()). Named defns for each test; startup type-checks all.

;; ── Indexable ✓ : first → element 0, across every ordered container ──

(:wat::core::defn :p::first-vector [] -> wat.type/i64
  (:wat::core::first (wat.type/Vector :- [wat.type/i64] 10 20 30)))

(:wat::core::defn :p::first-persistent-vector [] -> wat.type/i64
  (:wat::core::first (wat.type/PersistentVector :- [wat.type/i64] 10 20 30)))

(:wat::core::defn :p::first-list [] -> wat.type/i64
  (:wat::core::first (wat.type/List :- [wat.type/i64] 10 20 30)))

(:wat::core::defn :p::first-tuple [] -> wat.type/i64
  (:wat::core::first (wat.type/Tuple :- [wat.type/i64 wat.type/i64] 10 20)))

(:wat::core::defn :p::first-watast [] -> wat.type/AST
  (:wat::core::first (:wat::core::quote (a b c))))

;; ── index variants on a Vector (second/third) ──

(:wat::core::defn :p::second-vector [] -> wat.type/i64
  (:wat::core::second (wat.type/Vector :- [wat.type/i64] 10 20 30)))

(:wat::core::defn :p::third-vector [] -> wat.type/i64
  (:wat::core::third (wat.type/Vector :- [wat.type/i64] 10 20 30)))

;; ── seq-1b: measurable (length/empty?) ──

(:wat::core::defn :p::tuple-length [] -> wat.type/i64
  (:wat::core::length (wat.type/Tuple :- [wat.type/i64 wat.type/i64 wat.type/i64] 10 20 30)))

(:wat::core::defn :p::tuple-empty-false [] -> wat.type/bool
  (:wat::core::empty? (wat.type/Tuple :- [wat.type/i64 wat.type/i64 wat.type/i64] 10 20 30)))

(:wat::core::defn :p::tuple-empty-single [] -> wat.type/bool
  (:wat::core::empty? (wat.type/Tuple :- [wat.type/i64] 42)))

(:wat::core::defn :p::watastlist-length [] -> wat.type/i64
  (:wat::core::length (:wat::core::quote (a b c))))

(:wat::core::defn :p::watastlist-empty-false [] -> wat.type/bool
  (:wat::core::empty? (:wat::core::quote (a b c))))

;; ── seq-1b: searchable (contains?) ──

(:wat::core::defn :p::list-contains-found [] -> wat.type/bool
  (:wat::core::contains? (wat.type/List :- [wat.type/i64] 10 20 30) 20))

(:wat::core::defn :p::list-contains-not-found [] -> wat.type/bool
  (:wat::core::contains? (wat.type/List :- [wat.type/i64] 10 20 30) 99))

(:wat::core::defn :p::tuple-contains-found [] -> wat.type/bool
  (:wat::core::contains? (wat.type/Tuple :- [wat.type/i64 wat.type/i64 wat.type/i64] 10 20 30) 20))

(:wat::core::defn :p::tuple-contains-not-found [] -> wat.type/bool
  (:wat::core::contains? (wat.type/Tuple :- [wat.type/i64 wat.type/i64 wat.type/i64] 10 20 30) 99))

(:wat::core::defn :p::watastlist-contains-found [] -> wat.type/bool
  (:wat::core::contains?
    (:wat::core::quote (a b c))
    (:wat::core::first (:wat::core::quote (a b c)))))

(:wat::core::defn :p::watastlist-contains-not-found [] -> wat.type/bool
  (:wat::core::contains?
    (:wat::core::quote (a b c))
    (:wat::core::first (:wat::core::quote (x y z)))))

;; ── seq-1b: gettable (get → Option) ──

(:wat::core::defn :p::list-get-found [] -> (:wat::core::Option :- [wat.type/i64])
  (:wat::core::get (wat.type/List :- [wat.type/i64] 10 20 30) 1))

(:wat::core::defn :p::list-get-oob [] -> (:wat::core::Option :- [wat.type/i64])
  (:wat::core::get (wat.type/List :- [wat.type/i64] 10 20 30) 99))

(:wat::core::defn :p::watastlist-get-found [] -> (:wat::core::Option :- [wat.type/AST])
  (:wat::core::get (:wat::core::quote (a b c)) 1))

(:wat::core::defn :p::watastlist-get-oob [] -> (:wat::core::Option :- [wat.type/AST])
  (:wat::core::get (:wat::core::quote (a b c)) 99))

(:wat::core::defn :p::hashset-get-found [] -> (:wat::core::Option :- [wat.type/i64])
  (:wat::core::get (wat.type/HashSet :- [wat.type/i64] 10 20 30) 20))

(:wat::core::defn :p::hashset-get-not-found [] -> (:wat::core::Option :- [wat.type/i64])
  (:wat::core::get (wat.type/HashSet :- [wat.type/i64] 10 20 30) 99))
