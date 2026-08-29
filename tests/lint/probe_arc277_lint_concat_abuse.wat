;; tests/lint/probe_arc277_lint_concat_abuse.wat — co-located fixture for the sibling probe (.rs),
;; slurped via startup_beside(file!()). A SourceFile whose body is a string::concat chain mixing
;; string literals ("x: ", " of ") with non-literal values (a, b) — the hand-rolled template that
;; `format` cures. The probe eval_in_frozen's (:t::lint) and asserts >= 1 concat-abuse finding.
;; (The inner \"...\" are the wat source-string the lexer sees — a SourceFile body is wat-as-text.)

(:wat::core::defn :t::lint [] -> (:wat::core::Vector :- [:wat::lint::Finding])
  (:wat::lint::lint-source
    (:wat::core::Vector :- [:wat::source::File]
      (:wat::source::File :path "t.wat"
        :source "(:wat::core::defn :t::g [a <- :wat::core::String b <- :wat::core::String] -> :wat::core::String (:wat::string::concat \"x: \" a \" of \" b))"))))
