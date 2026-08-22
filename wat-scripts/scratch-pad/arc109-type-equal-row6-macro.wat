;; arc109-type-equal-row6-macro.wat — Stone BRIEF-STONE-type-equal-the-missing-door.md,
;; ★ row 6: `type-equal?` must be CALLABLE FROM A MACRO BODY. This is the row the verb exists
;; for — F5 (`src/macros/eval.rs`) is a default-deny admission list checked at macro DEFINITION
;; time, so if `:wat::core::type-equal?` is missing from it, loading THIS FILE fails before
;; `:probe::same-shape?` is ever expanded — a definition-time failure that looks like the
;; stdlib breaking, not like a wrong answer. Getting to the printed output below is itself the
;; positive result for row 6.
;;
;; The macro body calls `type-equal?` directly (not inside emitted/quoted code) to compare the
;; two DECLARED type nodes it was handed, and emits a different literal depending on the answer
;; — i.e., the comparison happens AT EXPAND TIME, in the macro's own Rust-reachable body.
(:wat::core::defmacro :probe::same-shape? [a <- :wat::WatAST b <- :wat::WatAST] -> :wat::WatAST
  (:wat::core::if (:wat::core::type-equal? a b)
    (:wat::core::keyword-node ":same")
    (:wat::core::keyword-node ":different")))

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::do
    (:wat::kernel::println "macroexpand (:probe::same-shape? Peer<A,B> (Peer :- [A B])) — same:")
    (:wat::kernel::println
      (:wat::core::write-forms
        (:wat::core::macroexpand
          (:wat::core::quote
            (:probe::same-shape? :wat::kernel::Peer<A,B> (:wat::kernel::Peer :- [A B]))))))

    (:wat::kernel::println "macroexpand (:probe::same-shape? Peer<A,B> Peer<B,A>) — different:")
    (:wat::kernel::println
      (:wat::core::write-forms
        (:wat::core::macroexpand
          (:wat::core::quote
            (:probe::same-shape? :wat::kernel::Peer<A,B> :wat::kernel::Peer<B,A>)))))
    nil))
