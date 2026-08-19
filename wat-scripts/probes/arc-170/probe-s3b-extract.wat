;; probe-s3b-extract.wat — verify extraction of the concrete arg/return type keywords
;; off the fn-forms output, and building the tuple-type keyword strings.
(:wat::core::defn :my::double [n <- :wat::core::i64] -> :wat::core::i64 (:wat::core::i64::* n 2))

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::let
    [work-fn  (:wat::core::fn [n <- :wat::core::i64] -> :wat::core::i64 (:my::double n))
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
     arg-name (:wat::core::ast-name arg-ty)
     ret-name (:wat::core::ast-name ret-ty)]
    (:wat::core::do
      (:wat::kernel::println (:wat::core::string::concat "arg-kind=" (:wat::core::ast-kind arg-ty)))
      (:wat::kernel::println (:wat::core::string::concat "arg-name=" arg-name))
      (:wat::kernel::println (:wat::core::string::concat "ret-kind=" (:wat::core::ast-kind ret-ty)))
      (:wat::kernel::println (:wat::core::string::concat "ret-name=" ret-name)))))
