;; tests/reflection/wat_arc201_extract_arg_types_bundle_children.wat
;; Fixture for test extract_arg_types_composes_with_bundle_children_on_parametric.
;; Probe: D2 chain — extract-arg-types → Bundle/children decomposes the Vector<i64> slot.
(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::let
              [f       (:wat::core::fn [xs <- :wat::core::Vector<wat::core::i64>]
                         -> :wat::core::i64
                         42)
               sig     (:wat::runtime::signature-of-fn f)
               tys     (:wat::runtime::extract-arg-types sig)
               ;; The Vector param is the only arg; grab it via get index 0.
               ;; get returns Option; unwrap with Option/expect.
               ty0     (:wat::core::Option/expect
                         (:wat::core::get tys 0)
                         "expected first type entry")
               ;; Decompose the Bundle: head = :wat::core::Vector, arg = :wat::core::i64
               parts   (:wat::holon::Bundle/children ty0)
               rendered (:wat::edn::write parts)]
              (:wat::kernel::println rendered)))
