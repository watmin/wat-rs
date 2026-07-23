;; tests/collection/vector_algebra.wat — co-located fixture for the sibling probe (.rs),
;; slurped via startup_beside(file!()). Functions RETURN String results for eval_in_frozen.

(:wat::core::defn :valg::bind-roundtrip [] -> :wat::core::String
  (:wat::core::let
    [va (:wat::holon::encode (:wat::holon::to-holon "a"))
     vb (:wat::holon::encode (:wat::holon::to-holon "b"))
     c1 (:wat::holon::vector-bind va vb)
     c2 (:wat::holon::vector-bind va vb)]
    (:wat::core::if (:wat::core::= c1 c2)  "yes" "no")))

(:wat::core::defn :valg::bundle-singleton [] -> :wat::core::String
  (:wat::core::let
    [va (:wat::holon::encode (:wat::holon::to-holon "x"))
     bundled
      (:wat::holon::vector-bundle (:wat::core::Vector :wat::holon::Vector va))
     c (:wat::holon::cosine va bundled)]
    (:wat::core::if (:wat::core::> c 0.99)  "near-1" "far")))

(:wat::core::defn :valg::blend-weighted [] -> :wat::core::String
  (:wat::core::let
    [va (:wat::holon::encode (:wat::holon::to-holon "x"))
     vb (:wat::holon::encode (:wat::holon::to-holon "y"))
     blended (:wat::holon::vector-blend va vb 1.0 0.0)
     c (:wat::holon::cosine va blended)]
    (:wat::core::if (:wat::core::> c 0.95)  "near-1" "far")))

(:wat::core::defn :valg::permute-changes [] -> :wat::core::String
  (:wat::core::let
    [va (:wat::holon::encode (:wat::holon::to-holon "x"))
     shifted (:wat::holon::vector-permute va 5)]
    (:wat::core::if (:wat::core::= va shifted)  "same" "differs")))
