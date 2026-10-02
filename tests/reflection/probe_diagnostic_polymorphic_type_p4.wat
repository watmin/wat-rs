;; tests/reflection/probe_diagnostic_polymorphic_type_p4.wat
;; Fixture for probe_4_type_on_keyword.
;; (:wat::core::type :foo) on a literal keyword returns "wat::type::keyword".
(:wat::core::defn :user::compute [] -> wat.type/String (:wat::core::type :foo))
