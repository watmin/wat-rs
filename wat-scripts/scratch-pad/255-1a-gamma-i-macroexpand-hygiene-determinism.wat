;; arc 255 Stone 1a-gamma-i — is `:wat::core::macroexpand-1` DETERMINISTIC in the
;; literal "same input -> identical output value" sense the axis requires, or does
;; `walk_template`'s hygiene scope-tagging (`fresh_scope()`, a process-global
;; monotonic counter, `src/macros/expand.rs:942`) mean two calls on the SAME quoted
;; macro-call form return two STRUCTURALLY DIFFERENT ASTs (different scope ids on
;; the template-introduced `tmp` binder)?
;;
;; `:probe::twice`'s body introduces exactly one template-local identifier (`tmp`)
;; that `walk_template` tags with `macro_scope` (arc 255 hygiene, `add_scope`).
;; `x` itself is spliced from the caller's argument and keeps ITS OWN scope
;; (unaffected either way for this probe, since we splice the identical literal `5`
;; both times).
;;
;; Scratch, per holon/CLAUDE.md's `.wat` scratch convention (not the ephemeral
;; session tmp).

(:wat::core::defmacro :probe::twice [x <- :wat::WatAST] -> :wat::WatAST
  `(:wat::core::let [tmp ~x] (:wat::i64::+ tmp tmp)))

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::let
    [form1 (:wat::core::macroexpand-1 (:wat::core::quote (:probe::twice 5)))
     form2 (:wat::core::macroexpand-1 (:wat::core::quote (:probe::twice 5)))]
    (:wat::core::do
      (:wat::kernel::println "form1 (write-forms):")
      (:wat::kernel::println (:wat::core::write-forms form1))
      (:wat::kernel::println "form2 (write-forms):")
      (:wat::kernel::println (:wat::core::write-forms form2))
      (:wat::kernel::println "form1 = form2 ?  (expect true if deterministic)")
      (:wat::kernel::println (:wat::core::= form1 form2)))))
