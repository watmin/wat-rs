;; tests/reflection/probe_diagnostic_typed_entities_reflection_p5.wat
;; Fixture for probe_5_composed_walk_to_field_binds.
;; Composed walk: extract-classifier + Bind/right + Bundle/children to get field-Bind list.
;; Coerce to holon-form via :wat::holon::to-holon first (Stone 234.6 migration).
(:wat::holon::defrecord :myapp::Point
  [x <- :wat::core::i64
   y <- :wat::core::i64])

(:wat::core::defn :user::compute [] -> :wat::core::i64
  (:wat::core::let
      [p          (:myapp::Point :x 3 :y 4)
       h          (:wat::holon::to-holon p)
       right-opt  (:wat::holon::Bind/right h)
       right      (:wat::core::Option/expect right-opt "right missing")
       children   (:wat::holon::Bundle/children right)]
      (:wat::vec::length children)))
