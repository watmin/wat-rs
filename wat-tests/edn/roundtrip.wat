;; wat-tests/edn/roundtrip.wat — :wat::edn::read smoke tests.
;;
;; Verify EDN round-trip: write a wat value to EDN, read the EDN
;; back, assert structural equality with the original.
;;
;; The read side reconstructs structs + enums via the type registry
;; (arc 085's SymbolTable.types capability). Tag dispatch:
;;   - `#ns/Name {map}` → Struct lookup at `:ns::Name`
;;   - `#ns/Variant [body]` → Enum tagged variant
;;   - `#ns/Variant nil` → Enum unit variant

(:wat::test::make-deftest :deftest
  (;; Test enum + struct used across the deftests below.
   (:wat::core::enum :test::Event
     (Buy
       (price :wat::core::f64)
       (qty :wat::core::i64))
     (Sell
       (price :wat::core::f64)
       (qty :wat::core::i64)
       (reason :wat::core::String)))
   (:wat::core::struct :test::Wrapper<E>
     (label :wat::core::String)
     (value :E))))


;; ─── Primitives ──────────────────────────────────────────────────

(:deftest :wat-tests::edn::roundtrip-i64
  (:wat::core::let*
    (((s :wat::core::String) (:wat::edn::write 42))
     ((back :wat::core::i64) (:wat::edn::read s)))
    (:wat::test::assert-eq back 42)))

(:deftest :wat-tests::edn::roundtrip-string
  (:wat::core::let*
    (((s :wat::core::String) (:wat::edn::write "hello"))
     ((back :wat::core::String) (:wat::edn::read s)))
    (:wat::test::assert-eq back "hello")))

(:deftest :wat-tests::edn::roundtrip-bool
  (:wat::core::let*
    (((s :wat::core::String) (:wat::edn::write true))
     ((back :wat::core::bool) (:wat::edn::read s)))
    (:wat::test::assert-eq back true)))


;; ─── Vec ─────────────────────────────────────────────────────────

(:deftest :wat-tests::edn::roundtrip-vec
  (:wat::core::let*
    (((v :wat::core::Vector<wat::core::i64>) (:wat::core::Vector :wat::core::i64 1 2 3))
     ((s :wat::core::String) (:wat::edn::write v))
     ((back :wat::core::Vector<wat::core::i64>) (:wat::edn::read s)))
    (:wat::test::assert-eq back v)))


;; ─── Enum tagged variant ─────────────────────────────────────────

(:deftest :wat-tests::edn::roundtrip-enum-variant
  (:wat::core::let*
    (((e :test::Event) (:test::Event::Buy 100.5 7))
     ((s :wat::core::String) (:wat::edn::write e))
     ((back :test::Event) (:wat::edn::read s)))
    (:wat::test::assert-eq back e)))


;; ─── Struct (with named fields) ──────────────────────────────────

(:deftest :wat-tests::edn::roundtrip-struct
  (:wat::core::let*
    (((w :test::Wrapper<wat::core::i64>) (:test::Wrapper/new "score" 42))
     ((s :wat::core::String) (:wat::edn::write w))
     ((back :test::Wrapper<wat::core::i64>) (:wat::edn::read s)))
    (:wat::test::assert-eq back w)))


;; ─── Nested: struct holding an enum ──────────────────────────────

(:deftest :wat-tests::edn::roundtrip-nested
  (:wat::core::let*
    (((w :test::Wrapper<test::Event>)
      (:test::Wrapper/new "trade" (:test::Event::Sell 102.25 3 "stop")))
     ((s :wat::core::String) (:wat::edn::write w))
     ((back :test::Wrapper<test::Event>) (:wat::edn::read s)))
    (:wat::test::assert-eq back w)))
