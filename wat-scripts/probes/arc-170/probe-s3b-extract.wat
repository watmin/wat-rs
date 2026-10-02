;; probe-s3b-extract.wat — verify extraction of the concrete arg/return type keywords
;; off the fn-forms output, and building the tuple-type keyword strings.
;; CLAIM: the hoisted fn-forms def's arg-type node and return-type node are both exactly the
;; i64 type keyword the corpus now spells (`:wat::type::i64`). `wat.type/AST` is Equatable
;; (`wat/class.wat`) and PartialEq
;; on `WatAST` compares structure, skipping span (`crates/wat-reader/src/ast.rs`) — so this is
;; asserted as AST-NODE equality against a node built by `:wat::core::keyword-node`, never as a
;; pair of ast-kind/ast-name STRING comparisons standing in for the node (coordinator correction,
;; 255.76 weigh: "measuring strings is anti-wat"). The kind/name prints stay for a human reader.
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
     expected (:wat::core::keyword-node ":wat::type::i64")]
    (:wat::core::do
      (:wat::test::assert-eq arg-ty expected)
      (:wat::test::assert-eq ret-ty expected)
      (:wat::kernel::println (:wat::string::concat "arg-kind=" (:wat::core::ast-kind arg-ty)))
      (:wat::kernel::println (:wat::string::concat "arg-name=" (:wat::core::ast-name arg-ty)))
      (:wat::kernel::println (:wat::string::concat "ret-kind=" (:wat::core::ast-kind ret-ty)))
      (:wat::kernel::println (:wat::string::concat "ret-name=" (:wat::core::ast-name ret-ty))))))
