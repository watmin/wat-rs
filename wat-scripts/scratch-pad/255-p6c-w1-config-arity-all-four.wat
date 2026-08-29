;; Scratch probe — arc 255 Stone P6-c-W1, acceptance row 3, ALL FOUR verbs.
;;
;; Same mechanism as `255-p6c-w1-config-arity-dim-count.wat` (see that file's header
;; for why `:wat::eval-ast!` + `:wat::core::quote` is needed to reach the RUNTIME
;; arity guard instead of the static type-checker's own arity gate). Each of the
;; four `:wat::config::*` nullary readers, called with one extra argument.

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::do
    (:wat::core::match (:wat::eval-ast! (:wat::core::quote (:wat::config::dim-count 1)))
      ((:wat::core::Ok v) (:wat::kernel::println (:wat::string::concat "dim-count UNEXPECTED ok: " (:wat::edn::write v))))
      ((:wat::core::Err e) (:wat::kernel::println (:wat::string::concat "dim-count kind=" (:wat::core::EvalError/kind e) " message=" (:wat::core::EvalError/message e)))))
    (:wat::core::match (:wat::eval-ast! (:wat::core::quote (:wat::config::dim-capacity 1)))
      ((:wat::core::Ok v) (:wat::kernel::println (:wat::string::concat "dim-capacity UNEXPECTED ok: " (:wat::edn::write v))))
      ((:wat::core::Err e) (:wat::kernel::println (:wat::string::concat "dim-capacity kind=" (:wat::core::EvalError/kind e) " message=" (:wat::core::EvalError/message e)))))
    (:wat::core::match (:wat::eval-ast! (:wat::core::quote (:wat::config::global-seed 1)))
      ((:wat::core::Ok v) (:wat::kernel::println (:wat::string::concat "global-seed UNEXPECTED ok: " (:wat::edn::write v))))
      ((:wat::core::Err e) (:wat::kernel::println (:wat::string::concat "global-seed kind=" (:wat::core::EvalError/kind e) " message=" (:wat::core::EvalError/message e)))))
    (:wat::core::match (:wat::eval-ast! (:wat::core::quote (:wat::config::noise-floor 1)))
      ((:wat::core::Ok v) (:wat::kernel::println (:wat::string::concat "noise-floor UNEXPECTED ok: " (:wat::edn::write v))))
      ((:wat::core::Err e) (:wat::kernel::println (:wat::string::concat "noise-floor kind=" (:wat::core::EvalError/kind e) " message=" (:wat::core::EvalError/message e)))))))
