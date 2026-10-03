;; tests/reflection/wat_arc201_structured_signature_types_alias.wat
;; Fixture for test define_alias_round_trips_on_parametric_signature.
;; Probe: defalias of :wat::core::foldl (parametric) round-trips and executes correctly.
(wat.core/defalias user/my-fold wat.core/foldl)

(wat.core/defn my/compute [] :- wat.type/i64
  (user/my-fold
              (wat.core/fn
                [acc :- wat.type/i64 x :- wat.type/i64] :- wat.type/i64
                (wat.core/+ acc x))
              0
              (wat.type/Vector :- [wat.type/i64] 1 2 3 4)))

(wat.core/defn user/main [] :- wat.type/nil
  (wat.kernel/println
              (wat.i64/to-string (my/compute))))
