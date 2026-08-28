;; Scratch probe (circumspicere recon, read-only) — does `metadata-of` on a
;; special form report a truthful `:arity`, or the hardcoded Variadic sentinel
;; (`src/intrinsic/mod.rs:410`) regardless of the form's actual documented shape?
;; `:wat::core::if` documents exactly 3 non-rest `@arg`s (cond/then/else) in
;; src/intrinsic/special/control_flow.rs — never variadic.
(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::let
    [m (:wat::runtime::metadata-of :wat::core::if)]
    (:wat::core::match m
      ((:wat::core::Some hm)
        (:wat::kernel::println
          (:wat::string::concat "if arity=" (:wat::edn::write (:wat::hashmap::get hm :arity)))))
      (:None (:wat::kernel::println "NONE")))))
