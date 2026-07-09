;; tests/reflection/wat_arc201_holon_ast_accessors_first_head.wat
;; Fixture for test bundle_first_returns_head_keyword_of_signature.
;; Probe: Bundle/first on signature-of-defn returns the fn name Symbol.
(:wat::core::defn :user::add-two [a <- :wat::core::i64 b <- :wat::core::i64] -> :wat::core::i64 (:wat::core::i64::+ a b))

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::let
              [sig-opt (:wat::runtime::signature-of-defn :user::add-two)
               sig     (:wat::core::match sig-opt -> :wat::holon::HolonAST
                         ((:wat::core::Some s) s)
                         (:wat::core::None     (:wat::kernel::abort "signature-of-defn returned None")))
               head    (:wat::holon::Bundle/first sig)
               rendered head]
              (:wat::kernel::println rendered)))
