;; Does a generic fn substitute its type param I into a `forms` quote when called concretely?
;; CLAIM (measured, decisive NO): calling (probe::mk 5) does NOT substitute the caller's
;; concrete type (i64) into the quoted inner defn — both the arg type and the return type
;; keyword nodes in the emitted form are still the literal, un-substituted symbol `:I`.
;; `forms` is a reflective AST quote, not a monomorphizing template. Asserted structurally
;; via ast-kind/ast-name on the actual type-keyword nodes (never a rendered edn/write string).
(:wat::core::defn :probe::mk :- [I]
  [x <- :I] -> (wat.type/Vector :- [wat.type/AST])
  (:wat::core::forms
    (:wat::core::defn :probe::inner [a <- :I] -> :I a)))

(:wat::core::defn :user::main [] -> wat.type/nil
  (:wat::core::let
    [forms     (:probe::mk 5)
     defn-node (:wat::core::nth forms 0)
     ch        (:wat::core::ast->children defn-node)
     ;; ch = [defn-kw, name-kw, argspec-vec, ->-sym, ret-type-kw, body]
     argspec   (:wat::core::nth ch 2)
     arg-ch    (:wat::core::ast->children argspec)
     ;; arg-ch = [a-sym, <--sym, I-kw] (one param, flat)
     arg-ty    (:wat::core::nth arg-ch 2)
     ret-ty    (:wat::core::nth ch 4)]
    (:wat::core::do
      (:wat::test::assert-eq (:wat::core::ast-kind arg-ty) "keyword")
      (:wat::test::assert-eq (:wat::core::ast-name arg-ty) ":I")
      (:wat::test::assert-eq (:wat::core::ast-kind ret-ty) "keyword")
      (:wat::test::assert-eq (:wat::core::ast-name ret-ty) ":I")
      (:wat::kernel::println (:wat::edn::write forms)))))
