;; tests/reflection/wat_arc201_structured_signature_types_foldl.wat
;; Fixture for test signature_of_defn_foldl_emits_structured_parametric_and_fn.
;; Probe: signature-of-defn :wat::core::foldl emits structured Parametric + Fn Bundles.
(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::let
              [sig
                (:wat::runtime::signature-of-defn :wat::core::foldl)
               rendered
                (:wat::edn::write sig)]
              (:wat::kernel::println rendered)))
