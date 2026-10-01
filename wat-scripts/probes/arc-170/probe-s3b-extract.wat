;; probe-s3b-extract.wat — verify extraction of the concrete arg/return type keywords
;; off the fn-forms output, and building the tuple-type keyword strings.
;; CLAIM: the hoisted fn-forms def's arg type and return type are both the keyword node
;; ":wat::core::i64" — ast-kind == "keyword" and ast-name == ":wat::core::i64" for each.
(:wat::core::defn :my::double [n <- wat.type/i64] -> wat.type/i64 (:wat::i64::* n 2))

(:wat::core::defn :user::main [] -> wat.type/nil
  (:wat::core::let
    [work-fn  (:wat::core::fn [n <- wat.type/i64] -> wat.type/i64 (:my::double n))
     forms    (:wat::kernel::fn-forms work-fn :bracket::__pool-work)
     def-node (:wat::core::Option/expect (:wat::core::last forms) "no def")
     def-ch   (:wat::core::ast->children def-node)
     fn-form  (:wat::core::nth def-ch 2)
     fn-ch    (:wat::core::ast->children fn-form)
     ;; fn-ch = [fn-kw, argspec-vec, ->-sym, ret-type-kw, body...]
     argspec  (:wat::core::nth fn-ch 1)
     arg-ch   (:wat::core::ast->children argspec)
     ;; arg-ch = [n-sym, <--sym, argtype-kw]
     arg-ty   (:wat::core::Option/expect (:wat::core::last arg-ch) "no argty")
     ret-ty   (:wat::core::nth fn-ch 3)
     arg-kind (:wat::core::ast-kind arg-ty)
     arg-name (:wat::core::ast-name arg-ty)
     ret-kind (:wat::core::ast-kind ret-ty)
     ret-name (:wat::core::ast-name ret-ty)]
    (:wat::core::do
      (:wat::test::assert-eq arg-kind "keyword")
      (:wat::test::assert-eq arg-name ":wat::core::i64")
      (:wat::test::assert-eq ret-kind "keyword")
      (:wat::test::assert-eq ret-name ":wat::core::i64")
      (:wat::kernel::println (:wat::string::concat "arg-kind=" arg-kind))
      (:wat::kernel::println (:wat::string::concat "arg-name=" arg-name))
      (:wat::kernel::println (:wat::string::concat "ret-kind=" ret-kind))
      (:wat::kernel::println (:wat::string::concat "ret-name=" ret-name)))))
