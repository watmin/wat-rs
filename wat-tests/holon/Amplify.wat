;; wat-tests/holon/Amplify.wat — tests for wat/holon/Amplify.wat.
;;
;; :wat::holon::Amplify (058-015) expands to (Blend x y 1.0 s): anchor
;; x at unit emphasis, scale y's contribution by s. Two claims:
;;
;; 1. Expansion equivalence — (Amplify x y s) coincides with the
;;    explicit (Blend x y 1.0 s). Proves the macro is pure sugar.
;; 2. Scale distinguishability — (Amplify x y 2.0) does NOT coincide
;;    with (Amplify x y 1.0): different s values produce different
;;    encodings, proving the scale parameter is load-bearing.


;; ─── 1. expansion equivalence ──────────────────────────────────────

(:wat::test::deftest' :wat-tests::holon::Amplify::test-amplify-is-blend-sugar
  
  (:wat::core::let
    [x (:wat::holon::to-holon "anchor")
     y (:wat::holon::to-holon "signal")
     s 2.5
     sugar    (:wat::holon::Amplify x y s)
     explicit (:wat::holon::Blend   x y 1.0 s)]
    (:wat::test::assert-eq
      (:wat::holon::coincident? sugar explicit)
      true)))

;; ─── 2. scale distinguishability ───────────────────────────────────

(:wat::test::deftest' :wat-tests::holon::Amplify::test-amplify-scale-differs
  
  (:wat::core::let
    [x    (:wat::holon::to-holon "anchor")
     y    (:wat::holon::to-holon "signal")
     ;; s=1.0: x and y equally weighted. s=2.0: y doubly weighted.
     ;; Different blend vectors → not coincident.
     unit (:wat::holon::Amplify x y 1.0)
     loud (:wat::holon::Amplify x y 2.0)]
    (:wat::test::assert-eq
      (:wat::holon::coincident? unit loud)
      false)))
