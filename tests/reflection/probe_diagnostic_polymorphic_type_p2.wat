;; tests/reflection/probe_diagnostic_polymorphic_type_p2.wat
;; Fixture for probe_2_type_on_string.
;; (:wat::core::type "hello") on a literal String returns "wat::core::String".
(:wat::core::defn :user::compute [] -> :wat::core::String (:wat::core::type "hello"))
