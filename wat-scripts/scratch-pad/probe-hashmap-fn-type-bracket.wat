;; Scratch probe (arc 255, three-orphans stone) — same question, for HashMap: does
;; eval_hashmap_ctor's is_type_arg_shaped guard (Keyword|List only) reject a
;; well-typed `[T :-> R]` fn-type-bracket K/V that check-time (parse_type_node,
;; via parse_param_spec_slot) accepts?
(:wat::core::def :user::main
  (:wat::core::fn [] -> :wat::core::nil
    (:wat::kernel::println
      (:wat::core::HashMap :- [[:wat::core::i64 :-> :wat::core::bool] :wat::core::i64]))))
