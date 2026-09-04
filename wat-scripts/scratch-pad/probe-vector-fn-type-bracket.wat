;; Scratch probe (arc 255, three-orphans stone) — does a `[T :-> R]` function-type
;; bracket, legal per `parse_type_node`'s WatAST::Vector arm (src/types.rs:5042), as
;; the sole slot inside a Vector constructor's `:-` param-spec, survive check-time as
;; a well-typed T, then trip `eval_vector_ctor`'s runtime match — which only accepts
;; WatAST::Keyword | WatAST::List for args[0], not WatAST::Vector — as MalformedForm?
(:wat::core::def :user::main
  (:wat::core::fn [] -> :wat::core::nil
    (:wat::kernel::println
      (:wat::core::Vector :- [[:wat::core::i64 :-> :wat::core::bool]]))))
