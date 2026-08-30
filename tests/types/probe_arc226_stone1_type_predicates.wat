;; tests/types/probe_arc226_stone1_type_predicates.wat
;; Co-located fixture for probe_arc226_stone1_type_predicates.rs
;; Loaded via startup_beside(file!()). Each named fn is exercised by its sibling Rust test.

;; ─── Polymorphic is? ─────────────────────────────────────────────────────────
(:wat::core::defn :user::probe-is-polymorphic-positive-map [] -> :wat::core::bool
  (:wat::holon::is? (:wat::holon::to-holon {:a 1}) "Map"))

(:wat::core::defn :user::probe-is-polymorphic-negative-wrong-class [] -> :wat::core::bool
  (:wat::holon::is? (:wat::holon::to-holon {:a 1}) "Set"))

(:wat::core::defn :user::probe-is-polymorphic-positive-vector [] -> :wat::core::bool
  (:wat::holon::is? (:wat::holon::to-holon [1 2 3]) "Vector"))

;; ─── is-Map? ─────────────────────────────────────────────────────────────────
(:wat::core::defn :user::probe-is-map-positive [] -> :wat::core::bool
  (:wat::holon::is-Map? (:wat::holon::to-holon {:a 1 :b 2})))

(:wat::core::defn :user::probe-is-map-negative [] -> :wat::core::bool
  (:wat::holon::is-Map? (:wat::holon::to-holon #{1 2 3})))

;; ─── is-Set? ─────────────────────────────────────────────────────────────────
(:wat::core::defn :user::probe-is-set-positive [] -> :wat::core::bool
  (:wat::holon::is-Set? (:wat::holon::to-holon #{1 2 3})))

(:wat::core::defn :user::probe-is-set-negative [] -> :wat::core::bool
  (:wat::holon::is-Set? (:wat::holon::to-holon [1 2 3])))

;; ─── is-Vector? ──────────────────────────────────────────────────────────────
(:wat::core::defn :user::probe-is-vector-positive [] -> :wat::core::bool
  (:wat::holon::is-Vector? (:wat::holon::to-holon [1 2 3])))

(:wat::core::defn :user::probe-is-vector-negative-tuple [] -> :wat::core::bool
  (:wat::core::let
    [items (:wat::core::Vector :- [:wat::holon::HolonAST]
              (:wat::holon::to-holon 1)
              (:wat::holon::to-holon 2))
     tup   (:wat::holon::Tuple items)]
    (:wat::holon::is-Vector? tup)))

;; ─── is-List? ────────────────────────────────────────────────────────────────
(:wat::core::defn :user::probe-is-list-positive [] -> :wat::core::bool
  (:wat::core::let
    [items (:wat::core::Vector :- [:wat::holon::HolonAST]
              (:wat::holon::to-holon 1)
              (:wat::holon::to-holon 2))
     lst   (:wat::holon::List items)]
    (:wat::holon::is-List? lst)))

(:wat::core::defn :user::probe-is-list-negative [] -> :wat::core::bool
  (:wat::holon::is-List? (:wat::holon::to-holon #{1 2 3})))

;; ─── is-Tuple? ───────────────────────────────────────────────────────────────
(:wat::core::defn :user::probe-is-tuple-positive [] -> :wat::core::bool
  (:wat::core::let
    [items (:wat::core::Vector :- [:wat::holon::HolonAST]
              (:wat::holon::to-holon 1)
              (:wat::holon::to-holon 2))
     tup   (:wat::holon::Tuple items)]
    (:wat::holon::is-Tuple? tup)))

(:wat::core::defn :user::probe-is-tuple-negative-vector [] -> :wat::core::bool
  (:wat::holon::is-Tuple? (:wat::holon::to-holon [1 2 3])))

;; ─── is-Symbol? ──────────────────────────────────────────────────────────────
(:wat::core::defn :user::probe-is-symbol-positive [] -> :wat::core::bool
  (:wat::holon::is-Symbol?
    (:wat::holon::Bind
      (:wat::holon::Atom (:wat::holon::to-holon "Symbol"))
      (:wat::holon::Atom (:wat::holon::to-holon "foo")))))

(:wat::core::defn :user::probe-is-symbol-negative-keyword [] -> :wat::core::bool
  (:wat::holon::is-Symbol? (:wat::holon::to-holon :foo)))

;; ─── is-Keyword? ─────────────────────────────────────────────────────────────
(:wat::core::defn :user::probe-is-keyword-positive [] -> :wat::core::bool
  (:wat::holon::is-Keyword? (:wat::holon::to-holon :foo)))

(:wat::core::defn :user::probe-is-keyword-negative-map [] -> :wat::core::bool
  (:wat::holon::is-Keyword? (:wat::holon::to-holon {:a 1})))

;; ─── is-Tag? ─────────────────────────────────────────────────────────────────
(:wat::core::defn :user::probe-is-tag-positive [] -> :wat::core::bool
  (:wat::holon::is-Tag?
    (:wat::holon::Bind
      (:wat::holon::Atom (:wat::holon::to-holon "Tag"))
      (:wat::holon::Atom (:wat::holon::to-holon "foo")))))

(:wat::core::defn :user::probe-is-tag-negative-keyword [] -> :wat::core::bool
  (:wat::holon::is-Tag? (:wat::holon::to-holon :foo)))

;; ─── is-Nil? ─────────────────────────────────────────────────────────────────
(:wat::core::defn :user::probe-is-nil-positive [] -> :wat::core::bool
  (:wat::holon::is-Nil? (:wat::holon::to-holon nil)))

(:wat::core::defn :user::probe-is-nil-negative-non-nil-symbol [] -> :wat::core::bool
  (:wat::holon::is-Nil?
    (:wat::holon::Bind
      (:wat::holon::Atom (:wat::holon::to-holon "Symbol"))
      (:wat::holon::Atom (:wat::holon::to-holon "foo")))))

(:wat::core::defn :user::probe-is-nil-negative-map [] -> :wat::core::bool
  (:wat::holon::is-Nil? (:wat::holon::to-holon {:a 1})))

;; ─── is-Symbol? subsumes nil ─────────────────────────────────────────────────
(:wat::core::defn :user::probe-is-symbol-true-for-nil [] -> :wat::core::bool
  (:wat::holon::is-Symbol? (:wat::holon::to-holon nil)))

;; ─── Edge cases ──────────────────────────────────────────────────────────────
(:wat::core::defn :user::probe-edge-holon-i64-leaf-not-map [] -> :wat::core::bool
  (:wat::holon::is-Map? (:wat::holon::to-holon 42)))

(:wat::core::defn :user::probe-edge-holon-string-leaf-not-keyword [] -> :wat::core::bool
  (:wat::holon::is-Keyword? (:wat::holon::to-holon "hello")))

(:wat::core::defn :user::probe-edge-holon-bool-leaf-not-symbol [] -> :wat::core::bool
  (:wat::holon::is-Symbol? (:wat::holon::to-holon true)))

;; probe_edge_cross_type_set_vs_vector — two assertions, two functions
(:wat::core::defn :user::probe-edge-cross-type-set-not-vector [] -> :wat::core::bool
  (:wat::holon::is-Vector? (:wat::holon::to-holon #{1 2 3})))

(:wat::core::defn :user::probe-edge-cross-type-vector-not-set [] -> :wat::core::bool
  (:wat::holon::is-Set? (:wat::holon::to-holon [1 2 3])))
