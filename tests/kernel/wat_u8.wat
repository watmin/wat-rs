;; Co-located fixture for wat_u8.rs — slurped via startup_beside(file!()).
;; The negative startup test (type mismatch) uses wat_u8.wat.bad.
;; compute-u8-256 and compute-u8-neg1 error at eval time (runtime bounds check).

(:wat::core::defn :my::compute-u8-42 [] -> wat.type/u8
  (wat.type/u8 42))

(:wat::core::defn :my::compute-u8-zero [] -> wat.type/u8
  (wat.type/u8 0))

(:wat::core::defn :my::compute-u8-max [] -> wat.type/u8
  (wat.type/u8 255))

(:wat::core::defn :my::compute-u8-256 [] -> wat.type/u8
  (wat.type/u8 256))

(:wat::core::defn :my::compute-u8-neg1 [] -> wat.type/u8
  (wat.type/u8 -1))

(:wat::core::defn :my::compute-u8-eq [] -> wat.type/bool
  (:wat::core::= (wat.type/u8 10) (wat.type/u8 10)))

(:wat::core::defn :my::compute-u8-neq [] -> wat.type/bool
  (:wat::core::= (wat.type/u8 10) (wat.type/u8 11)))

(:wat::core::defn :my::compute-vec-u8 [] -> (wat.type/Vector :- [wat.type/u8])
  (wat.type/Vector :- [wat.type/u8]
    (wat.type/u8 0)
    (wat.type/u8 65)
    (wat.type/u8 127)
    (wat.type/u8 255)))

(:wat::core::defn :my::app::identity [b <- wat.type/u8] -> wat.type/u8 b)

(:wat::core::defn :my::compute-identity [] -> wat.type/u8
  (:my::app::identity (wat.type/u8 100)))

