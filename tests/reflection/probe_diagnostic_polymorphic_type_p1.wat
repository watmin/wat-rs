;; tests/reflection/probe_diagnostic_polymorphic_type_p1.wat
;; Fixture for probe_1_type_on_i64.
;; (:wat::core::type 5) on a literal i64 returns "wat::core::i64".
(:wat::core::defn :user::compute [] -> :wat::core::String (:wat::core::type 5))
