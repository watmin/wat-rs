;; tests/reflection/probe_diagnostic_polymorphic_type_p5.wat
;; Fixture for probe_5_type_on_vector.
;; (:wat::core::type [1 2 3]) on a Vector literal returns "wat::type::Vector".
(wat.core/defn user/compute [] :- wat.type/String (wat.core/type [1 2 3]))
