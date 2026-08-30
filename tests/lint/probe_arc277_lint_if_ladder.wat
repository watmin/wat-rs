;; tests/lint/probe_arc277_lint_if_ladder.wat — co-located fixture for the sibling probe (.rs),
;; slurped via startup_beside(file!()). A SourceFile whose body is a 3-deep nested-if-=-ladder over
;; one var `x` — a set membership in disguise. The probe eval_in_frozen's (:t::lint) and asserts >= 1
;; finding from the nested-if-=-ladder rule. (The inner \"...\" are the wat source-string the lexer sees.)

(:wat::core::defn :t::lint [] -> (:wat::core::Vector :- [:wat::lint::Finding])
  (:wat::lint::lint-source
    (:wat::core::Vector :- [:wat::source::File]
      (:wat::source::File :path "t.wat"
        :source "(:wat::core::defn :t::f [x <- :wat::core::String] -> :wat::core::bool (:wat::core::if (:wat::core::= x \"a\") true (:wat::core::if (:wat::core::= x \"b\") true (:wat::core::if (:wat::core::= x \"c\") true false))))"))))
