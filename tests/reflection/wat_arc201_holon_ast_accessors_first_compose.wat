;; tests/reflection/wat_arc201_holon_ast_accessors_first_compose.wat
;; Fixture for test bundle_first_composes_with_atom_value.
;; Probe: Bundle/first + from-holon extracts the head keyword value.
(:wat::core::defn :user::add-two [a <- :wat::core::i64 b <- :wat::core::i64] -> :wat::core::i64 (:wat::core::i64::+ a b))

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::let
              [sig-opt (:wat::runtime::signature-of-defn :user::add-two)
               sig     (:wat::core::match sig-opt -> :wat::holon::HolonAST
                         ((:wat::core::Some s) s)
                         (:wat::core::None     (:wat::kernel::abort "signature-of-defn returned None")))
               head    (:wat::holon::Bundle/first sig)
               name-kw (:wat::holon::from-holon head)
               rendered name-kw]
              (:wat::kernel::println rendered)))
