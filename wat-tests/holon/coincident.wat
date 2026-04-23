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

(:wat::config::set-capacity-mode! :error)
(:wat::config::set-dims! 1024)

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
        (:wat::core::vec :wat::holon::HolonAST
          (:wat::holon::Atom "a")
          (:wat::holon::Atom "b")
          (:wat::holon::Atom "c"))))
     ((bundle :wat::holon::HolonAST)
      (:wat::core::match bundled -> :wat::holon::HolonAST
        ((Ok h)  h)
        ((Err _) (:wat::holon::Atom "unreachable"))))
     ((atom :wat::holon::HolonAST) (:wat::holon::Atom "a")))
    ;; presence? fires (atom's signal IS in the bundle).
    (:wat::test::assert-eq
      (:wat::core::if (:wat::holon::presence? atom bundle)
                      -> :bool
        ;; And coincident? does NOT fire (the bundle is not the atom).
        (:wat::core::if (:wat::holon::coincident? atom bundle)
                        -> :bool
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
                         (:wat::holon::Thermometer 0.5 -1.0 1.0)))
     ((error :f64)
      (:wat::core::f64::- 1.0 (:wat::holon::cosine a a)))
     ((floor :f64) (:wat::config::noise-floor)))
    (:wat::test::assert-eq
      (:wat::core::< error floor)
      true)))
