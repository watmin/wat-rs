;; wb5-witness-thread-last-macroexpand.wat — 296 Wave B5 rider: isolated macroexpand of
;; `(->> 5 ())` (the thread-first sibling script's `main` aborts on its own -> witness before
;; reaching this one, since a WAT program's `main` is a single `do` and the first RuntimeError
;; halts it — so `->>`'s expansion is dumped standalone here). Pure introspection, never executed.
(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::kernel::println (:wat::core::write-forms
    (:wat::core::macroexpand (:wat::core::quote
      (:wat::core::->> 5 ()))))))
