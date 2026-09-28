;; tests/reflection/wat_arc201_signature_of_fn_compose_bundle.wat
;; Fixture for test signature_of_fn_composes_with_bundle_children.
;; Probe: Bundle/children on signature-of-fn output yields structured children.
(:wat::core::defn :user::main [] -> wat.type/nil
  (:wat::core::let
              [f      (:wat::core::fn [peer <- (wat.type/Vector :- [wat.type/String])]
                       -> wat.type/String
                       "ok")
               sig    (:wat::runtime::signature-of-fn f)
               kids   (:wat::core::ast->children sig)
               rendered kids]
              (:wat::kernel::println rendered)))
