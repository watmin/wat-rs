;; Scratch probe — arc 255 Stone O-i, acceptance row 4.
;;
;; THE CLAIM: after the value-door arity guard, a wrong-arity `apply` reports the ERROR
;; KIND `arity-mismatch` (`RuntimeErrorKind::ArityMismatch`, surfaced by
;; `runtime_error_to_eval_error_value` at src/runtime.rs:22452 as the EvalError struct's
;; `kind` field) — NOT `unknown-function`. This is STOP-2's positive form: a guard that
;; returned `None` instead of `Some(Err(...))` would fall through to `eval_apply`'s
;; "unknown function" path for a verb that plainly exists, trading one lie for another.

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::match (:wat::eval-ast! (:wat::core::quote
                        (:wat::core::apply :wat::i64::+ (:wat::core::Vector :- [:wat::core::i64] 20))))
    ((:wat::core::Ok _) (:wat::kernel::println "UNEXPECTED: ok"))
    ((:wat::core::Err e)
      (:wat::kernel::println (:wat::string::concat "kind=" (:wat::core::EvalError/kind e)
                                " message=" (:wat::core::EvalError/message e))))))
