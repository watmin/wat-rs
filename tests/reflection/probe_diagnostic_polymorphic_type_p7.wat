;; tests/reflection/probe_diagnostic_polymorphic_type_p7.wat
;; Fixture for probe_7_type_on_defrecord_instance.
;; (:wat::core::type (:myapp::Voltage 5.0)) on a defrecord instance returns "myapp::Voltage".
(:wat::core::defrecord :myapp::Voltage [magnitude <- :wat::core::f64])

(:wat::core::defn :user::compute [] -> :wat::core::String
  (:wat::core::type (:myapp::Voltage :magnitude 5.0)))
