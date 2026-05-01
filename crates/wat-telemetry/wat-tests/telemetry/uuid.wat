;; wat-tests/measure/uuid.wat — arc 091 slice 2 smoke tests for
;; `:wat::telemetry::uuid::v4`.
;;
;; Two checks:
;;
;;   - test-distinct-pair — two consecutive calls produce different
;;     strings. The minimum-meaningful uniqueness assertion: a
;;     constant-returning shim would fail this immediately.
;;
;;   - test-many-distinct — three calls go into a wat::core::HashSet<String>;
;;     the set must have length 3. Belt-and-suspenders against
;;     short-period RNG drift.
;;
;; The rigorous uniqueness + canonical-form proofs live in arc 092's
;; `crates/wat-edn/tests/uuid_v4_mint.rs` (256 mints, 8-4-4-4-12
;; format check). These wat-tests verify the SUBSTRATE wiring —
;; that the shim is registered, callable, and returns a `:wat::core::String`.

;; ─── Distinct pair ─────────────────────────────────────────────────

(:wat::test::deftest :wat-telemetry::uuid::test-distinct-pair
  ()
  (:wat::core::let*
    (((a :wat::core::String) (:wat::telemetry::uuid::v4))
     ((b :wat::core::String) (:wat::telemetry::uuid::v4)))
    (:wat::test::assert-eq (:wat::core::= a b) false)))


;; ─── Three distinct ────────────────────────────────────────────────

(:wat::test::deftest :wat-telemetry::uuid::test-many-distinct
  ()
  (:wat::core::let*
    (((a :wat::core::String) (:wat::telemetry::uuid::v4))
     ((b :wat::core::String) (:wat::telemetry::uuid::v4))
     ((c :wat::core::String) (:wat::telemetry::uuid::v4))
     ((s :wat::core::HashSet<wat::core::String>) (:wat::core::HashSet :wat::core::String a b c)))
    (:wat::test::assert-eq (:wat::core::length s) 3)))
