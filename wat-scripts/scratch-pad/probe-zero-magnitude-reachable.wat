;; probe-zero-magnitude-reachable.wat — arc 278, the dimension-heresy stone.
;;
;; QUESTION: is a zero-magnitude Vector REACHABLE from wat?
;;
;; It decides whether `:wat::holon::cosine`'s outcome enum carries a degenerate
;; variant or not. `Similarity::cosine` guards `norm < 1e-10 -> 0.0`
;; (holon-rs/src/kernel/similarity.rs) — a sentinel that reads as "orthogonal,
;; unrelated" and sails through a `> 0.9` threshold as a confident NO-MATCH.
;; If the degenerate case is UNREACHABLE, minting a variant for it is an
;; unreachable arm accumulating lies. If it IS reachable, the sentinel is live.
;;
;; SUSPICION UNDER TEST: `vector-blend` with cancelling weights (w1=1, w2=-1 on
;; the SAME vector) should produce an all-zero Vector — i.e. the degenerate case
;; is reachable through a verb this arc just converted to `CombineOutcome`.
;;
;; NON-VACUITY CONTROL: `(cosine v v)` must come back ~1.0. Without it, a 0.0
;; from the degenerate probe could just as easily mean the probe is broken.

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
