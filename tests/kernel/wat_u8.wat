;; Co-located fixture for wat_u8.rs — slurped via startup_beside(file!()).
;; The negative startup test (type mismatch) uses wat_u8.wat.bad.
;; compute-u8-256 and compute-u8-neg1 error at eval time (runtime bounds check).

(:wat::core::defn :my::compute-u8-42 [] -> :wat::core::u8
  (:wat::core::u8 42))

(:wat::core::defn :my::compute-u8-zero [] -> :wat::core::u8
  (:wat::core::u8 0))

(:wat::core::defn :my::compute-u8-max [] -> :wat::core::u8
  (:wat::core::u8 255))

(:wat::core::defn :my::compute-u8-256 [] -> :wat::core::u8
  (:wat::core::u8 256))

(:wat::core::defn :my::compute-u8-neg1 [] -> :wat::core::u8
  (:wat::core::u8 -1))

(:wat::core::defn :my::compute-u8-eq [] -> :wat::core::bool
  (:wat::core::= (:wat::core::u8 10) (:wat::core::u8 10)))

(:wat::core::defn :my::compute-u8-neq [] -> :wat::core::bool
  (:wat::core::= (:wat::core::u8 10) (:wat::core::u8 11)))

(:wat::core::defn :my::compute-vec-u8 [] -> (:wat::core::Vector :- [:wat::core::u8])
  (:wat::core::Vector :- [:wat::core::u8]
    (:wat::core::u8 0)
    (:wat::core::u8 65)
    (:wat::core::u8 127)
    (:wat::core::u8 255)))

(:wat::core::defn :my::app::identity [b <- :wat::core::u8] -> :wat::core::u8 b)

(:wat::core::defn :my::compute-identity [] -> :wat::core::u8
  (:my::app::identity (:wat::core::u8 100)))

