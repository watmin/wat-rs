;; probe-zero-magnitude-reachable.wat — arc 278, the dimension-heresy stone,
;; now the cosine outcome wall (BRIEF-cosine-outcome-wall.md).
;;
;; ORIGINAL QUESTION (2026-08-02): is a zero-magnitude Vector REACHABLE from
;; wat? RESOLVED BY THIS PROBE, BY RUN: yes, trivially, via `vector-blend`
;; with cancelling weights. `Similarity::cosine` used to guard
;; `norm < 1e-10 -> 0.0` (holon-rs/src/kernel/similarity.rs) — a sentinel
;; that reads as "orthogonal, unrelated" and sails through a `> 0.9`
;; threshold as a confident NO-MATCH, indistinguishable from the genuine
;; unrelatedness the non-vacuity control below reads (≈ -0.0086, never
;; exactly 0.0). Reachability being proven is why `CosineOutcome::Degenerate`
;; is a real, load-bearing variant and not an unreachable arm.
;;
;; THIS PROBE NOW ALSO PROVES THE FIX: `:wat::holon::cosine` returns
;; `:wat::holon::CosineOutcome`, not a bare f64 — `println` renders it
;; structurally, so `z-vs-v` / `z-vs-z` below print `Degenerate[...]`, never
;; a bare `0.0` a caller could mistake for measured unrelatedness.
;;
;; NON-VACUITY CONTROL: `(cosine v v)` must still come back `Similarity[~1.0]`.
;; Without it, a `Degenerate` from the candidate probe could just as easily
;; mean the probe itself is broken.
;;
;; OUT OF SCOPE FOR THIS PROBE: `CosineOutcome::DimensionMismatch`'s own
;; reachability from ordinary wat is a SEPARATE, still-open question (per the
;; design stone's own "REACHABILITY IS UNPROVEN IN BOTH DIRECTIONS" — every
;; obvious route produces same-`d` Vectors by construction, `dim-count` being
;; a program-wide constant). This probe does not attempt to force it; it
;; only proves the DEGENERATE hole is faced, which was the reachable,
;; grounded hazard the wall exists to close.

(:wat::core::defn :probe::run [] -> :wat::core::nil
  (:wat::core::let
    [v (:wat::holon::encode (:wat::holon::to-holon "some-atom"))

     ;; CONTROL — a vector against itself. Must be ~1.0 or this probe proves nothing.
     self-cos (:wat::holon::cosine v v)

     ;; A SECOND, UNRELATED atom — its cancellation must land on the SAME value
     ;; as v's if both are genuinely the zero vector. Two non-zero vectors
     ;; derived from different atoms would never be `=`.
     w (:wat::holon::encode (:wat::holon::to-holon "an-entirely-different-atom"))
     v-vs-w (:wat::holon::cosine v w)

     ;; THE CANDIDATE — v*1.0 + v*(-1.0). Every i8 cell should cancel to 0.
     blended   (:wat::holon::vector-blend v v 1.0 -1.0)
     blended-w (:wat::holon::vector-blend w w 1.0 -1.0)]

    (:wat::core::match blended
      ((:wat::holon::CombineOutcome::Combined z)
        (:wat::core::match blended-w
          ((:wat::holon::CombineOutcome::Combined zw)
            (:wat::core::let
              [;; z against a real vector, and z against itself — the two shapes
               ;; a degenerate operand can take at a comparison site.
               z-vs-v (:wat::holon::cosine z v)
               z-vs-z (:wat::holon::cosine z z)
               ;; THE PROOF, not the inference: if both cancellations are the
               ;; zero vector they are bit-identical. `=` on Vector is exact
               ;; element equality (runtime.rs values_equal).
               both-zero (:wat::core::= z zw)]
              (:wat::core::do
                (:wat::kernel::println
                  (:wat::core::PersistentMap
                    :control-self-cos self-cos
                    :control-v-vs-w   v-vs-w
                    :zero-vs-real     z-vs-v
                    :zero-vs-zero     z-vs-z))
                (:wat::kernel::println
                  (:wat::core::PersistentMap :two-cancellations-identical both-zero)))))
          ((:wat::holon::CombineOutcome::DimensionMismatch e g)
            (:wat::kernel::println
              (:wat::core::PersistentMap :unexpected-w-mismatch
                (:wat::core::PersistentMap :expected e :got g))))))

      ((:wat::holon::CombineOutcome::DimensionMismatch e g)
        (:wat::kernel::println
          (:wat::core::PersistentMap :unexpected-dimension-mismatch
            (:wat::core::PersistentMap :expected e :got g)))))))

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:probe::run))
