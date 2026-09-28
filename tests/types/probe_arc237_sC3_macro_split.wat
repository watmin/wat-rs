;; tests/types/probe_arc237_sC3_macro_split.wat
;; Co-located fixture for probe_arc237_sC3_macro_split.rs
;; Loaded via startup_beside(file!()). Each named fn is exercised by its sibling Rust test.
;; liskov_base_into_holon_rejected uses a separate .wat.bad fixture.

(:wat::core::defrecord :my::Pt  [x <- wat.type/i64  y <- wat.type/i64])
(:wat::holon::defrecord :my::HPt [x <- wat.type/i64  y <- wat.type/i64])

;; Shared helpers for liskov checks
(:wat::core::defn :my::wb [v <- wat.type/Record] -> wat.type/bool true)
(:wat::core::defn :my::wh [v <- :wat::holon::Record] -> wat.type/bool true)

;; ─── BASE flavor ──────────────────────────────────────────────────────────────
(:wat::core::defn :user::base-construct-and-field [] -> wat.type/i64 (:my::Pt/x (:my::Pt :x 1 :y 2)))
(:wat::core::defn :user::base-accessor [] -> wat.type/i64 (:my::Pt/y (:my::Pt :x 1 :y 2)))
(:wat::core::defn :user::base-predicate-true [] -> wat.type/bool (:my::is-Pt? (:my::Pt :x 1 :y 2)))
(:wat::core::defn :user::base-predicate-false [] -> wat.type/bool (:my::is-Pt? (:my::HPt :x 1 :y 2)))
(:wat::core::defn :user::base-eq-equal [] -> wat.type/bool (:wat::core::= (:my::Pt :x 1 :y 2) (:my::Pt :x 1 :y 2)))
(:wat::core::defn :user::base-eq-diff [] -> wat.type/bool (:wat::core::= (:my::Pt :x 1 :y 2) (:my::Pt :x 1 :y 9)))
(:wat::core::defn :user::base-same-data [] -> wat.type/bool (:wat::core::Record/same-data? (:my::Pt :x 1 :y 2) (:my::Pt :x 1 :y 2)))
(:wat::core::defn :user::base-assoc-then-read [] -> wat.type/i64
  (:my::Pt/y (:wat::core::Record/assoc (:my::Pt :x 1 :y 2) :y 9)))
(:wat::core::defn :user::base-to-holon-errors [] -> :wat::holon::HolonAST
  (:wat::holon::to-holon (:my::Pt :x 1 :y 2)))

;; ─── HOLONIC flavor ───────────────────────────────────────────────────────────
(:wat::core::defn :user::holonic-construct-field [] -> wat.type/i64 (:my::HPt/x (:my::HPt :x 7 :y 8)))
(:wat::core::defn :user::holonic-predicate-true [] -> wat.type/bool (:my::is-HPt? (:my::HPt :x 7 :y 8)))
(:wat::core::defn :user::holonic-to-holon-ok [] -> :wat::holon::HolonAST
  (:wat::holon::to-holon (:my::HPt :x 1 :y 2)))

;; ─── Liskov — positive cases (type-check confirms these are valid) ─────────────
(:wat::core::defn :my::fb [p <- :my::Pt] -> wat.type/bool (:my::wb p))
(:wat::core::defn :my::fh [p <- :my::HPt] -> wat.type/bool (:my::wb p))
(:wat::core::defn :my::gh [p <- :my::HPt] -> wat.type/bool (:my::wh p))

;; ─── Cross-flavor ─────────────────────────────────────────────────────────────
(:wat::core::defn :user::cross-flavor-same-data-true [] -> wat.type/bool
  (:wat::core::Record/same-data? (:my::Pt :x 0 :y 0) (:my::HPt :x 0 :y 0)))

