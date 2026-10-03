;; tests/diagnostics/probe_arc296_record_in_surface_vector.wat — co-located fixture
;;
;; Arc 296 — a record satisfying a surface used as a (Vector :- [Surface]) element.
;;
;; RED at HEAD: infer_list_constructor uses bare unify →
;;   TypeMismatch {:expected ":g::E" :got ":g::Boom"}
;; even though :g::Boom structurally satisfies surface :g::E.
;;
;; GREEN after arc 296 fix: element check uses assignable when elem_ty is a surface,
;; accepting :g::Boom as a valid (Vector :- [:g::E]) element.

(:wat::core::defsurface :g::E
  :nature wat.type/Record
  :features [msg <- wat.type/String])

(:wat::core::defrecord :g::Boom
  [msg <- wat.type/String])

(:wat::core::defn :probe::drive [] -> wat.type/i64
  (:wat::core::let
    [v (wat.type/Vector :- [:g::E] (:g::Boom :msg "x"))]
    (:wat::core::length v)))
