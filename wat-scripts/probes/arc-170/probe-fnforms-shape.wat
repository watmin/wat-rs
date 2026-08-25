;; print the fn-forms output shape for a concrete named work-fn — to see where the
;; arg-type + return-type keywords live (for the parent-side AST-splice).
(:wat::core::defn :my::double [n <- :wat::core::i64] -> :wat::core::i64 (:wat::core::i64::* n 2))
(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::let
    [work-fn (:wat::core::fn [n <- :wat::core::i64] -> :wat::core::i64 (:my::double n))
     forms   (:wat::kernel::fn-forms work-fn :bracket::__pool-work)]
    (:wat::core::do
      (:wat::kernel::println (:wat::edn::write forms))
      (:wat::kernel::println (:wat::string::concat "return-type-of = " (:wat::runtime::return-type-of work-fn))))))
