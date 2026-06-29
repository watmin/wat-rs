;; wat-tests/edn/newtypes.wat — arc 245.3b deftest-green ward for edn.wat.
;;
;; Asserts edn.wat's two load-bearing claims IN-BAND:
;;
;;   Claim 1 — Tagged: a HolonAST written via :wat::edn::write is
;;   round-trip-safe; :wat::edn::read reconstructs the original value.
;;
;;   Claim 2 — NoTag: :wat::edn::write-notag drops #namespace/Type tags
;;   from struct/enum renders, producing a tag-free string.  A regression
;;   that routed NoTag through the tagged writer would diverge here.
;;
;;   Claims 3+4 — ctor/accessor: :wat::edn::Tagged/new wraps an inner
;;   HolonAST; :wat::edn::Tagged/0 extracts it at field index 0.  Same
;;   pair for :wat::edn::NoTag.
;;
;; Grounded strings (edn_shim.rs write / write-notag paths):
;;
;;   (:test::Event::Buy 100.5 7) via write        → "#test.Event/Buy [100.5 7]"
;;   (:test::Event::Buy 100.5 7) via write-notag  → "{:_type :test.Event/Buy :price 100.5 :qty 7}"
;;
;;   (:wat::holon::to-holon "hello") via write    → "#wat-edn.holon/String \"hello\""
;;   (:wat::holon::to-holon "hello") via write-notag → "\"hello\""
;;
;; Shared prelude: the test defenum whose renders drive claims 2+3+4.
;; Registered in the TypeEnv at freeze time so write-notag has the type
;; registry and emits named fields (:price :qty) rather than (:field-0 :field-1).

(:wat::test::make-deftest :deftest
  ((:wat::core::defenum :test::edn::nt::Event
     :Buy  [price <- :wat::core::f64
            qty   <- :wat::core::i64]
     :Sell [price  <- :wat::core::f64
            qty    <- :wat::core::i64
            reason <- :wat::core::String])))


;; ─── Claims 3+4 — Tagged ctor / accessor ─────────────────────────────────
;;
;; Construct a :wat::edn::Tagged wrapping a HolonAST string leaf.
;; Extract the inner value via Tagged/0.  Assert structural equality
;; (HolonAST implements PartialEq; assert-eq uses :wat::core::=).

(:deftest :wat-tests::edn::newtypes::tagged-ctor-accessor
  (:wat::core::let
    [ast    (:wat::holon::to-holon "hello")
     tagged (:wat::edn::Tagged ast)
     inner  (:wat::edn::Tagged/0 tagged)]
    (:wat::test::assert-eq inner ast)))


;; ─── Claims 3+4 — NoTag ctor / accessor ──────────────────────────────────
;;
;; Same shape for :wat::edn::NoTag.  The two newtypes share the same
;; runtime representation (Value::Struct arity-1); the type_name
;; discriminates them.  Verifies NoTag/new + NoTag/0 are both callable
;; and that the accessor returns the identical inner HolonAST.

(:deftest :wat-tests::edn::newtypes::notag-ctor-accessor
  (:wat::core::let
    [ast   (:wat::holon::to-holon "hello")
     notag (:wat::edn::NoTag ast)
     inner (:wat::edn::NoTag/0 notag)]
    (:wat::test::assert-eq inner ast)))


;; ─── Claim 1 — Tagged: HolonAST write + read round-trip ─────────────────
;;
;; :wat::edn::write on a HolonAST string leaf emits the tagged form
;; #wat-edn.holon/String "hello".  :wat::edn::read reconstructs the
;; original HolonAST via the wat-edn.holon tag dispatch path.
;; assert-eq uses HolonAST structural equality (PartialEq on the Rust
;; side; exposed to wat as :wat::core::=).

(:deftest :wat-tests::edn::newtypes::tagged-holon-roundtrip
  (:wat::core::let
    [ast  (:wat::holon::to-holon "hello")
     s    (:wat::edn::write ast)
     back (:wat::edn::read s)]
    (:wat::test::assert-eq back ast)))


;; ─── Claim 1 — Tagged write string matches known form ────────────────────
;;
;; Pins the exact tagged-write output so a regression that changes the
;; wat-edn.holon namespace or tag name turns this RED immediately.

(:deftest :wat-tests::edn::newtypes::tagged-write-string
  (:wat::core::let
    [ast (:wat::holon::to-holon "hello")
     s   (:wat::edn::write ast)]
    (:wat::test::assert-eq s "#wat-edn.holon/String \"hello\"")))


;; ─── Claim 2 — NoTag: write-notag produces a tag-free string ─────────────
;;
;; :wat::edn::write on :test::edn::nt::Event::Buy emits the tagged EDN
;; form #test.edn.nt.Event/Buy [100.5 7] (the #namespace/Type marker is
;; present).  :wat::edn::write-notag drops that tag and emits a flat
;; {:_type :test.edn.nt.Event/Buy :price 100.5 :qty 7} map instead.
;;
;; Both strings are asserted against their known exact forms.  A bug that
;; routes write-notag through the tagged writer would produce the first
;; string in both arms and the second assert-eq would FAIL.

(:deftest :wat-tests::edn::newtypes::notag-drops-tags
  (:wat::core::let
    [e          (:test::edn::nt::Event::Buy 100.5 7)
     tagged-s   (:wat::edn::write e)
     notag-s    (:wat::edn::write-notag e)]
    (:wat::core::do
      (:wat::test::assert-eq tagged-s "#test.edn.nt.Event/Buy [100.5 7]")
      (:wat::test::assert-eq notag-s "{:_type :test.edn.nt.Event/Buy :price 100.5 :qty 7}"))))


;; ─── Claim 2 — NoTag write-notag on HolonAST drops the wrapper tag ───────
;;
;; A HolonAST string leaf written via the tagged path emits
;; #wat-edn.holon/String "hello".  Via write-notag the leaf unwraps to
;; its bare EDN form "hello" (holon_ast_to_edn_notag String arm →
;; OwnedValue::String).  Asserts both the notag form AND its difference
;; from the tagged form.

(:deftest :wat-tests::edn::newtypes::notag-holon-drops-wrapper
  (:wat::core::let
    [ast      (:wat::holon::to-holon "hello")
     tagged-s (:wat::edn::write ast)
     notag-s  (:wat::edn::write-notag ast)]
    (:wat::core::do
      (:wat::test::assert-eq notag-s "\"hello\"")
      (:wat::test::assert-eq (:wat::core::= tagged-s notag-s) false))))
