;; tests/reflection/wat_arc201_holon_ast_accessors_children_sig.wat
;; Fixture for test bundle_children_returns_vec_of_holonast_from_signature.
;; Probe: Bundle/children on signature-of-defn :user::add-two returns structured children.
(:wat::core::defn :user::add-two [a <- :wat::core::i64 b <- :wat::core::i64] -> :wat::core::i64 (:wat::i64::+ a b))

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::let
              [sig-opt (:wat::runtime::signature-of-defn :user::add-two)
               sig     (:wat::core::match sig-opt 
                         ((:wat::core::Some s) s)
                         (:wat::core::None     (:wat::kernel::abort "signature-of-defn returned None")))
               kids    (:wat::core::ast->children sig)
               rendered kids]
              (:wat::kernel::println rendered)))
