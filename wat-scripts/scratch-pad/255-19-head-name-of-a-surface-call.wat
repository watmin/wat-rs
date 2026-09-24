;; arc 255 Stone 255.19 — MEASUREMENT. defservice's `start`/`resume` macros and bracket's
;; `map`/`each` macros pick the per-locus impl by the `:locus` argument's HEAD NAME, a string
;; compare (`ast-name`). 255.19 moves `with-label` onto the surface, so that head becomes
;; `:wat::spawn::Locus/with-label`. What string does `ast-name` return for it — the colon
;; keyword, or the faithful `wat.spawn.Locus/with-label`? Prints both heads.
(:wat::core::defmacro :probe::head-of [form <- :wat::WatAST] -> :wat::WatAST
  (:wat::core::let [ch (:wat::core::ast->children form)]
    (:wat::core::ast-name (:wat::core::first ch))))
(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::do
    (:wat::kernel::println (:probe::head-of (:wat::spawn::Locus/with-label (:wat::spawn::process) 0)))
    (:wat::kernel::println (:probe::head-of (:wat::spawn::process)))))
