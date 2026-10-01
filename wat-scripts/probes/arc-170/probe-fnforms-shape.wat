;; print the fn-forms output shape for a concrete named work-fn — to see where the
;; arg-type + return-type keywords live (for the parent-side AST-splice).
;; CLAIM: fn-forms on a closure capturing a named fn ships exactly 2 forms (the hoisted defn
;; + the bracket-target def alias), and return-type-of reports "wat::core::i64".
(:wat::core::defn :my::double [n <- wat.type/i64] -> wat.type/i64 (:wat::i64::* n 2))
(:wat::core::defn :user::main [] -> wat.type/nil
  (:wat::core::let
    [work-fn (:wat::core::fn [n <- wat.type/i64] -> wat.type/i64 (:my::double n))
     forms   (:wat::kernel::fn-forms work-fn :bracket::__pool-work)
     rt      (:wat::runtime::return-type-of work-fn)]
    (:wat::core::do
      (:wat::test::assert-eq (:wat::core::length forms) 2)
      (:wat::test::assert-eq rt "wat::core::i64")
      (:wat::kernel::println (:wat::edn::write forms))
      (:wat::kernel::println (:wat::string::concat "return-type-of = " rt)))))
