;; tests/reflection/probe_diagnostic_typed_entities_reflection_p1.wat
;; Fixture for probe_1_extract_classifier_on_defrecord_instance.
;; extract-classifier on a :wat::core::defrecord instance returns the class name.
(:wat::core::defrecord :myapp::Voltage [magnitude <- :wat::core::f64])

(:wat::core::defn :user::compute [] -> :wat::core::String
  (:wat::core::let
      [v (:myapp::Voltage :magnitude 5.0)]
      (:wat::holon::extract-classifier v)))
