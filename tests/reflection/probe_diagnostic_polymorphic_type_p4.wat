;; tests/reflection/probe_diagnostic_polymorphic_type_p4.wat
;; Fixture for probe_4_type_on_keyword.
;; (:wat::core::type :foo) on a literal keyword returns "wat::core::keyword".
(:wat::core::defn :user::compute [] -> :wat::core::String (:wat::core::type :foo))
