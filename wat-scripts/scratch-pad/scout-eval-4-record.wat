;; scout-eval-4: realistic (fn [log] -> :bool ...) over a real record, per-record apply.
;; Plus: impure predicate must be REJECTED by the pure? gate.
(:wat::core::defrecord :user::Log
  [level   <- :wat::core::keyword
   message <- :wat::core::String])

(:wat::core::defn :user::unwrap-form
  [src <- :wat::core::String] -> :wat::WatAST
  (:wat::core::first (:wat::core::ast->children (:wat::core::read-string src))))

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::let
    [;; a pure predicate over a Log record
     pred-src "(:wat::core::fn [log <- :user::Log] -> :wat::core::bool (:wat::core::= (:user::Log/level log) :error))"
     form     (:user::unwrap-form pred-src)
     pure     (:wat::rete::pure? form)
     det      (:wat::rete::deterministic? form)
     pred-fn  (:wat::core::Result/expect (:wat::eval-ast! form) "eval-ast! failed")
     err-log  (:user::Log :level :error   :message "boom")
     info-log (:user::Log :level :info    :message "ok")
     keeps-err (:wat::core::apply -> :wat::core::bool pred-fn [err-log])
     drops-info (:wat::core::apply -> :wat::core::bool pred-fn [info-log])
     ;; an IMPURE predicate — calls println (effectful)
     imp-src  "(:wat::core::fn [log <- :user::Log] -> :wat::core::bool (:wat::kernel::println \"side\"))"
     imp-form (:user::unwrap-form imp-src)
     imp-pure (:wat::rete::pure? imp-form)]
    (:wat::kernel::println (:wat::core::str pure))
    (:wat::kernel::println (:wat::core::str det))
    (:wat::kernel::println (:wat::core::str keeps-err))
    (:wat::kernel::println (:wat::core::str drops-info))
    (:wat::kernel::println (:wat::core::str imp-pure))))
