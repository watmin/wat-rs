;; tests/types/probe_arc237_stone6_is_predicate.wat
;; Co-located fixture for probe_arc237_stone6_is_predicate.rs
;; Loaded via startup_beside(file!()). Each named fn is exercised by its sibling Rust test.

(:wat::core::defrecord :my::Circle [radius <- wat.type/f64])
(:wat::core::defrecord :my::Square [side <- wat.type/f64])
(:wat::core::typeunion :my::Shape [:my::Circle :my::Square])
(:wat::core::defenum :my::Color :wat::enum::Pure :Red :Blue :Green)
(:wat::core::newtype :my::Price wat.type/f64)
(:wat::core::defstruct :my::Point [x <- wat.type/i64 y <- wat.type/i64])

;; ─── struct ───────────────────────────────────────────────────────────────────
(:wat::core::defn :user::probe01 [] -> wat.type/bool
  (:my::is-Point? (:wat::core::struct-new :my::Point 3 4)))

(:wat::core::defn :user::probe02 [] -> wat.type/bool
  (:my::is-Point? 1))

;; ─── enum ─────────────────────────────────────────────────────────────────────
(:wat::core::defn :user::probe03 [] -> wat.type/bool
  (:my::is-Color? :my::Color.Red))

(:wat::core::defn :user::probe04 [] -> wat.type/bool
  (:my::is-Color? 1))

;; ─── newtype ──────────────────────────────────────────────────────────────────
(:wat::core::defn :user::probe05 [] -> wat.type/bool
  (:my::is-Price? (:my::Price 1.5)))

(:wat::core::defn :user::probe06 [] -> wat.type/bool
  (:my::is-Price? 1.5))

;; ─── union (THE PAYLOAD) ──────────────────────────────────────────────────────
(:wat::core::defn :user::probe07 [] -> wat.type/bool
  (:my::is-Shape? (:my::Circle :radius 1.0)))

(:wat::core::defn :user::probe08 [] -> wat.type/bool
  (:my::is-Shape? (:my::Square :side 2.0)))

(:wat::core::defn :user::probe09 [] -> wat.type/bool
  (:my::is-Shape? 1))

;; ─── record (regression: exists via Record.wat) ───────────────────────────────
(:wat::core::defn :user::probe10 [] -> wat.type/bool
  (:my::is-Circle? (:my::Circle :radius 1.0)))
