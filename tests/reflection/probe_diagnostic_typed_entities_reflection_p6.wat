;; tests/reflection/probe_diagnostic_typed_entities_reflection_p6.wat
;; Fixture for probe_6_bind_left_on_defrecord_instance.
;; Bind/left on the holon-form of a defrecord instance returns Some(left Atom).
(:wat::holon::defrecord :myapp::Voltage [magnitude <- :wat::core::f64])

(:wat::core::defn :user::compute [] -> (:wat::core::Option :- [:wat::holon::HolonAST])
  (:wat::core::let
      [v (:myapp::Voltage :magnitude 5.0)
       h (:wat::holon::to-holon v)]
      (:wat::holon::Bind/left h)))
