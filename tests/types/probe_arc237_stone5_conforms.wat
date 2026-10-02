;; tests/types/probe_arc237_stone5_conforms.wat
;; Co-located fixture for probe_arc237_stone5_conforms.rs
;; Loaded via startup_beside(file!()). Each named fn is exercised by its sibling Rust test.

(:wat::core::defrecord :my::Circle [radius <- wat.type/f64])
(:wat::core::defrecord :my::Square [side <- wat.type/f64])
(:wat::core::typeunion :my::Shape [:my::Circle :my::Square])
(:wat::core::typeunion :my::Numeric [wat.type/i64 wat.type/f64])
(:wat::core::typealias :my::Bytes (wat.type/Vector :- [wat.type/u8]))

;; probe_01: record conforms its own type → true
(:wat::core::defn :user::probe01 [] -> wat.type/bool
  (:wat::core::conforms? (:my::Circle :radius 1.0) :my::Circle))

;; probe_02: record does NOT conform a different record → false
(:wat::core::defn :user::probe02 [] -> wat.type/bool
  (:wat::core::conforms? (:my::Circle :radius 1.0) :my::Square))

;; probe_03a: i64 value conforms :i64 → true
(:wat::core::defn :user::probe03a [] -> wat.type/bool
  (:wat::core::conforms? 1 wat.type/i64))

;; probe_03b: i64 value does NOT conform :f64 → false
(:wat::core::defn :user::probe03b [] -> wat.type/bool
  (:wat::core::conforms? 1 wat.type/f64))

;; probe_04a: u8 value conforms :u8 → true
(:wat::core::defn :user::probe04a [] -> wat.type/bool
  (:wat::core::conforms? (wat.type/u8 1) wat.type/u8))

;; probe_04b: u8 value does NOT conform :i64 → false (non-erasure)
(:wat::core::defn :user::probe04b [] -> wat.type/bool
  (:wat::core::conforms? (wat.type/u8 1) wat.type/i64))

;; probe_05: union member conforms the union → true
(:wat::core::defn :user::probe05 [] -> wat.type/bool
  (:wat::core::conforms? (:my::Circle :radius 1.0) :my::Shape))

;; probe_06: non-member does NOT conform the union → false
(:wat::core::defn :user::probe06 [] -> wat.type/bool
  (:wat::core::conforms? 1 :my::Shape))

;; probe_07a: i64 conforms :Numeric → true
(:wat::core::defn :user::probe07a [] -> wat.type/bool
  (:wat::core::conforms? 1 :my::Numeric))

;; probe_07b: String does NOT conform :Numeric → false
(:wat::core::defn :user::probe07b [] -> wat.type/bool
  (:wat::core::conforms? "x" :my::Numeric))

;; probe_08: all-u8 (Vector :- [u8]) conforms (Vector :- [u8]) → true
(:wat::core::defn :user::probe08 [] -> wat.type/bool
  (:wat::core::conforms?
    (wat.type/Vector :- [wat.type/u8] (wat.type/u8 1) (wat.type/u8 2) (wat.type/u8 3))
    (wat.type/Vector :- [wat.type/u8])))

;; probe_09: i64-vector does NOT conform (Vector :- [u8]) → false (element check recurses)
(:wat::core::defn :user::probe09 [] -> wat.type/bool
  (:wat::core::conforms?
    (wat.type/Vector :- [wat.type/i64] 1 2 3)
    (wat.type/Vector :- [wat.type/u8])))

;; probe_10a: u8-vector conforms :Bytes (= (Vector :- [u8])) → true (alias resolves)
(:wat::core::defn :user::probe10a [] -> wat.type/bool
  (:wat::core::conforms?
    (wat.type/Vector :- [wat.type/u8] (wat.type/u8 1) (wat.type/u8 2) (wat.type/u8 3))
    :my::Bytes))

;; probe_10b: i64-vector does NOT conform :Bytes → false
(:wat::core::defn :user::probe10b [] -> wat.type/bool
  (:wat::core::conforms?
    (wat.type/Vector :- [wat.type/i64] 1 2 3)
    :my::Bytes))

;; probe_11a: (Vector :- [Shape]) with members → true (nested union-in-element)
(:wat::core::defn :user::probe11a [] -> wat.type/bool
  (:wat::core::conforms?
    (wat.type/Vector :- [:my::Shape] (:my::Circle :radius 1.0) (:my::Square :side 2.0))
    (wat.type/Vector :- [:my::Shape])))

;; probe_11b: i64-vector does NOT conform (Vector :- [Shape]) → false
(:wat::core::defn :user::probe11b [] -> wat.type/bool
  (:wat::core::conforms?
    (wat.type/Vector :- [wat.type/i64] 1 2 3)
    (wat.type/Vector :- [:my::Shape])))

;; probe_12: unknown type name → Err at eval (not false); this fn exists for the startup check,
;; the Rust test evaluates it and asserts the result is_err().
(:wat::core::defn :user::probe12 [] -> wat.type/bool
  (:wat::core::conforms? 1 :my::DoesNotExist))
