;; Scratch probe — arc 255 Stone O-iv-a, acceptance row 3 (the underlying-kind half).
;;
;; `255-stone-o-apply-has-three-broken-doors.wat`'s own rendering prints only "ERR" for
;; any error, so DOOR2's `(apply max-of [...])` row is byte-identical before and after
;; this stone even though its underlying EvalError kind changed from "unknown-function"
;; to something else. This probe reads the kind directly to show that change.

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::match (:wat::eval-ast! (:wat::core::quote
                        (:wat::core::apply :wat::f64::max-of
                          (:wat::core::Vector :- [:wat::core::f64] 3.0 9.0 41.0))))
    ((:wat::core::Ok _) (:wat::kernel::println "UNEXPECTED: ok"))
    ((:wat::core::Err e)
      (:wat::kernel::println (:wat::string::concat "DOOR2 max-of kind=" (:wat::core::EvalError/kind e))))))
