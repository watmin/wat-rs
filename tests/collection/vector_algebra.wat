;; tests/collection/vector_algebra.wat — co-located fixture for the sibling probe (.rs),
;; slurped via startup_beside(file!()). Functions RETURN String results for eval_in_frozen.

;; Arc 278 the dimension-heresy strike — vector-bind/bundle/blend now
;; return :wat::holon::CombineOutcome instead of a bare Vector (was a
;; TypeMismatch raise on differing dims). va/vb are always encoded at the
;; same ambient d here, so DimensionMismatch is unreachable in these
;; fixtures, but the match must still face it exhaustively (no `_` arm).

(:wat::core::defn :valg::bind-roundtrip [] -> :wat::core::String
  (:wat::core::let
    [va (:wat::holon::encode (:wat::holon::to-holon "a"))
     vb (:wat::holon::encode (:wat::holon::to-holon "b"))]
    (:wat::core::match (:wat::holon::vector-bind va vb)
      ((:wat::holon::CombineOutcome::Combined c1)
        (:wat::core::match (:wat::holon::vector-bind va vb)
          ((:wat::holon::CombineOutcome::Combined c2)
            (:wat::core::if (:wat::core::= c1 c2)  "yes" "no"))
          ((:wat::holon::CombineOutcome::DimensionMismatch _e _g) "mismatch")))
      ((:wat::holon::CombineOutcome::DimensionMismatch _e _g) "mismatch"))))

(:wat::core::defn :valg::bundle-singleton [] -> :wat::core::String
  (:wat::core::let
    [va (:wat::holon::encode (:wat::holon::to-holon "x"))]
    (:wat::core::match
      (:wat::holon::vector-bundle (:wat::core::Vector :- [:wat::holon::Vector] va))
      ((:wat::holon::CombineOutcome::Combined bundled)
        (:wat::core::match (:wat::holon::cosine va bundled)
          ((:wat::holon::CosineOutcome::Similarity s)
            (:wat::core::if (:wat::core::> s 0.99)  "near-1" "far"))
          ((:wat::holon::CosineOutcome::Degenerate _side) "degenerate")
          ((:wat::holon::CosineOutcome::DimensionMismatch _e _g) "mismatch")))
      ((:wat::holon::CombineOutcome::DimensionMismatch _e _g) "mismatch"))))

(:wat::core::defn :valg::blend-weighted [] -> :wat::core::String
  (:wat::core::let
    [va (:wat::holon::encode (:wat::holon::to-holon "x"))
     vb (:wat::holon::encode (:wat::holon::to-holon "y"))]
    (:wat::core::match (:wat::holon::vector-blend va vb 1.0 0.0)
      ((:wat::holon::CombineOutcome::Combined blended)
        (:wat::core::match (:wat::holon::cosine va blended)
          ((:wat::holon::CosineOutcome::Similarity s)
            (:wat::core::if (:wat::core::> s 0.95)  "near-1" "far"))
          ((:wat::holon::CosineOutcome::Degenerate _side) "degenerate")
          ((:wat::holon::CosineOutcome::DimensionMismatch _e _g) "mismatch")))
      ((:wat::holon::CombineOutcome::DimensionMismatch _e _g) "mismatch"))))

(:wat::core::defn :valg::permute-changes [] -> :wat::core::String
  (:wat::core::let
    [va (:wat::holon::encode (:wat::holon::to-holon "x"))
     shifted (:wat::holon::vector-permute va 5)]
    (:wat::core::if (:wat::core::= va shifted)  "same" "differs")))
