;; Scratch probe — arc 255 Stone O, the DISCONFIRMING probe drawn BEFORE the brief.
;;
;; THE CLAIM UNDER TEST: `:wat::core::apply` answers from its own picture of what
;; exists, not from the registry. A verb that is registered and works when called
;; DIRECTLY is reported "unknown function" through `apply` — unless it happens to
;; carry a `value = <path>` slot (44 of 381 names do, arc 255 Stone N).
;;
;; Each row evaluates the SAME verb twice — through the AST door and through the
;; apply door — and prints both outcomes side by side. A row printing
;;   DIRECT=ok   APPLY=err:unknown-function
;; is the lie: the registry knows the name, and apply says it does not.
;;
;; Rows are chosen to straddle the seam: `:wat::i64::+` HAS a value_handler and is
;; expected to answer on both doors; the rest do not and are expected to split.
;; If a row ever prints DIRECT=err the probe itself is wrong — read that first.

(:wat::core::defn :probe::outcome [r <- (:wat::core::Result :wat::core::Value :wat::core::EvalError)]
  -> :wat::core::String
  (:wat::core::match r
    ((:wat::core::Ok v)  (:wat::string::concat "ok:" (:wat::edn::write v)))
    ((:wat::core::Err e) (:wat::string::concat "err:" (:wat::core::EvalError/message e)))))

(:wat::core::defn :probe::both [name   <- :wat::core::String
                                direct <- :wat::WatAST
                                thru   <- :wat::WatAST]
  -> :wat::core::nil
  (:wat::kernel::println
    (:wat::string::concat
      name
      "  DIRECT=" (:probe::outcome (:wat::eval-ast! direct))
      "  APPLY="  (:probe::outcome (:wat::eval-ast! thru)))))

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::let
    [;; HAS a value_handler (Stone N) — expected to answer on BOTH doors.
     _01 (:probe::both ":wat::i64::+       [HASVAL]"
           (:wat::core::quote (:wat::i64::+ 20 22))
           (:wat::core::quote (:wat::core::apply :wat::i64::+ (:wat::core::Vector :wat::core::i64 20 22))))

     ;; NO value_handler — registered, works directly, invisible to apply.
     _02 (:probe::both ":wat::f64::max-of  [no val]"
           (:wat::core::quote (:wat::f64::max-of 3.0 9.0 41.0))
           (:wat::core::quote (:wat::core::apply :wat::f64::max-of (:wat::core::Vector :wat::core::f64 3.0 9.0 41.0))))

     _03 (:probe::both ":wat::string::to-uppercase [no val]"
           (:wat::core::quote (:wat::string::to-uppercase "wat"))
           (:wat::core::quote (:wat::core::apply :wat::string::to-uppercase (:wat::core::Vector :wat::core::String "wat"))))

     _04 (:probe::both ":wat::vector::length[no val]"
           (:wat::core::quote (:wat::vector::length (:wat::core::PersistentVector 1 2 3)))
           (:wat::core::quote (:wat::core::apply :wat::vector::length (:wat::core::Vector (:wat::core::PersistentVector :- [:wat::core::i64]) (:wat::core::PersistentVector 1 2 3)))))

     _05 (:probe::both ":wat::math::sqrt   [no val]"
           (:wat::core::quote (:wat::math::sqrt 16.0))
           (:wat::core::quote (:wat::core::apply :wat::math::sqrt (:wat::core::Vector :wat::core::f64 16.0))))]
    nil))
