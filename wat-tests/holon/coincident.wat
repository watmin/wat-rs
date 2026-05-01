;; wat-tests/holon/coincident.wat — tests for :wat::holon::coincident?
;; (arc 023).
;;
;; coincident? is the VSA-native equivalence predicate. Dual to
;; presence? using the same noise-floor:
;;
;;   presence?   a b = cosine(a, b)       > noise-floor   ; signal detected
;;   coincident? a b = (1 - cosine(a, b)) < noise-floor   ; same holon
;;
;; Same threshold, two directions, one substrate.


;; ─── Self-coincidence: a holon is the same as itself ─────────────

(:wat::test::deftest :wat-tests::holon::coincident::test-self-coincident
  ()
  (:wat::core::let*
    (((a :wat::holon::HolonAST) (:wat::holon::Atom "rsi")))
    (:wat::test::assert-eq
      (:wat::holon::coincident? a a)
      true)))

;; ─── Structural equivalence: same-shape ASTs coincide ────────────

(:wat::test::deftest :wat-tests::holon::coincident::test-structurally-same
  ()
  (:wat::core::let*
    (((a :wat::holon::HolonAST)
      (:wat::holon::Bind (:wat::holon::Atom "k") (:wat::holon::Atom "v")))
     ((b :wat::holon::HolonAST)
      (:wat::holon::Bind (:wat::holon::Atom "k") (:wat::holon::Atom "v"))))
    (:wat::test::assert-eq
      (:wat::holon::coincident? a b)
      true)))

;; ─── Unrelated holons do NOT coincide ────────────────────────────

(:wat::test::deftest :wat-tests::holon::coincident::test-unrelated-not-coincident
  ()
  (:wat::core::let*
    (((a :wat::holon::HolonAST) (:wat::holon::Atom "alice"))
     ((b :wat::holon::HolonAST) (:wat::holon::Atom "charlie")))
    (:wat::test::assert-eq
      (:wat::holon::coincident? a b)
      false)))

;; ─── Coincident? is STRICTER than presence? ──────────────────────
;;
;; An Atom is present in a Bundle that contains it, but is NOT
;; coincident with the Bundle — the Bundle is a superposition, not
;; the atom itself.
(:wat::test::deftest :wat-tests::holon::coincident::test-stricter-than-presence
  ()
  (:wat::core::let*
    (((bundled :wat::holon::BundleResult)
      (:wat::holon::Bundle
        (:wat::core::Vector :wat::holon::HolonAST
          (:wat::holon::Atom "a")
          (:wat::holon::Atom "b")
          (:wat::holon::Atom "c"))))
     ((bundle :wat::holon::HolonAST)
      (:wat::core::match bundled -> :wat::holon::HolonAST
        ((:wat::core::Ok h)  h)
        ((:wat::core::Err _) (:wat::holon::Atom "unreachable"))))
     ((atom :wat::holon::HolonAST) (:wat::holon::Atom "a")))
    ;; presence? fires (atom's signal IS in the bundle).
    (:wat::test::assert-eq
      (:wat::core::if (:wat::holon::presence? atom bundle)
                      -> :wat::core::bool
        ;; And coincident? does NOT fire (the bundle is not the atom).
        (:wat::core::if (:wat::holon::coincident? atom bundle)
                        -> :wat::core::bool
          false    ;; would mean they coincide — wrong
          true)
        false)     ;; presence? false means test setup is wrong
      true)))

;; ─── Self-cosine jitter stays well under noise-floor ─────────────
;;
;; Locks the numerical invariant the predicate depends on:
;; encoded-vector float precision at d=1024 jitters by ~1e-10,
;; which is 15 orders of magnitude below noise-floor (~0.156).
;; Coincident? has massive headroom for self-equivalence checks.
(:wat::test::deftest :wat-tests::holon::coincident::test-self-cosine-within-floor
  ()
  (:wat::core::let*
    (((a :wat::holon::HolonAST)
      (:wat::holon::Bind (:wat::holon::Atom "rsi")
                         (:wat::holon::Thermometer 0.5 -1.0 1.0))))
    ;; Arc 037: coincident? does the per-d threshold comparison
    ;; internally. Replaces the pre-arc-037 hand-rolled
    ;; `(cosine a a) vs (noise-floor)` — the accessor is retired
    ;; since noise-floor is per-d now, not a global config value.
    (:wat::test::assert-eq (:wat::holon::coincident? a a) true)))
