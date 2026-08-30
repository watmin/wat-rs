;; tests/collection/probe_seq_container_registry.wat — co-located fixture for the sibling probe (.rs),
;; slurped via startup_beside(file!()). Named defns for each test; startup type-checks all.

;; ── Indexable ✓ : first → element 0, across every ordered container ──

(:wat::core::defn :p::first-vector [] -> :wat::core::i64
  (:wat::core::first (:wat::core::Vector :- [:wat::core::i64] 10 20 30)))

(:wat::core::defn :p::first-persistent-vector [] -> :wat::core::i64
  (:wat::core::first (:wat::core::PersistentVector 10 20 30)))

(:wat::core::defn :p::first-list [] -> :wat::core::i64
  (:wat::core::first (:wat::core::List 10 20 30)))

(:wat::core::defn :p::first-tuple [] -> :wat::core::i64
  (:wat::core::first (:wat::core::Tuple 10 20)))

(:wat::core::defn :p::first-watast [] -> :wat::WatAST
  (:wat::core::first (:wat::core::quote (a b c))))

;; ── index variants on a Vector (second/third) ──

(:wat::core::defn :p::second-vector [] -> :wat::core::i64
  (:wat::core::second (:wat::core::Vector :- [:wat::core::i64] 10 20 30)))

(:wat::core::defn :p::third-vector [] -> :wat::core::i64
  (:wat::core::third (:wat::core::Vector :- [:wat::core::i64] 10 20 30)))

;; ── seq-1b: measurable (length/empty?) ──

(:wat::core::defn :p::tuple-length [] -> :wat::core::i64
  (:wat::core::length (:wat::core::Tuple 10 20 30)))

(:wat::core::defn :p::tuple-empty-false [] -> :wat::core::bool
  (:wat::core::empty? (:wat::core::Tuple 10 20 30)))

(:wat::core::defn :p::tuple-empty-single [] -> :wat::core::bool
  (:wat::core::empty? (:wat::core::Tuple 42)))

(:wat::core::defn :p::watastlist-length [] -> :wat::core::i64
  (:wat::core::length (:wat::core::quote (a b c))))

(:wat::core::defn :p::watastlist-empty-false [] -> :wat::core::bool
  (:wat::core::empty? (:wat::core::quote (a b c))))

;; ── seq-1b: searchable (contains?) ──

(:wat::core::defn :p::list-contains-found [] -> :wat::core::bool
  (:wat::core::contains? (:wat::core::List 10 20 30) 20))

(:wat::core::defn :p::list-contains-not-found [] -> :wat::core::bool
  (:wat::core::contains? (:wat::core::List 10 20 30) 99))

(:wat::core::defn :p::tuple-contains-found [] -> :wat::core::bool
  (:wat::core::contains? (:wat::core::Tuple 10 20 30) 20))

(:wat::core::defn :p::tuple-contains-not-found [] -> :wat::core::bool
  (:wat::core::contains? (:wat::core::Tuple 10 20 30) 99))

(:wat::core::defn :p::watastlist-contains-found [] -> :wat::core::bool
  (:wat::core::contains?
    (:wat::core::quote (a b c))
    (:wat::core::first (:wat::core::quote (a b c)))))

(:wat::core::defn :p::watastlist-contains-not-found [] -> :wat::core::bool
  (:wat::core::contains?
    (:wat::core::quote (a b c))
    (:wat::core::first (:wat::core::quote (x y z)))))

;; ── seq-1b: gettable (get → Option) ──

(:wat::core::defn :p::list-get-found [] -> (:wat::core::Option :- [:wat::core::i64])
  (:wat::core::get (:wat::core::List 10 20 30) 1))

(:wat::core::defn :p::list-get-oob [] -> (:wat::core::Option :- [:wat::core::i64])
  (:wat::core::get (:wat::core::List 10 20 30) 99))

(:wat::core::defn :p::watastlist-get-found [] -> (:wat::core::Option :- [:wat::WatAST])
  (:wat::core::get (:wat::core::quote (a b c)) 1))

(:wat::core::defn :p::watastlist-get-oob [] -> (:wat::core::Option :- [:wat::WatAST])
  (:wat::core::get (:wat::core::quote (a b c)) 99))

(:wat::core::defn :p::hashset-get-found [] -> (:wat::core::Option :- [:wat::core::i64])
  (:wat::core::get (:wat::core::HashSet :- [:wat::core::i64] 10 20 30) 20))

(:wat::core::defn :p::hashset-get-not-found [] -> (:wat::core::Option :- [:wat::core::i64])
  (:wat::core::get (:wat::core::HashSet :- [:wat::core::i64] 10 20 30) 99))
