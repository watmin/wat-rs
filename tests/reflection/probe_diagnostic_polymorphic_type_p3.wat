;; tests/reflection/probe_diagnostic_polymorphic_type_p3.wat
;; Fixture for probe_3_type_on_bool.
;; (:wat::core::type true) on a literal bool returns "wat::core::bool".
(:wat::core::defn :user::compute [] -> :wat::core::String (:wat::core::type true))
