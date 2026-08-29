;; Scratch probe — arc 255 Stone O-iii, acceptance row 6.
;;
;; THE CLAIM: migrating the 6 `:wat::vector::*` verbs from BINDING (hand-written AST shell,
;; `concat` also hand-written value twin) to ALGEBRA (one declaration, macro-generated doors)
;; changes NOTHING about the direct call's observable behaviour — value AND error text,
;; byte-identical before and after. This probe exercises the success path for all six PLUS
;; one error path (a type mismatch on `length`, and an arity mismatch via `:wat::eval-ast!` to
;; bypass the static checker, matching the shape of the sibling O-i probes).
;;
;; Run against the pre-migration tree and the post-migration tree; diff the two transcripts.

(:wat::core::defn :probe::outcome [r <- (:wat::core::Result :- [:wat::core::Value :wat::core::EvalError])]
  -> :wat::core::String
  (:wat::core::match r
    ((:wat::core::Ok v)  (:wat::string::concat "ok:" (:wat::edn::write v)))
    ((:wat::core::Err e) (:wat::string::concat "err:" (:wat::core::EvalError/message e)))))

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::let
    [_01 (:wat::kernel::println (:wat::string::concat "length        " (:wat::edn::write (:wat::vector::length (:wat::core::PersistentVector 1 2 3)))))
     _02 (:wat::kernel::println (:wat::string::concat "empty? true   " (:wat::edn::write (:wat::vector::empty? (:wat::core::PersistentVector)))))
     _03 (:wat::kernel::println (:wat::string::concat "empty? false  " (:wat::edn::write (:wat::vector::empty? (:wat::core::PersistentVector 1)))))
     _04 (:wat::kernel::println (:wat::string::concat "contains? true  " (:wat::edn::write (:wat::vector::contains? (:wat::core::PersistentVector 1 2 3) 2))))
     _05 (:wat::kernel::println (:wat::string::concat "contains? false " (:wat::edn::write (:wat::vector::contains? (:wat::core::PersistentVector 1 2 3) 9))))
     _06 (:wat::kernel::println (:wat::string::concat "get in-range  " (:wat::edn::write (:wat::vector::get (:wat::core::PersistentVector 1 2 3) 0))))
     _07 (:wat::kernel::println (:wat::string::concat "get oob       " (:wat::edn::write (:wat::vector::get (:wat::core::PersistentVector 1 2 3) 9))))
     _08 (:wat::kernel::println (:wat::string::concat "conj          " (:wat::edn::write (:wat::vector::length (:wat::vector::conj (:wat::core::PersistentVector) 1)))))
     _09 (:wat::kernel::println (:wat::string::concat "concat        " (:wat::edn::write (:wat::vector::length (:wat::vector::concat (:wat::core::PersistentVector 1) (:wat::core::PersistentVector 2))))))

     ;; error path — type mismatch, direct call, real span (bypasses eval-ast! entirely).
     _10 (:wat::kernel::println (:wat::string::concat "length type-mismatch: " (:probe::outcome (:wat::eval-ast! (:wat::core::quote (:wat::vector::length 5))))))

     ;; error path — arity mismatch, via eval-ast! (the checker would refuse this statically).
     _11 (:wat::kernel::println (:wat::string::concat "length arity-mismatch: " (:probe::outcome (:wat::eval-ast! (:wat::core::quote (:wat::vector::length (:wat::core::PersistentVector 1) (:wat::core::PersistentVector 2)))))))
     _12 (:wat::kernel::println (:wat::string::concat "concat arity-mismatch: " (:probe::outcome (:wat::eval-ast! (:wat::core::quote (:wat::vector::concat (:wat::core::PersistentVector 1)))))))
     _13 (:wat::kernel::println (:wat::string::concat "conj arity-mismatch: "   (:probe::outcome (:wat::eval-ast! (:wat::core::quote (:wat::vector::conj (:wat::core::PersistentVector)))))))
     _14 (:wat::kernel::println (:wat::string::concat "get arity-mismatch: "    (:probe::outcome (:wat::eval-ast! (:wat::core::quote (:wat::vector::get (:wat::core::PersistentVector 1)))))))
     _15 (:wat::kernel::println (:wat::string::concat "contains? arity-mismatch: " (:probe::outcome (:wat::eval-ast! (:wat::core::quote (:wat::vector::contains? (:wat::core::PersistentVector 1)))))))
     _16 (:wat::kernel::println (:wat::string::concat "empty? arity-mismatch: " (:probe::outcome (:wat::eval-ast! (:wat::core::quote (:wat::vector::empty? (:wat::core::PersistentVector 1) (:wat::core::PersistentVector 2)))))))]
    nil))
