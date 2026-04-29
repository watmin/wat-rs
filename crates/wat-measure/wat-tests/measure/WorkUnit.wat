;; wat-tests/measure/WorkUnit.wat — arc 091 slice 3 smoke tests for
;; the WorkUnit data primitives.
;;
;; Six tests cover the contract:
;;
;;   - test-uuid-non-empty       new wu has a uuid String
;;   - test-uuid-distinct        two wu's mint distinct uuids
;;   - test-counter-default      counter on absent name returns 0
;;   - test-incr-then-counter    incr! once, counter returns 1
;;   - test-incr-many            incr! 3x, counter returns 3
;;   - test-append-dt-then-read  append-dt! 2x, durations returns the Vec
;;
;; Keys are HolonAST throughout — `(:wat::holon::Atom :requests)`
;; lifts a wat keyword into the algebra. Passing a bare keyword
;; would type-check-fail since the WorkUnit/incr! signature
;; declares `name :wat::holon::HolonAST`.
;;
;; The mutation-visible-across-calls property is implicit in
;; test-incr-many — if mutation didn't persist between
;; consecutive (:incr! wu k) calls, the final counter would be 1.

;; ─── uuid is non-empty ────────────────────────────────────────────

(:wat::test::deftest :wat-measure::WorkUnit::test-uuid-non-empty
  ()
  (:wat::core::let*
    (((wu :wat::measure::WorkUnit) (:wat::measure::WorkUnit::new))
     ((id :String) (:wat::measure::WorkUnit/uuid wu)))
    ;; A canonical 8-4-4-4-12 hex uuid is 36 chars — but :String
    ;; has no length primitive in slice-3 wat surface, and the
    ;; rigorous format checks live in arc 092's Rust tests. Here
    ;; we just prove the read returns SOME string — the empty
    ;; sentinel "" would equal "" so the assertion would catch
    ;; a degenerate shim that returned the empty string.
    (:wat::test::assert-eq (:wat::core::= id "") false)))


;; ─── uuids are distinct across new() calls ───────────────────────

(:wat::test::deftest :wat-measure::WorkUnit::test-uuid-distinct
  ()
  (:wat::core::let*
    (((wu1 :wat::measure::WorkUnit) (:wat::measure::WorkUnit::new))
     ((wu2 :wat::measure::WorkUnit) (:wat::measure::WorkUnit::new))
     ((id1 :String) (:wat::measure::WorkUnit/uuid wu1))
     ((id2 :String) (:wat::measure::WorkUnit/uuid wu2)))
    (:wat::test::assert-eq (:wat::core::= id1 id2) false)))


;; ─── counter on an absent key returns 0 ──────────────────────────

(:wat::test::deftest :wat-measure::WorkUnit::test-counter-default
  ()
  (:wat::core::let*
    (((wu :wat::measure::WorkUnit) (:wat::measure::WorkUnit::new))
     ((name :wat::holon::HolonAST) (:wat::holon::Atom :never-incremented))
     ((n :i64) (:wat::measure::WorkUnit/counter wu name)))
    (:wat::test::assert-eq n 0)))


;; ─── incr! then counter — single bump ────────────────────────────

(:wat::test::deftest :wat-measure::WorkUnit::test-incr-then-counter
  ()
  (:wat::core::let*
    (((wu :wat::measure::WorkUnit) (:wat::measure::WorkUnit::new))
     ((name :wat::holon::HolonAST) (:wat::holon::Atom :requests))
     ((_ :()) (:wat::measure::WorkUnit/incr! wu name))
     ((n :i64) (:wat::measure::WorkUnit/counter wu name)))
    (:wat::test::assert-eq n 1)))


;; ─── incr! many — accumulation across calls ──────────────────────

(:wat::test::deftest :wat-measure::WorkUnit::test-incr-many
  ()
  (:wat::core::let*
    (((wu :wat::measure::WorkUnit) (:wat::measure::WorkUnit::new))
     ((name :wat::holon::HolonAST) (:wat::holon::Atom :requests))
     ((_a :()) (:wat::measure::WorkUnit/incr! wu name))
     ((_b :()) (:wat::measure::WorkUnit/incr! wu name))
     ((_c :()) (:wat::measure::WorkUnit/incr! wu name))
     ((n :i64) (:wat::measure::WorkUnit/counter wu name)))
    (:wat::test::assert-eq n 3)))


;; ─── append-dt! then read ────────────────────────────────────────

(:wat::test::deftest :wat-measure::WorkUnit::test-append-dt-then-read
  ()
  (:wat::core::let*
    (((wu :wat::measure::WorkUnit) (:wat::measure::WorkUnit::new))
     ((name :wat::holon::HolonAST) (:wat::holon::Atom :sql-page))
     ((_a :()) (:wat::measure::WorkUnit/append-dt! wu name 0.5))
     ((_b :()) (:wat::measure::WorkUnit/append-dt! wu name 1.5))
     ((dts :Vec<f64>) (:wat::measure::WorkUnit/durations wu name)))
    (:wat::test::assert-eq dts (:wat::core::vec :f64 0.5 1.5))))


;; ─── Tags — the third concern ────────────────────────────────────

;; Tag absent reads None.
(:wat::test::deftest :wat-measure::WorkUnit::test-tag-default
  ()
  (:wat::core::let*
    (((wu :wat::measure::WorkUnit) (:wat::measure::WorkUnit::new))
     ((key :wat::holon::HolonAST) (:wat::holon::Atom :asset))
     ((got :Option<wat::holon::HolonAST>)
      (:wat::measure::WorkUnit/tag wu key)))
    (:wat::test::assert-eq got :None)))


;; assoc-tag! then tag returns the value.
(:wat::test::deftest :wat-measure::WorkUnit::test-assoc-tag-then-read
  ()
  (:wat::core::let*
    (((wu  :wat::measure::WorkUnit) (:wat::measure::WorkUnit::new))
     ((key :wat::holon::HolonAST)   (:wat::holon::Atom :asset))
     ((val :wat::holon::HolonAST)   (:wat::holon::Atom :BTC))
     ((_   :())                      (:wat::measure::WorkUnit/assoc-tag! wu key val))
     ((got :Option<wat::holon::HolonAST>)
      (:wat::measure::WorkUnit/tag wu key)))
    (:wat::test::assert-eq got (Some val))))


;; assoc-tag! twice on the same key — last write wins.
(:wat::test::deftest :wat-measure::WorkUnit::test-assoc-tag-overwrites
  ()
  (:wat::core::let*
    (((wu   :wat::measure::WorkUnit) (:wat::measure::WorkUnit::new))
     ((key  :wat::holon::HolonAST)   (:wat::holon::Atom :stage))
     ((v1   :wat::holon::HolonAST)   (:wat::holon::Atom :market-eval))
     ((v2   :wat::holon::HolonAST)   (:wat::holon::Atom :ship-and-fold))
     ((_a   :())                      (:wat::measure::WorkUnit/assoc-tag! wu key v1))
     ((_b   :())                      (:wat::measure::WorkUnit/assoc-tag! wu key v2))
     ((got  :Option<wat::holon::HolonAST>)
      (:wat::measure::WorkUnit/tag wu key)))
    (:wat::test::assert-eq got (Some v2))))


;; disassoc-tag! removes the tag.
(:wat::test::deftest :wat-measure::WorkUnit::test-disassoc-tag-removes
  ()
  (:wat::core::let*
    (((wu  :wat::measure::WorkUnit) (:wat::measure::WorkUnit::new))
     ((key :wat::holon::HolonAST)   (:wat::holon::Atom :run-id))
     ((val :wat::holon::HolonAST)   (:wat::holon::Atom "abc-123"))
     ((_a  :())                      (:wat::measure::WorkUnit/assoc-tag! wu key val))
     ((_b  :())                      (:wat::measure::WorkUnit/disassoc-tag! wu key))
     ((got :Option<wat::holon::HolonAST>)
      (:wat::measure::WorkUnit/tag wu key)))
    (:wat::test::assert-eq got :None)))
