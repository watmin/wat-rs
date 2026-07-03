;; Co-located fixture for wat_simhash.rs — slurped via startup_beside(file!()).
;; Each fn returns a String label rather than println-ing it.

(:wat::core::defn :my::compute-deterministic [] -> :wat::core::String
  (:wat::core::let
    [a  (:wat::holon::Bind
          (:wat::holon::to-holon "role")
          (:wat::holon::to-holon "filler"))
     k1 (:wat::holon::simhash a)
     k2 (:wat::holon::simhash a)]
    (:wat::core::if (:wat::core::= k1 k2) -> :wat::core::String "yes" "no")))

(:wat::core::defn :my::compute-atom-stable [] -> :wat::core::String
  (:wat::core::let
    [k1 (:wat::holon::simhash (:wat::holon::to-holon 0))
     k2 (:wat::holon::simhash (:wat::holon::to-holon 0))]
    (:wat::core::if (:wat::core::= k1 k2) -> :wat::core::String "yes" "no")))

(:wat::core::defn :my::compute-same-shape [] -> :wat::core::String
  (:wat::core::let
    [a  (:wat::holon::Bind
          (:wat::holon::to-holon "role")
          (:wat::holon::to-holon "filler"))
     b  (:wat::holon::Bind
          (:wat::holon::to-holon "role")
          (:wat::holon::to-holon "filler"))
     k1 (:wat::holon::simhash a)
     k2 (:wat::holon::simhash b)]
    (:wat::core::if (:wat::core::= k1 k2) -> :wat::core::String "same" "diff")))

(:wat::core::defn :my::compute-distinct-atoms [] -> :wat::core::String
  (:wat::core::let
    [alpha (:wat::holon::to-holon "alpha")
     beta  (:wat::holon::to-holon "beta")
     k-a   (:wat::holon::simhash alpha)
     k-b   (:wat::holon::simhash beta)]
    (:wat::core::if (:wat::core::= k-a k-b) -> :wat::core::String "same" "diff")))

(:wat::core::defn :my::compute-arithmetic [] -> :wat::core::String
  (:wat::core::let
    ;; a simhash result is a usable :wat::core::i64 in arithmetic. Arc 300 C3 —
    ;; `(+ k k)` on a hash (a large i64) OVERFLOWS, which now honestly errors
    ;; (don't-wrap-error); overflow-tosses is covered by probe_rational_C3_i64_overflow.
    ;; Here we demonstrate usability with a non-overflowing op.
    [k    (:wat::holon::simhash (:wat::holon::to-holon "x"))
     zero (:wat::core::- k k)]
    "ok"))

