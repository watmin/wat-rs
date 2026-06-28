;; tests/reflection/probe_diagnostic_polymorphic_type_p6.wat
;; Fixture for probe_6_type_on_hashmap.
;; (:wat::core::type {:a 1}) on a HashMap literal returns "wat::core::HashMap".
(:wat::core::defn :user::compute [] -> :wat::core::String (:wat::core::type {:a 1}))
