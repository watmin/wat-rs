;; tests/types/probe_arc237_stone5fix_nominal.wat
;; Co-located fixture for probe_arc237_stone5fix_nominal.rs
;; Loaded via startup_beside(file!()). Each named fn is exercised by its sibling Rust test.

(:wat::core::defenum :my::Color :wat::enum::Pure :Red :Blue :Green)
(:wat::core::defenum :my::Suit :wat::enum::Pure :Hearts :Spades)
(:wat::core::newtype :my::Price :wat::core::f64)
(:wat::core::defstruct :my::Point [x <- :wat::core::i64 y <- :wat::core::i64])
(:wat::core::defstruct :my::Pair [a <- :wat::core::i64 b <- :wat::core::i64])
(:wat::core::defrecord :my::Circle [radius <- :wat::core::f64])

;; ─── enum (the confirmed break) ──────────────────────────────────────────────
(:wat::core::defn :user::probe01 [] -> :wat::core::bool
  (:wat::core::conforms? :my::Color::Red :my::Color))

(:wat::core::defn :user::probe02 [] -> :wat::core::bool
  (:wat::core::conforms? :my::Color::Red :my::Suit))

(:wat::core::defn :user::probe03 [] -> :wat::core::bool
  (:wat::core::conforms? 1 :my::Color))

;; ─── newtype ──────────────────────────────────────────────────────────────────
(:wat::core::defn :user::probe04 [] -> :wat::core::bool
  (:wat::core::conforms? (:my::Price 1.5) :my::Price))

(:wat::core::defn :user::probe05 [] -> :wat::core::bool
  (:wat::core::conforms? (:my::Price 1.5) :wat::core::f64))

;; ─── struct ───────────────────────────────────────────────────────────────────
(:wat::core::defn :user::probe06 [] -> :wat::core::bool
  (:wat::core::conforms? (:wat::core::struct-new :my::Point 3 4) :my::Point))

(:wat::core::defn :user::probe07 [] -> :wat::core::bool
  (:wat::core::conforms? (:wat::core::struct-new :my::Point 3 4) :my::Pair))

;; ─── regression sentinels ─────────────────────────────────────────────────────
(:wat::core::defn :user::probe08 [] -> :wat::core::bool
  (:wat::core::conforms? (:my::Circle :radius 1.0) :my::Circle))

(:wat::core::defn :user::probe09 [] -> :wat::core::bool
  (:wat::core::conforms? 1 :wat::core::i64))

;; ─── the OTHER consumer: :wat::core::type ────────────────────────────────────
(:wat::core::defn :user::probe10 [] -> :wat::core::String
  (:wat::core::type :my::Color::Red))

(:wat::core::defn :user::probe11 [] -> :wat::core::String
  (:wat::core::type (:my::Price 1.5)))

(:wat::core::defn :user::probe12 [] -> :wat::core::String
  (:wat::core::type (:wat::core::struct-new :my::Point 3 4)))
