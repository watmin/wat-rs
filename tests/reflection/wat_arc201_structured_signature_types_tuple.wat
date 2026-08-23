;; tests/reflection/wat_arc201_structured_signature_types_tuple.wat
;; Fixture for test signature_of_defn_emits_structured_tuple_return_type.
;; Probe: signature-of-defn :user::make-pair emits :Tuple Bundle head.
(:wat::core::defn :user::make-pair [] -> (:wat::core::Tuple :- [:wat::core::i64 :wat::core::String]) (:wat::core::Tuple 42 "hi"))

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::let
              [sig
                (:wat::runtime::signature-of-defn :user::make-pair)
               rendered
                sig]
              (:wat::kernel::println rendered)))
