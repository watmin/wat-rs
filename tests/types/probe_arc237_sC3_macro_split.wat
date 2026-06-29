;; tests/types/probe_arc237_sC3_macro_split.wat
;; Co-located fixture for probe_arc237_sC3_macro_split.rs
;; Loaded via startup_beside(file!()). Each named fn is exercised by its sibling Rust test.
;; liskov_base_into_holon_rejected uses a separate _bad.wat fixture.

(:wat::core::defrecord :my::Pt  [x <- :wat::core::i64  y <- :wat::core::i64])
(:wat::holon::defrecord :my::HPt [x <- :wat::core::i64  y <- :wat::core::i64])

;; Shared helpers for liskov checks
(:wat::core::defn :wb [v <- :wat::core::Record] -> :wat::core::bool true)
(:wat::core::defn :wh [v <- :wat::holon::Record] -> :wat::core::bool true)

;; ─── BASE flavor ──────────────────────────────────────────────────────────────
(:wat::core::defn :user::base-construct-and-field [] -> :wat::core::i64 (:my::Pt/x (:my::Pt 1 2)))
(:wat::core::defn :user::base-accessor [] -> :wat::core::i64 (:my::Pt/y (:my::Pt 1 2)))
(:wat::core::defn :user::base-predicate-true [] -> :wat::core::bool (:my::is-Pt? (:my::Pt 1 2)))
(:wat::core::defn :user::base-predicate-false [] -> :wat::core::bool (:my::is-Pt? (:my::HPt 1 2)))
(:wat::core::defn :user::base-eq-equal [] -> :wat::core::bool (:wat::core::= (:my::Pt 1 2) (:my::Pt 1 2)))
(:wat::core::defn :user::base-eq-diff [] -> :wat::core::bool (:wat::core::= (:my::Pt 1 2) (:my::Pt 1 9)))
(:wat::core::defn :user::base-same-data [] -> :wat::core::bool (:wat::core::Record/same-data? (:my::Pt 1 2) (:my::Pt 1 2)))
(:wat::core::defn :user::base-assoc-then-read [] -> :wat::core::i64
  (:my::Pt/y (:wat::core::Record/assoc (:my::Pt 1 2) :y 9)))
(:wat::core::defn :user::base-to-holon-errors [] -> :wat::holon::HolonAST
  (:wat::holon::to-holon (:my::Pt 1 2)))

;; ─── HOLONIC flavor ───────────────────────────────────────────────────────────
(:wat::core::defn :user::holonic-construct-field [] -> :wat::core::i64 (:my::HPt/x (:my::HPt 7 8)))
(:wat::core::defn :user::holonic-predicate-true [] -> :wat::core::bool (:my::is-HPt? (:my::HPt 7 8)))
(:wat::core::defn :user::holonic-to-holon-ok [] -> :wat::holon::HolonAST
  (:wat::holon::to-holon (:my::HPt 1 2)))

;; ─── Liskov — positive cases (type-check confirms these are valid) ─────────────
(:wat::core::defn :fb [p <- :my::Pt] -> :wat::core::bool (:wb p))
(:wat::core::defn :fh [p <- :my::HPt] -> :wat::core::bool (:wb p))
(:wat::core::defn :gh [p <- :my::HPt] -> :wat::core::bool (:wh p))

;; ─── Cross-flavor ─────────────────────────────────────────────────────────────
(:wat::core::defn :user::cross-flavor-same-data-true [] -> :wat::core::bool
  (:wat::core::Record/same-data? (:my::Pt 0 0) (:my::HPt 0 0)))
(:wat::core::defn :user::cross-flavor-eq-false [] -> :wat::core::bool
  (:wat::core::= (:my::Pt 0 0) (:my::HPt 0 0)))
