;; arc109-type-equal-acceptance.wat — Stone BRIEF-STONE-type-equal-the-missing-door.md,
;; acceptance rows 1-4 (ordinary-code call site). Row 6 (macro-body callability) is a separate
;; file (arc109-type-equal-row6-macro.wat) since it needs a real `defmacro` + `macroexpand`.
;;
;; Row 1: `Peer<A,B>` (keyword surface) vs `(Peer :- [A B])` (parametric-form surface) -> true.
;; Row 2: nested `Vector<HashMap<K,V>>` vs the equivalent form -> true.
;; Row 3: NEGATIVE CONTROL — `Peer<A,B>` vs `Peer<B,A>` (args swapped) -> false. Without this, a
;;        verb that returned `true` unconditionally would still pass rows 1 and 2.
;; Row 4: identity (keyword against itself -> true) and two unrelated types -> false.

;; `read-string` returns `:wat::core::ReadOutcome` (Forms|Malformed), not a bare `:wat::WatAST` —
;; unwrap the single top-level form the same way `wat/core.wat`'s own macros do (e.g. core.wat:1816).
(:wat::core::defn :user::form-of [src <- :wat::core::String] -> :wat::WatAST
  (:wat::core::match (:wat::core::read-string src)
    ((:wat::core::ReadOutcome::Forms __forms) (:wat::core::first __forms))
    ((:wat::core::ReadOutcome::Malformed __cause) (:wat::kernel::panic! "form-of: malformed source"))))

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::do
    (:wat::kernel::println "row1 Peer<A,B> vs (Peer :- [A B]):")
    (:wat::kernel::println
      (:wat::core::show
        (:wat::core::type-equal?
          (:wat::core::keyword-node ":wat::kernel::Peer<A,B>")
          (:user::form-of "(:wat::kernel::Peer :- [A B])"))))

    (:wat::kernel::println "row2 nested Vector<HashMap<K,V>> vs form:")
    (:wat::kernel::println
      (:wat::core::show
        (:wat::core::type-equal?
          (:wat::core::keyword-node ":wat::core::Vector<wat::core::HashMap<K,V>>")
          (:user::form-of "(:wat::core::Vector :- [(:wat::core::HashMap :- [K V])])"))))

    (:wat::kernel::println "row3 NEGATIVE CONTROL Peer<A,B> vs Peer<B,A>:")
    (:wat::kernel::println
      (:wat::core::show
        (:wat::core::type-equal?
          (:wat::core::keyword-node ":wat::kernel::Peer<A,B>")
          (:wat::core::keyword-node ":wat::kernel::Peer<B,A>"))))

    (:wat::kernel::println "row4a identity, i64 vs i64:")
    (:wat::kernel::println
      (:wat::core::show
        (:wat::core::type-equal?
          (:wat::core::keyword-node ":wat::core::i64")
          (:wat::core::keyword-node ":wat::core::i64"))))

    (:wat::kernel::println "row4b unrelated, i64 vs String:")
    (:wat::kernel::println
      (:wat::core::show
        (:wat::core::type-equal?
          (:wat::core::keyword-node ":wat::core::i64")
          (:wat::core::keyword-node ":wat::core::String"))))
    nil))
