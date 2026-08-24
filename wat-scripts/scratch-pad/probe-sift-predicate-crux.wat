;; disconfirming probe — arc 278 sift Predicate crux, at RUNTIME (via :user::main).
;;
;; Does the predicate-as-EDN-source chain compose + produce a CALLABLE fn value?
;;   String of EDN source -> read-string -> :wat::WatAST
;;   pure? / deterministic?  -> verify the quoted form (the no-hidden-failures gate)
;;   eval-ast! -> (Result :- [fn]), unwrapped -> a :wat::core::fn value
;;   apply -> the fn value called on one record -> :bool
;; GREEN (pure=true det=true keeps5=true drops2=false) => the chain works => sift-logs briefable.
;; If eval-ast! of a fn-form Err's at runtime too, that's the trap — re-plan the carry.

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::let
    [pred-src "(:wat::core::fn [n <- :wat::core::i64] -> :wat::core::bool (:wat::core::> n 3))"
     form     (:wat::core::match (:wat::core::read-string pred-src) ((:wat::core::ReadOutcome::Forms __forms) __forms) ((:wat::core::ReadOutcome::Malformed __cause) (:wat::kernel::assertion-failed! (:wat::core::Error/message __cause) :wat::core::None :wat::core::None)))
     pure     (:wat::rete::pure? form)
     det      (:wat::rete::deterministic? form)
     pred-fn  (:wat::core::Result/expect (:wat::eval-ast! form) "eval-ast! failed")
     keeps5   (:wat::core::apply  pred-fn [5])
     drops2   (:wat::core::apply  pred-fn [2])]
    (:wat::kernel::println
      (:wat::core::str "pure=" pure " det=" det " keeps5=" keeps5 " drops2=" drops2))))
