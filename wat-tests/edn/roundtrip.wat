;; wat-tests/edn/roundtrip.wat — :wat::edn::read smoke tests.
;;
;; Verify EDN round-trip: write a wat value to EDN, read the EDN
;; back, assert structural equality with the original.
;;
;; The read side reconstructs structs + enums via the type registry
;; (arc 085's SymbolTable.types capability). Tag dispatch:
;;   - `#ns/Name {map}` → Struct lookup at `:ns::Name`
;;   - `#ns/Variant [body]` → Enum tagged variant
;;   - `#ns/Variant []` → Enum unit variant (arc 278 A.0; `nil` = the unit value only)

;; Test enum + struct used across the deftests below.
;; Stone 241.9 — migrated from :wat::core::enum to :wat::core::defenum (HARD CUT).
(:wat::core::defenum :test::Event :wat::enum::Pure
  :Buy  [price <- :wat::core::f64
         qty   <- :wat::core::i64]
  :Sell [price  <- :wat::core::f64
         qty    <- :wat::core::i64
         reason <- :wat::core::String])
(:wat::core::defstruct :test::Wrapper :- [E]
  [label <- :wat::core::String
   value <- :E])




;; ─── Primitives ──────────────────────────────────────────────────

(:wat::test::deftest :wat-tests::edn::roundtrip-i64
  (:wat::core::let
    [s (:wat::edn::write 42)
     back (:wat::edn::read s)]
    (:wat::test::assert-eq back 42)))

(:wat::test::deftest :wat-tests::edn::roundtrip-string
  (:wat::core::let
    [s (:wat::edn::write "hello")
     back (:wat::edn::read s)]
    (:wat::test::assert-eq back "hello")))

(:wat::test::deftest :wat-tests::edn::roundtrip-bool
  (:wat::core::let
    [s (:wat::edn::write true)
     back (:wat::edn::read s)]
    (:wat::test::assert-eq back true)))


;; ─── Vec ─────────────────────────────────────────────────────────

(:wat::test::deftest :wat-tests::edn::roundtrip-vec
  (:wat::core::let
    [v (:wat::core::Vector :wat::core::i64 1 2 3)
     s (:wat::edn::write v)
     back (:wat::edn::read s)]
    (:wat::test::assert-eq back v)))


;; ─── Enum tagged variant ─────────────────────────────────────────

(:wat::test::deftest :wat-tests::edn::roundtrip-enum-variant
  (:wat::core::let
    [e (:test::Event::Buy 100.5 7)
     s (:wat::edn::write e)
     back (:wat::edn::read s)]
    (:wat::test::assert-eq back e)))


;; ─── Struct (with named fields) ──────────────────────────────────

(:wat::test::deftest :wat-tests::edn::roundtrip-struct
  (:wat::core::let
    [w (:test::Wrapper :label "score" :value 42)
     s (:wat::edn::write w)
     back (:wat::edn::read s)]
    (:wat::test::assert-eq back w)))


;; ─── Nested: struct holding an enum ──────────────────────────────

(:wat::test::deftest :wat-tests::edn::roundtrip-nested
  (:wat::core::let
    [w
      (:test::Wrapper :label "trade" :value (:test::Event::Sell 102.25 3 "stop"))
     s (:wat::edn::write w)
     back (:wat::edn::read s)]
    (:wat::test::assert-eq back w)))
