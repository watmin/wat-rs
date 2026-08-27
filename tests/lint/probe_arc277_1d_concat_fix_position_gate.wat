;; tests/lint/probe_arc277_1d_concat_fix_position_gate.wat — co-located fixture for the sibling probe
;; (.rs), slurped via startup_beside(file!()). A SourceFile with BOTH: a defmacro whose body builds a
;; name via a bare-symbol concat (expand-time → must become `interpolate`), and a defn whose body has a
;; bare-symbol concat (runtime → must stay `format`). The probe eval_in_frozen's (:t::fix) and asserts
;; the concat-fix picks the head by position. (The inner \"...\" are the wat source-string the lexer sees.)

(:wat::core::defn :t::fix [] -> :wat::core::String
  (:wat::lint::lint-fix-file
    (:wat::source::File :path "t.wat"
      :source "(:wat::core::defmacro :u::m [x <- :wat::WatAST] -> :wat::core::String (:wat::core::let [s (:wat::core::ast-name x) nm (:wat::string::concat s \"::Op\")] nm)) (:wat::core::defn :u::f [a <- :wat::core::String] -> :wat::core::String (:wat::string::concat \"x: \" a))")))
