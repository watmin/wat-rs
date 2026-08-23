;; tests/reflection/probe_diagnostic_typed_entities_reflection_p3.wat
;; Fixture for probe_3_bind_right_on_defrecord_instance.
;; Bind/right on the holon-form of a defrecord instance returns Some(right Bundle).
(:wat::holon::defrecord :myapp::Voltage [magnitude <- :wat::core::f64])

(:wat::core::defn :user::compute [] -> (:wat::core::Option :- [:wat::holon::HolonAST])
  (:wat::core::let
      [v (:myapp::Voltage :magnitude 5.0)
       h (:wat::holon::to-holon v)]
      (:wat::holon::Bind/right h)))
