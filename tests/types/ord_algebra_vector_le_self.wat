;; ord_algebra_vector_le_self.wat — v <= v is true
(wat.core/defn user/compute [] :- wat.type/bool
  (wat.core/let
    [v (wat.holon/encode (wat.holon/to-holon "x"))]
    (wat.core/<= v v)))
