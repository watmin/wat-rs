;; Co-located fixture for wat_math_sqrt.rs — slurped via startup_beside(file!()).

(:wat::core::defn :my::compute-perfect-square [] -> :wat::core::String
  (:wat::f64::to-string (:wat::math::sqrt 16.0)))

(:wat::core::defn :my::compute-sqrt-zero [] -> :wat::core::String
  (:wat::f64::to-string (:wat::math::sqrt 0.0)))

(:wat::core::defn :my::compute-round-trip [] -> :wat::core::String
  (:wat::core::let
    [x  7.5
     rt (:wat::math::sqrt (:wat::core::* x x))]
    (:wat::f64::to-string rt)))

