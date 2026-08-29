;; Scratch probe — arc 255 Stone O, second defect. Drawn BEFORE the brief.
;;
;; ⚠ THIS PROGRAM IS EXPECTED TO DIE. That death IS the finding, and the two lines it
;; prints first are the control that makes the death mean something.
;;
;; THE CLAIM: the value door (`:wat::core::apply` -> `dispatch_substrate_impl` -> a
;; registered `value_handler`) performs NO arity check. Every value handler opens with
;; `vals.first().expect("arity-checked")` / `vals.get(1).expect("arity-checked")` — naming
;; a check that happens only on the OTHER door (the `#[wat_intrinsic]`-generated AST shim,
;; `crates/wat-macros/src/wat_intrinsic.rs:545`, which raises `ArityMismatch`). So a
;; wrong-arity `apply` PANICS the process where the identical wrong-arity direct call
;; returns a clean error.
;;
;; MEASURED at 9b25f3bbf, ./target/release/wat:
;;   (:wat::i64::+ 20)                     -> err ":wat::i64::+: expected 2 args, got 1"
;;   (apply :wat::i64::+ [20])             -> PANIC  src/runtime.rs:11605  "arity-checked"
;;   (apply :wat::vector::concat [one-pv]) -> PANIC  src/intrinsic/vector.rs:214
;; Censused: 25 unchecked-index sites across 5 intrinsic files, plus the shared
;; `arith_{i64,f64,bigint,rational}_*_inner` fns — and NO value handler checks `vals.len()`.
;; All 44 verbs carrying a value door are reachable panics.
;;
;; AFTER Stone O this program must PRINT its third line and EXIT 0: the third row's outcome
;; becomes an ArityMismatch error value, identical in kind to row 1's. Re-run it as the
;; acceptance instrument; do not rewrite it.

(:wat::core::defn :probe::outcome [r <- (:wat::core::Result :- [:wat::core::Value :wat::core::EvalError])]
  -> :wat::core::String
  (:wat::core::match r
    ((:wat::core::Ok v)  (:wat::string::concat "ok:" (:wat::edn::write v)))
    ((:wat::core::Err e) (:wat::string::concat "err:" (:wat::core::EvalError/message e)))))

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::let
    [;; row 1 — THE CONTROL. The AST door, wrong arity: a clean ArityMismatch.
     _01 (:wat::kernel::println
           (:wat::string::concat "AST-door  wrong arity: "
             (:probe::outcome (:wat::eval-ast! (:wat::core::quote (:wat::i64::+ 20))))))

     ;; row 2 — the AST door with RIGHT arity, so row 3 cannot be blamed on the verb.
     _02 (:wat::kernel::println
           (:wat::string::concat "AST-door  right arity: "
             (:probe::outcome (:wat::eval-ast! (:wat::core::quote (:wat::i64::+ 20 22))))))

     ;; row 3 — THE FINDING. Same verb, same wrong arity, through apply.
     ;; TODAY: the process dies here and rows below never print.
     ;; AFTER STONE O: prints `err::wat::i64::+: expected 2 args, got 1` — row 1's text.
     _03 (:wat::kernel::println
           (:wat::string::concat "value-door wrong arity: "
             (:probe::outcome (:wat::eval-ast! (:wat::core::quote
               (:wat::core::apply :wat::i64::+ (:wat::core::Vector :- [:wat::core::i64] 20)))))))]
    nil))
