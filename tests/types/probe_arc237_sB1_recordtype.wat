;; tests/types/probe_arc237_sB1_recordtype.wat
;; Co-located fixture for probe_arc237_sB1_recordtype.rs (arc 237 Stone S-B.1).
;; Probes 01-05 use this shared world. Probe 06 (negative) uses _bad.wat.

(:wat::core::recordtype :my::Circle :wat::core::Record [])
(:wat::core::recordtype :my::Sphere :wat::holon::Record [])

;; ─── Probe 01: recordtype registers (trivial true) ───────────────────────────
(:wat::core::defn :user::probe-01 [] -> :wat::core::bool (:wat::core::= 1 1))

;; ─── Probe 02: is-X? synthesized ∀T — false on non-record ───────────────────
(:wat::core::defn :user::probe-02 [] -> :wat::core::bool (:my::is-Circle? 42))

;; ─── Probe 03: edge wired (Circle is-a Record) ───────────────────────────────
(:wat::core::defn :user::probe-03 [] -> :wat::core::bool (:wat::core::subtype? :my::Circle :wat::core::Record))

;; ─── Probe 04: directional (Record is NOT-a Circle) ──────────────────────────
(:wat::core::defn :user::probe-04 [] -> :wat::core::bool (:wat::core::subtype? :wat::core::Record :my::Circle))

;; ─── Probe 05: holon-flavor parent + transitive ──────────────────────────────
(:wat::core::defn :user::probe-05a [] -> :wat::core::bool (:wat::core::subtype? :my::Sphere :wat::core::Record))
(:wat::core::defn :user::probe-05b [] -> :wat::core::bool (:wat::core::subtype? :my::Sphere :wat::holon::Record))
