;; Scratch probe — arc 255 Stone P2, acceptance row 0.
;;
;; THE TWO LIES, BEFORE AND AFTER. For both `:wat::core::if` and `:wat::core::let`, print:
;;   (:wat::core::show-source <form>)
;;   (:wat::runtime::metadata-of <form>)  — the :arity field
;;
;; BEFORE (src/intrinsic/mod.rs's special-form fold hardcodes `source: ""` and
;; `arity: Arity::Variadic` for every Kind::SpecialForm):
;;   if    show-source -> ""            (the empty-string lie)
;;   if    :arity      -> -1            (the wrong-number lie — `if` declares 3 @args)
;;   let   show-source -> ""
;;   let   :arity      -> -1            (CORRECT for `let` — @syntax, zero @args, genuinely variadic)
;;
;; AFTER (expected):
;;   if    show-source -> ";; :wat::core::if — substrate primitive (no source available in this context)"
;;   if    :arity      -> 3
;;   let   show-source -> ";; :wat::core::let — substrate primitive (no source available in this context)"
;;   let   :arity      -> -1            (STILL variadic — @syntax, not @arg; this must NOT change)

(:wat::core::defn :user::report [form <- :wat::core::keyword] -> :wat::core::nil
  (:wat::core::let
    [src (:wat::core::show-source form)
     m   (:wat::runtime::metadata-of form)]
    (:wat::core::do
      (:wat::kernel::println (:wat::string::concat (:wat::edn::write form) "  show-source= " src))
      (:wat::core::match m
        ((:wat::core::Some hm)
          (:wat::kernel::println
            (:wat::string::concat (:wat::edn::write form) "  :arity= " (:wat::edn::write (:wat::hashmap::get hm :arity)))))
        (:None (:wat::kernel::println (:wat::string::concat (:wat::edn::write form) "  :arity= NONE")))))))

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::do
    (:user::report :wat::core::if)
    (:user::report :wat::core::let)))
