;; tests/collection/vector_first_class.wat — co-located fixture for the sibling probe (.rs),
;; slurped via startup_beside(file!()). Functions RETURN String results for eval_in_frozen.
;; Arc 294.a: cosine accepts any EdnRepresentable value, lifting via to_holon_inner.

(:wat::core::defn :vfc::construct-via-encode [] -> :wat::core::String
  (:wat::core::let
    [v1 (:wat::holon::encode (:wat::holon::to-holon "x"))
     v2 (:wat::holon::encode (:wat::holon::to-holon "x"))]
    (:wat::core::if (:wat::core::= v1 v2)  "equal" "diff")))

(:wat::core::defn :vfc::distinct-atoms [] -> :wat::core::String
  (:wat::core::let
    [va (:wat::holon::encode (:wat::holon::to-holon "alpha"))
     vb (:wat::holon::encode (:wat::holon::to-holon "beta"))]
    (:wat::core::if (:wat::core::= va vb)  "same" "diff")))

(:wat::core::defstruct :my::Engram
  [label <- :wat::core::String
   vec   <- :wat::holon::Vector])

(:wat::core::defn :vfc::struct-field-roundtrip [] -> :wat::core::String
  (:wat::core::let
    [v (:wat::holon::encode (:wat::holon::to-holon "x"))
     e (:my::Engram :label "alpha" :vec v)
     retrieved (:my::Engram/vec e)]
    (:wat::core::if (:wat::core::= v retrieved)  "yes" "no")))

;; Arc 278 the cosine outcome wall — cosine and dot now return matchable
;; CosineOutcome/DotOutcome instead of a bare f64; every call site faces
;; the Degenerate/DimensionMismatch cases exhaustively (no `_` arm).
;; None of these fixtures can reach either domain hole (same-dim, non-zero
;; vectors), but the match must still name every variant.

(:wat::core::defn :vfc::cosine-ast-ast [] -> :wat::core::String
  (:wat::core::let
    [a (:wat::holon::to-holon "x")
     b (:wat::holon::to-holon "x")]
    (:wat::core::match (:wat::holon::cosine a b)
      ((:wat::holon::CosineOutcome::Similarity c)
        (:wat::core::if (:wat::core::> c 0.99)  "near-1" "far"))
      ((:wat::holon::CosineOutcome::Degenerate _side) "degenerate")
      ((:wat::holon::CosineOutcome::DimensionMismatch _e _g) "mismatch"))))

(:wat::core::defn :vfc::cosine-vec-vec [] -> :wat::core::String
  (:wat::core::let
    [va (:wat::holon::encode (:wat::holon::to-holon "x"))
     vb (:wat::holon::encode (:wat::holon::to-holon "x"))]
    (:wat::core::match (:wat::holon::cosine va vb)
      ((:wat::holon::CosineOutcome::Similarity c)
        (:wat::core::if (:wat::core::> c 0.99)  "near-1" "far"))
      ((:wat::holon::CosineOutcome::Degenerate _side) "degenerate")
      ((:wat::holon::CosineOutcome::DimensionMismatch _e _g) "mismatch"))))

(:wat::core::defn :vfc::cosine-ast-vec [] -> :wat::core::String
  (:wat::core::let
    [a (:wat::holon::to-holon "x")
     vb (:wat::holon::encode (:wat::holon::to-holon "x"))]
    (:wat::core::match (:wat::holon::cosine a vb)
      ((:wat::holon::CosineOutcome::Similarity c)
        (:wat::core::if (:wat::core::> c 0.99)  "near-1" "far"))
      ((:wat::holon::CosineOutcome::Degenerate _side) "degenerate")
      ((:wat::holon::CosineOutcome::DimensionMismatch _e _g) "mismatch"))))

(:wat::core::defn :vfc::cosine-vec-ast [] -> :wat::core::String
  (:wat::core::let
    [va (:wat::holon::encode (:wat::holon::to-holon "x"))
     b (:wat::holon::to-holon "x")]
    (:wat::core::match (:wat::holon::cosine va b)
      ((:wat::holon::CosineOutcome::Similarity c)
        (:wat::core::if (:wat::core::> c 0.99)  "near-1" "far"))
      ((:wat::holon::CosineOutcome::Degenerate _side) "degenerate")
      ((:wat::holon::CosineOutcome::DimensionMismatch _e _g) "mismatch"))))

(:wat::core::defn :vfc::dot-vec-vec [] -> :wat::core::String
  (:wat::core::let
    [va (:wat::holon::encode (:wat::holon::to-holon "x"))
     vb (:wat::holon::encode (:wat::holon::to-holon "x"))]
    (:wat::core::match (:wat::holon::dot va vb)
      ((:wat::holon::DotOutcome::Computed d)
        (:wat::core::if (:wat::core::> d 0.0)  "positive" "non-positive"))
      ((:wat::holon::DotOutcome::DimensionMismatch _e _g) "mismatch"))))

(:wat::core::defn :vfc::simhash-agree [] -> :wat::core::String
  (:wat::core::let
    [ast (:wat::holon::to-holon "alpha")
     vec (:wat::holon::encode ast)
     k-ast (:wat::holon::simhash ast)
     k-vec (:wat::holon::simhash vec)]
    (:wat::core::if (:wat::core::= k-ast k-vec)  "same" "diff")))

;; Arc 278 the cosine outcome wall — cosine's return type is now
;; :wat::holon::CosineOutcome, not a bare f64; this fixture exists purely to
;; force type-checking of cosine on String args at startup_beside time (see
;; the co-located .rs's `polymorphic_cosine_accepts_string`), so it returns
;; the outcome directly rather than narrowing it.
(:wat::core::defn :vfc::cosine-string [] -> :wat::holon::CosineOutcome
  (:wat::holon::cosine "hello" "world"))

(:wat::core::defn :vfc::encode-deterministic [] -> :wat::core::String
  (:wat::core::let
    [a
      (:wat::holon::Bind
        (:wat::holon::to-holon "role")
        (:wat::holon::to-holon "filler"))
     b
      (:wat::holon::Bind
        (:wat::holon::to-holon "role")
        (:wat::holon::to-holon "filler"))
     va (:wat::holon::encode a)
     vb (:wat::holon::encode b)]
    (:wat::core::if (:wat::core::= va vb)  "deterministic" "drift")))
