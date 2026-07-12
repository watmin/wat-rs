;; tests/lint/probe_arc277_1b_ladder_autofix.wat — co-located fixture for the sibling probe (.rs),
;; slurped via startup_beside(file!()). A SourceFile whose body is a 3-deep nested-if-=-ladder over one
;; var `x`; lint-fix-file must rewrite the ladder into a (contains? (HashSet …) x) call. The probe
;; eval_in_frozen's (:t::fix) and asserts the fixed source contains "contains?"/"HashSet" and no longer
;; the nested if-=-ladder. (The inner \"...\" are the wat source-string the lexer sees.)

(:wat::core::defn :t::fix [] -> :wat::core::String
  (:wat::lint::lint-fix-file
    (:wat::source::File :path "t.wat"
      :source "(:wat::core::defn :t::f [x <- :wat::core::String] -> :wat::core::bool (:wat::core::if (:wat::core::= x \"a\") true (:wat::core::if (:wat::core::= x \"b\") true (:wat::core::if (:wat::core::= x \"c\") true false))))")))
