;; tests/types/probe_arc237_sB2_defrecord_recordtype.wat
;; Co-located fixture for probe_arc237_sB2_defrecord_recordtype.rs (arc 237 Stone S-B.2).

(:wat::core::defrecord :my::Circle [radius <- :wat::core::f64])
(:wat::core::defrecord :my::Square [side <- :wat::core::f64])

;; ─── Probe 01: everyday is-X? ∀T ────────────────────────────────────────────
(:wat::core::defn :user::probe-01 [] -> :wat::core::bool (:my::is-Circle? 42))

;; ─── Probe 02: is-X? TRUE-path ───────────────────────────────────────────────
(:wat::core::defn :user::probe-02 [] -> :wat::core::bool (:my::is-Circle? (:my::Circle 1.0)))

;; ─── Probe 03: is-X? cross-class false ───────────────────────────────────────
(:wat::core::defn :user::probe-03 [] -> :wat::core::bool (:my::is-Circle? (:my::Square 2.0)))

;; ─── Probe 04: edge wired by emitted recordtype ──────────────────────────────
(:wat::core::defn :user::probe-04 [] -> :wat::core::bool (:wat::core::subtype? :my::Circle :wat::Record))

;; ─── Probe 05: accessors + constructor still work ─────────────────────────────
(:wat::core::defn :user::probe-05 [] -> :wat::core::f64 (:my::Circle/radius (:my::Circle 1.0)))
