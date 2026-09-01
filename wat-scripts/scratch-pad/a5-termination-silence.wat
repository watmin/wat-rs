;; A5 recon — does `compile-all` distinguish "termination PROVEN" from "nothing to analyse"?
;;
;; `stratify::refuse_non_terminating` returns `Ok(())` from four places: rules-is-not-a-vector
;; (:838), nothing-computes (:894), the graph found no unbounded cycle (:988) — and it `continue`s
;; past any rule whose lhs/rhs are empty (:853), the shape an imported Export has.
;;
;; An empty-AST Rule value is that shape, and it is writable here. If compile-all answers
;; `Compiled` for it, the verdict "terminates" and the verdict "was never looked at" are the same
;; value to every caller.
(:wat::core::defrecord :a5::In  [v <- :wat::core::i64])

(:wat::core::defn :a5::ast-less-rule [] -> :wat::rete::Rule
  (:wat::rete::Rule :name "ast-less"
    :lhs (:wat::core::PersistentVector)
    :rhs (:wat::core::PersistentVector)))

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::let
    [rules   (:wat::core::PersistentVector :- [:wat::rete::Rule] (:a5::ast-less-rule))
     verdict (:wat::core::match (:wat::rete::compile-all rules (:wat::core::PersistentVector))
               ((:wat::rete::CompileOutcome::Compiled __s) "Compiled")
               ((:wat::rete::CompileOutcome::MayNotTerminate __r __f) "MayNotTerminate"))]
    (:wat::kernel::println verdict)))
