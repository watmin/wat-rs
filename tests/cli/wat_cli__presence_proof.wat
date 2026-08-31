;; Programs-are-atoms hello-world (vector side, with presence proof). See
;; tests/cli/wat_cli.rs::presence_proof_hello_world for the full proof
;; narrative: this extends the structural hello-world with a VECTOR-level
;; demonstration that MAP's bind / unbind self-inverse is observable through
;; presence measurement. Arc 170 migration: outer main uses canonical
;; [] -> :nil; inner quoted program uses (:wat::kernel::println "wat-atoms")
;; instead of the retired IOReader/IOWriter stdin-echo path. Presence proof
;; prints "absent"/"present" via println (EDN-encoded Strings). Observable
;; stdout: "absent"\n"present"\n"wat-atoms"\n.
(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::let
    [program
       (:wat::core::quote
         (:wat::kernel::println "wat-atoms"))
     program-atom
       (:wat::holon::Atom (:wat::holon::to-holon program))
     key-atom
       (:wat::holon::Atom (:wat::holon::to-holon "hello-world"))

     ;; Compose: program-atom bound under key-atom.
     bound
       (:wat::holon::Bind key-atom program-atom)

     ;; Substrate proof #1: program-atom's signal is GONE from bound.
     ;; Arc 037 slice 3: presence? does the honest per-d threshold
     ;; comparison internally. absent = not present.
     _
       (:wat::kernel::println
         (:wat::core::if
           (:wat::holon::presence? program-atom bound)
           "present"
           "absent"))

     ;; Self-inverse: bind(bind(k, p), k) recovers p at the vector level.
     recovered
       (:wat::holon::Bind bound key-atom)

     ;; Substrate proof #2: program-atom's signal is BACK in recovered.
     _
       (:wat::kernel::println
         (:wat::core::if
           (:wat::holon::presence? program-atom recovered)
           "present"
           "absent"))

]
    ;; arc 057 Story-2 recovery: the presence measurements above proved
    ;; the vector dynamics (absent/present). to-watast (HolonAST → WatAST)
    ;; is no longer available; run the original quoted WatAST directly.
    ;; eval-ast! returns (:Result :- [wat::holon::HolonAST EvalError]) per
    ;; the 2026-04-20 INSCRIPTION.
    (:wat::core::match (:wat::eval-ast! program)
      ((:wat::core::Ok _) nil)
      ((:wat::core::Err _) nil))))
