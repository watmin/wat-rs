;; tests/reflection/wat_arc201_structured_signature_types_parametric_fn.wat
;; Fixture for test signature_of_defn_emits_structured_parametric_user_fn.
;; Probe: signature-of-defn :user::sum-list emits (Vector :- [i64]) as structured Bundle.
(:wat::core::defn :user::sum-list [init <- wat.type/i64 & xs <- (wat.type/Vector :- [wat.type/i64])] -> wat.type/i64
  (:wat::core::foldl
              (:wat::core::fn [acc <- wat.type/i64 x <- wat.type/i64] -> wat.type/i64
                (:wat::i64::+ acc x))
              init
              xs))

(:wat::core::defn :user::main [] -> wat.type/nil
  (:wat::core::let
              [sig
                (:wat::runtime::signature-of-defn :user::sum-list)
               rendered
                sig]
              (:wat::kernel::println rendered)))
