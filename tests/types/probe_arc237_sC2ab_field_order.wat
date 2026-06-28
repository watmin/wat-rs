;; tests/types/probe_arc237_sC2ab_field_order.wat
;; Co-located fixture for probe_arc237_sC2ab_field_order.rs
;; Loaded via startup_beside(file!()). Each named fn is exercised by its sibling Rust test.

(:wat::core::defrecord :my::Pt [x <- :wat::core::f64  y <- :wat::core::f64])

;; first_field_by_keyword
(:wat::core::defn :user::first-field-by-keyword [] -> :wat::core::f64 (:x (:my::Pt 1.0 2.0)))

;; second_field_by_keyword — the name-ORDER guard
(:wat::core::defn :user::second-field-by-keyword [] -> :wat::core::f64 (:y (:my::Pt 1.0 2.0)))

;; second_field_by_accessor
(:wat::core::defn :user::second-field-by-accessor [] -> :wat::core::f64 (:my::Pt/y (:my::Pt 1.0 2.0)))

;; assoc_second_field_by_name
(:wat::core::defn :user::assoc-second-field-by-name [] -> :wat::core::f64
  (:y (:wat::Record/assoc (:my::Pt 1.0 2.0) :y 9.0)))

;; assoc_second_leaves_first
(:wat::core::defn :user::assoc-second-leaves-first [] -> :wat::core::f64
  (:x (:wat::Record/assoc (:my::Pt 1.0 2.0) :y 9.0)))
