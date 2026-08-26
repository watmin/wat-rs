;; scratchpad/probe-mod-in-macro.wat — gate probe for the strike adding
;; i64::mod/rem/quot to the macro pure-total allow-list (src/macros/eval.rs
;; is_pure_total). `:wat::core::count` (already pure-total) gives an actual
;; i64 at expand time (the length of the variadic arg vector); the macro
;; body then calls (:wat::core::i64::mod n 2) directly on that i64 to branch
;; parity and splice a different literal into the expansion. Before the fix
;; this RefusedInMacro's at expand time on the i64::mod head; after the fix
;; it expands + runs green.
(:wat::core::defmacro :my::list-parity [& xs <- (:wat::core::Vector :- [:wat::WatAST])]
  -> :wat::WatAST
  (:wat::core::if (:wat::core::= (:wat::i64::mod (:wat::core::length xs) 2) 0)
    
    `"even"
    `"odd"))

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::do
    (:wat::kernel::println (:my::list-parity 1 2 3 4))
    (:wat::kernel::println (:my::list-parity 1 2 3))
    nil))
