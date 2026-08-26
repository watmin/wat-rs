;; tests/reflection/wat_arc201_signature_of_fn_anon_head.wat
;; Fixture for test signature_of_fn_emits_anonymous_head.
;; Probe: signature-of-fn on an anonymous fn emits :anonymous head keyword.
(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::let
              [f   (:wat::core::fn [a <- :wat::core::i64 b <- :wat::core::i64] -> :wat::core::i64
                     (:wat::i64::+ a b))
               sig (:wat::runtime::signature-of-fn f)
               rendered sig]
              (:wat::kernel::println rendered)))
