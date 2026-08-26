;; probe-vsa-seam-rete-rows.wat — DESIGN-STONE-the-vsa-seam-opens.md, arc 278.
;;
;; Verifies the four `:wat::rete::holon::*` rows minted by this strike:
;; `cosine`/`dot` (Fallback), `coincident?` (Redispatch), `presence?` (Alias).
;;
;; ROW 2 — the seam opens: `(:wat::rete::core::f64::> (:wat::rete::holon::cosine a b
;;   :undefined 0.0) 0.9)` type-checks and runs on two similar holons -> true.
;;
;; ROW 3/4 — the fallback FIRES on a degenerate operand, and it is the CALLER'S
;;   value, not a constant. The degenerate holon is built with `:wat::holon::Blend`
;;   (the HolonAST-level constructor, NOT `vector-blend`): `zero = (Blend h h 1.0
;;   -1.0)`. `Blend`'s own encoder (holon-rs `holon_ast.rs::encode`,
;;   `HolonAST::Blend(a,b,w1,w2) => Primitives::blend_weighted(&encode(a), &encode(b),
;;   w1, w2)`) computes the IDENTICAL arithmetic `probe-zero-magnitude-reachable.wat`
;;   already proved reachable via `vector-blend` on the post-encode Vector — here it
;;   is expressed pre-encode, at the HolonAST level, because the rete row's `Holon`
;;   param (the deliberate per-type narrowing, STOP-5) rejects a raw `Vector` argument
;;   outright (that rejection is ROW 8, checked separately). `h` and `zero` are both
;;   `HolonAST`, so `zero` type-checks as a `Holon` argument to the rete row; ITS
;;   ENCODING is what is zero, discovered only when `cosine`'s core routine runs.
;;
;; Two separate degenerate calls (`degenerate-a`/`degenerate-b`) with DIFFERENT
;; `:undefined` fallbacks (-1.0 then 7.0) is ROW 4 — rows 2-3 alone pass if the arm
;; just returns a constant.
;;
;; ROW 6 — `dot` unwraps `Computed.product`, same shape.
;; ROW 7 — `presence?`/`coincident?` are 2-arity, no `:undefined` marker.
;; ROW 9 — the i64/f64 Fallback quartets are unregressed by this strike.

(:wat::core::defn :probe::run [] -> :wat::core::nil
  (:wat::core::let
    [h     (:wat::holon::to-holon "some-atom")
     other (:wat::holon::to-holon "an-entirely-different-atom")
     zero  (:wat::holon::Blend h h 1.0 -1.0)

     ;; ROW 2 — the seam: unwrapped scalar feeds :wat::rete::core::f64::> directly.
     row2-similar-above-0.9
       (:wat::rete::f64::> (:wat::rete::holon::cosine h h :undefined 0.0) 0.9)

     ;; ROW 5 — the happy payload as a bare f64 (row 2 already proves it composes
     ;; with f64::>; this captures the raw value too).
     row5-happy-scalar (:wat::rete::holon::cosine h h :undefined 0.0)

     ;; ROW 3 — degenerate operand, fallback fires (not a fabricated 0.0).
     row3-degenerate-fallback (:wat::rete::holon::cosine zero other :undefined -1.0)

     ;; ROW 4 — SAME degenerate expression, two different fallback constants.
     row4-run-a (:wat::rete::holon::cosine zero other :undefined -1.0)
     row4-run-b (:wat::rete::holon::cosine zero other :undefined 7.0)

     ;; ROW 6 — dot: happy path unwraps Computed.product; degenerate dot is still
     ;; an HONEST 0.0 by DEFINITION (dot has no Degenerate arm) — this call exists
     ;; only to prove `dot`'s Fallback arm doesn't misfire on the same zero operand
     ;; cosine treats as degenerate, taking the REAL computed 0.0, not the fallback.
     row6-dot-happy      (:wat::rete::holon::dot h h :undefined -999.0)
     row6-dot-zero-honest (:wat::rete::holon::dot zero other :undefined -999.0)

     ;; ROW 7 — predicates need no marker: 2-arity, bool.
     row7-presence   (:wat::rete::holon::presence? h h)
     row7-coincident (:wat::rete::holon::coincident? h h)

     ;; ROW 9 — i64/f64 fallback quartets unregressed.
     row9-i64-div (:wat::rete::i64::/ 1 0 :undefined -1)
     row9-f64-div (:wat::rete::f64::/ 0.0 0.0 :undefined -1.0)]

    (:wat::core::do
      (:wat::kernel::println (:wat::core::PersistentMap :row2-similar-above-0.9 row2-similar-above-0.9))
      (:wat::kernel::println (:wat::core::PersistentMap :row5-happy-scalar row5-happy-scalar))
      (:wat::kernel::println (:wat::core::PersistentMap :row3-degenerate-fallback row3-degenerate-fallback))
      (:wat::kernel::println (:wat::core::PersistentMap :row4-run-a row4-run-a))
      (:wat::kernel::println (:wat::core::PersistentMap :row4-run-b row4-run-b))
      (:wat::kernel::println (:wat::core::PersistentMap :row6-dot-happy row6-dot-happy))
      (:wat::kernel::println (:wat::core::PersistentMap :row6-dot-zero-honest row6-dot-zero-honest))
      (:wat::kernel::println (:wat::core::PersistentMap :row7-presence row7-presence))
      (:wat::kernel::println (:wat::core::PersistentMap :row7-coincident row7-coincident))
      (:wat::kernel::println (:wat::core::PersistentMap :row9-i64-div row9-i64-div))
      (:wat::kernel::println (:wat::core::PersistentMap :row9-f64-div row9-f64-div)))))

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:probe::run))
