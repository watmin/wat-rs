;; Programs-are-atoms hello-world (structural side). See
;; tests/cli/wat_cli.rs::programs_are_atoms_hello_world for the full proof
;; narrative: (:wat::core::quote ...) captures a println expression as a
;; :wat::WatAST without firing its side effects; Atom wraps it as a holon;
;; eval-ast! executes the captured program under constrained eval. Arc 170
;; migration: outer main uses canonical [] -> :nil signature; inner quoted
;; program uses (:wat::kernel::println "wat-atoms") — the println call is
;; the load-bearing expression captured as data and re-executed via
;; eval-ast!. No stdin required.
(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::let
    [program
       (:wat::core::quote
         (:wat::kernel::println "wat-atoms"))
     program-atom
       (:wat::holon::Atom (:wat::holon::to-holon program))]
    ;; arc 057 Story-2 recovery: program-atom is now a structural
    ;; HolonAST (the form lowered onto the algebra grid). to-watast
    ;; was the original reverse path (HolonAST → WatAST) but is no
    ;; longer available; use the original quoted WatAST directly.
    ;; eval-ast! returns (:Result :- [wat::holon::HolonAST EvalError]) per
    ;; the 2026-04-20 INSCRIPTION. Match both arms to preserve main's
    ;; declared return type of :(). Err arm is unreachable here
    ;; (the quoted program is well-formed and non-mutating).
    (:wat::core::match (:wat::eval-ast! program)
      ((Ok _) nil)
      ((Err _) nil))))
