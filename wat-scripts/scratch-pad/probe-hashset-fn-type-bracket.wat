;; Scratch probe (arc 255, three-orphans stone) — same question as
;; probe-vector-fn-type-bracket.wat, for HashSet: does eval_hashset_ctor's
;; Keyword|List-only match (src/collection/eval.rs) reject a well-typed
;; `[T :-> R]` fn-type-bracket T that check-time (parse_type_node) accepts?
(:wat::core::def :user::main
  (:wat::core::fn [] -> :wat::core::nil
    (:wat::kernel::println
      (:wat::core::HashSet :- [[:wat::core::i64 :-> :wat::core::bool]]))))
