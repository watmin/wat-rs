;; Scratch probe — arc 255 Stone O-i, acceptance row 1.
;;
;; THE CLAIM: `dispatch_substrate_impl`'s arity guard (src/runtime.rs:11561) is CENTRAL —
;; it guards every registered `value_handler`, not just the i64 arithmetic ones the other
;; scratch probe exercises. `:wat::vector::concat`'s value twin
;; (`eval_persistentvector_concat_home_value`, src/intrinsic/vector.rs:214) opens with
;; `vals.first().expect("arity-checked")` / `vals.get(1).expect("arity-checked")` — the
;; SAME pattern, in a DIFFERENT file, for a DIFFERENT type (PersistentVector, not i64).
;;
;; BEFORE Stone O-i: (:wat::core::apply :wat::vector::concat [<one PersistentVector>])
;; PANICS the process at src/intrinsic/vector.rs:214 ("arity-checked") — concat needs 2
;; args (to, from) and gets 1.
;;
;; AFTER Stone O-i: row 3 must PRINT and its outcome must be an ArityMismatch error,
;; identical in kind to row 1's (the AST-door control, same verb, same wrong arity).

(:wat::core::defn :probe::outcome [r <- (:wat::core::Result :wat::core::Value :wat::core::EvalError)]
  -> :wat::core::String
  (:wat::core::match r
    ((:wat::core::Ok v)  (:wat::string::concat "ok:" (:wat::edn::write v)))
    ((:wat::core::Err e) (:wat::string::concat "err:" (:wat::core::EvalError/message e)))))

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::let
    [;; row 1 — THE CONTROL. The AST door, wrong arity (1 instead of 2): a clean ArityMismatch.
     _01 (:wat::kernel::println
           (:wat::string::concat "AST-door  wrong arity: "
             (:probe::outcome (:wat::eval-ast! (:wat::core::quote
               (:wat::vector::concat (:wat::core::PersistentVector 1 2 3)))))))

     ;; row 2 — the AST door with RIGHT arity, so row 3 cannot be blamed on the verb.
     _02 (:wat::kernel::println
           (:wat::string::concat "AST-door  right arity: "
             (:probe::outcome (:wat::eval-ast! (:wat::core::quote
               (:wat::vector::concat (:wat::core::PersistentVector 1 2 3) (:wat::core::PersistentVector 4 5)))))))

     ;; row 3 — THE FINDING. Same verb, same wrong arity, through apply's value door.
     ;; BEFORE Stone O-i: the process dies here and nothing below prints.
     ;; AFTER Stone O-i: prints an ArityMismatch, matching row 1's KIND (op/expected/got).
     _03 (:wat::kernel::println
           (:wat::string::concat "value-door wrong arity: "
             (:probe::outcome (:wat::eval-ast! (:wat::core::quote
               (:wat::core::apply :wat::vector::concat
                 (:wat::core::Vector (:wat::core::PersistentVector :- [:wat::core::i64])
                   (:wat::core::PersistentVector 1 2 3))))))))]
    nil))
