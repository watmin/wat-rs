;; tests/reflection/probe_diagnostic_polymorphic_type_p8.wat
;; Fixture for probe_8_type_on_struct_instance.
;; (:wat::core::type point-instance) on a struct instance returns the FQDN without leading colon.
(:wat::core::defstruct :myapp::Point [x <- :wat::core::i64 y <- :wat::core::i64])

(:wat::core::defn :user::compute [] -> :wat::core::String
  (:wat::core::type (:wat::core::struct-new :myapp::Point 3 4)))
