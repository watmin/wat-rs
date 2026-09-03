;; arc 255 Stone 1a-epsilon probe — does :wat::config::set-redef!'s eval arm actually
;; evaluate its argument, or ignore it entirely (as runtime.rs:2120's comment implies)?
;; If the arg is truly never evaluated, the println inside the `do` below never fires.
(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::let
    [q (:wat::config::set-redef!
         (:wat::core::do (:wat::kernel::println "SHOULD-NOT-PRINT-IF-ARG-UNEVALUATED") true))]
    (:wat::kernel::println (:wat::string::concat "q result => " (:wat::core::show q)))))
