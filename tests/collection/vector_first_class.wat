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

(:wat::core::defn :vfc::cosine-ast-ast [] -> :wat::core::String
  (:wat::core::let
    [a (:wat::holon::to-holon "x")
     b (:wat::holon::to-holon "x")
     c (:wat::holon::cosine a b)]
    (:wat::core::if (:wat::core::> c 0.99)  "near-1" "far")))

(:wat::core::defn :vfc::cosine-vec-vec [] -> :wat::core::String
  (:wat::core::let
    [va (:wat::holon::encode (:wat::holon::to-holon "x"))
     vb (:wat::holon::encode (:wat::holon::to-holon "x"))
     c (:wat::holon::cosine va vb)]
    (:wat::core::if (:wat::core::> c 0.99)  "near-1" "far")))

(:wat::core::defn :vfc::cosine-ast-vec [] -> :wat::core::String
  (:wat::core::let
    [a (:wat::holon::to-holon "x")
     vb (:wat::holon::encode (:wat::holon::to-holon "x"))
     c (:wat::holon::cosine a vb)]
    (:wat::core::if (:wat::core::> c 0.99)  "near-1" "far")))

(:wat::core::defn :vfc::cosine-vec-ast [] -> :wat::core::String
  (:wat::core::let
    [va (:wat::holon::encode (:wat::holon::to-holon "x"))
     b (:wat::holon::to-holon "x")
     c (:wat::holon::cosine va b)]
    (:wat::core::if (:wat::core::> c 0.99)  "near-1" "far")))

(:wat::core::defn :vfc::dot-vec-vec [] -> :wat::core::String
  (:wat::core::let
    [va (:wat::holon::encode (:wat::holon::to-holon "x"))
     vb (:wat::holon::encode (:wat::holon::to-holon "x"))
     d (:wat::holon::dot va vb)]
    (:wat::core::if (:wat::core::> d 0.0)  "positive" "non-positive")))

(:wat::core::defn :vfc::simhash-agree [] -> :wat::core::String
  (:wat::core::let
    [ast (:wat::holon::to-holon "alpha")
     vec (:wat::holon::encode ast)
     k-ast (:wat::holon::simhash ast)
     k-vec (:wat::holon::simhash vec)]
    (:wat::core::if (:wat::core::= k-ast k-vec)  "same" "diff")))

(:wat::core::defn :vfc::cosine-string [] -> :wat::core::f64
  (:wat::core::let [sim (:wat::holon::cosine "hello" "world")] sim))

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
