;; arc109-type-equal-acceptance.wat — Stone BRIEF-STONE-type-equal-the-missing-door.md,
;; acceptance for `:wat::core::type-equal?` (ordinary-code call site). Row 6 (macro-body
;; callability) is a separate file (arc109-type-equal-row6-macro.wat).
;;
;; ⛔ REWRITTEN 2026-09-09 (arc 255 stone ④ follow-on, builder's ruling). The original rows 1-3
;; compared the ANGLE-BRACKET keyword surface (`Peer<A,B>`) against the parametric-form surface
;; (`(Peer :- [A B])`). Arc 109 ("annihilate the angle bracket") retired that spelling at the
;; LEXER: `(:wat::core::keyword-node ":wat::kernel::Peer<A,B>")` is refused at its own door, so
;; this file TYPE-CHECKED and then died at RUNTIME on that wall — invisible to
;; `every_wat_scripts_file_loads`, which parses and type-checks but never runs `main`.
;;
;; ★ The builder's ruling: *"the turbofish comparisons are illogical — turbofish is annihilated,
;; it is not supported, IT HAS NO FORM."* A comparison against a thing with no form is not a
;; weaker test; it is not a test. Those three rows are DELETED, and their INTENT is re-expressed
;; below in the only spelling that survives.
;;
;; Row 1: equivalence  — `(Peer :- [A B])` against itself, built through two independent reads.
;; Row 2: nested       — `(Vector :- [(HashMap :- [K V])])` likewise.
;; Row 3: NEGATIVE CONTROL — `(Peer :- [A B])` vs `(Peer :- [B A])`, args swapped -> false.
;;        This is the original row 3's job, and it is what stops a verb that returns `true`
;;        unconditionally from passing rows 1 and 2.
;; Row 4: identity (keyword against itself -> true) and two unrelated types -> false.
;;
;; ⚠ `type-equal?`'s own doc (`src/intrinsic/reflect.rs`) still states its purpose as the gap
;; between those two surfaces, and it still carries an angle-bracket branch nothing can reach.
;; Its three registered `@example`s, and `type-params-used-in`'s two, are the same fossil and are
;; still sealed behind the `#[ignore]` on `verify_examples_reports_no_failures`
;; (`tests/reflection/probe_arc255_ivb2b_verify_examples.rs`). Rewriting THIS file does not
;; resolve that — it removes one artifact of it. The open ruling stands, recorded there.

;; `read-string` returns `:wat::core::ReadOutcome` (Forms|Malformed), not a bare `:wat::WatAST` —
;; unwrap the single top-level form the same way `wat/core.wat`'s own macros do (e.g. core.wat:1816).
(:wat::core::defn :user::form-of [src <- :wat::core::String] -> :wat::WatAST
  (:wat::core::match (:wat::core::read-string src)
    [:wat::core::ReadOutcome::Forms {:forms __forms} (:wat::core::first __forms)]
    [:wat::core::ReadOutcome::Malformed {:cause __cause} (:wat::kernel::assertion-failed! :message (:wat::core::Error/message __cause))]))

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::do
    (:wat::kernel::println "row1 (Peer :- [A B]) vs (Peer :- [A B]) [two independent reads]:")
    (:wat::kernel::println
      (:wat::core::show
        (:wat::core::type-equal?
          (:user::form-of "(:wat::kernel::Peer :- [A B])")
          (:user::form-of "(:wat::kernel::Peer :- [A B])"))))

    (:wat::kernel::println "row2 nested (Vector :- [(HashMap :- [K V])]):")
    (:wat::kernel::println
      (:wat::core::show
        (:wat::core::type-equal?
          (:user::form-of "(:wat::core::Vector :- [(:wat::core::HashMap :- [K V])])")
          (:user::form-of "(:wat::core::Vector :- [(:wat::core::HashMap :- [K V])])"))))

    (:wat::kernel::println "row3 NEGATIVE CONTROL (Peer :- [A B]) vs (Peer :- [B A]):")
    (:wat::kernel::println
      (:wat::core::show
        (:wat::core::type-equal?
          (:user::form-of "(:wat::kernel::Peer :- [A B])")
          (:user::form-of "(:wat::kernel::Peer :- [B A])"))))

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
