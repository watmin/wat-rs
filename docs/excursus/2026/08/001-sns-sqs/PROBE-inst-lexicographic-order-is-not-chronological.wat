;; PROBE-inst-lexicographic-order-is-not-chronological.wat — excursus 001 stone INST.
;;
;; RED AT HEAD. GREEN AFTER THE STONE, with no edit to this file.
;;
;; THE PROPERTY, which nothing in the tree currently asserts:
;;
;;     for any two instants a and b,  a < b (in TIME)  ⟺  (edn/write a) < (edn/write b) (as STRINGS)
;;
;; Every range `scan` over a timestamp sort key depends on it. `:wat::query::Store/scan` orders by
;; the `sk` string; `:wat::telemetry::journal` puts a timestamp there. If the property does not
;; hold, a range scan silently drops rows — and it does not hold today.
;;
;; THE CAUSE: crates/wat-edn/src/writer.rs:227 renders with `SecondsFormat::AutoSi` — chrono's
;; "shortest representation that is a multiple of 3 digits". So 1.200000000s prints ".200Z" while
;; 1.200000100s prints ".200000100Z", and 'Z' (0x5A) sorts AFTER '0' (0x30) — the EARLIER instant
;; compares GREATER. `SecondsFormat::Nanos` always emits 9 digits and the property holds.
;;
;; The pairs below sit on AutoSi's 0/3/6/9-digit boundaries, which is where it switches width.
;; Each pair is (earlier, later) by construction: the second nanos value is strictly larger.
;;
;; NOTE this would be a natural `:wat::gen::` property once wat-gen reaches this branch (it lives
;; on grok-rete today). A hand-picked table is the honest instrument until then — and it is chosen
;; from the RULE (where does the renderer change width?), not from the one failure that was found.

(:wat::core::defn :probe::ordered?
  [earlier-ns <- :wat::core::i64  later-ns <- :wat::core::i64] -> :wat::core::bool
  (:wat::core::<
    (:wat::edn::write (:wat::time::at-nanos earlier-ns))
    (:wat::edn::write (:wat::time::at-nanos later-ns))))

;; ── the boundary table — each row crosses one of AutoSi's width switches ─────────
(:wat::test::deftest :user::lexicographic-order-is-chronological-at-the-9-digit-boundary
  ;; 1.200000000 renders ".200Z" (3 digits); 1.200000100 renders ".200000100Z" (9).
  (:wat::test::assert-true (:probe::ordered? 1200000000 1200000100)))

(:wat::test::deftest :user::lexicographic-order-is-chronological-at-the-whole-second-boundary
  ;; 1.000000000 renders ".000Z" or shorter; 1.000000001 needs all 9.
  (:wat::test::assert-true (:probe::ordered? 1000000000 1000000001)))

(:wat::test::deftest :user::lexicographic-order-is-chronological-at-the-6-digit-boundary
  ;; 1.123456000 renders ".123456Z" (6 digits); 1.123456001 renders 9.
  (:wat::test::assert-true (:probe::ordered? 1123456000 1123456001)))

(:wat::test::deftest :user::lexicographic-order-is-chronological-at-the-3-digit-boundary
  ;; 1.100000000 renders ".100Z"; 1.100000001 renders 9.
  (:wat::test::assert-true (:probe::ordered? 1100000000 1100000001)))

;; ── the control: two instants that ALREADY render at the same width must still order ──
;; If this row ever goes red, the probe itself is broken, not the renderer.
(:wat::test::deftest :user::same-width-instants-order-correctly
  (:wat::test::assert-true (:probe::ordered? 1123456789 1123456790)))

;; ── the width claim, stated directly ────────────────────────────────────────────
;; A constant-width rendering is the MECHANISM behind the property above. Asserting it
;; separately means a future change that keeps the four rows green by accident (e.g. by
;; special-casing a comparison) still goes red here.
(:wat::test::deftest :user::every-instant-renders-at-the-same-width
  (:wat::core::let
    [a (:wat::edn::write (:wat::time::at-nanos 1200000000))   ;; trailing zeros
     b (:wat::edn::write (:wat::time::at-nanos 1200000100))   ;; needs 9
     c (:wat::edn::write (:wat::time::at-nanos 1000000000))]  ;; whole second
    (:wat::core::let
      [_1 (:wat::test::assert-eq (:wat::string::length a) (:wat::string::length b))]
      (:wat::test::assert-eq (:wat::string::length b) (:wat::string::length c)))))
