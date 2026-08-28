;; tests/lint/probe_arc277_1c_concat_format_autofix.wat — co-located fixture for the sibling probe (.rs),
;; slurped via startup_beside(file!()). The two cases of the concat-abuse AUTO-FIX, enumerated as named
;; defns the probe calls by name (no inlined Rust strings, no format!): a BARE-SYMBOL concat that the fix
;; rewrites to a `format` call, and a COMPOUND-slot concat that stays report-only (no honest derivable
;; name). Each returns the fixed source String. (The inner \"...\" are the wat source-string the lexer sees.)

;; BARE-SYMBOL slots → auto-fix to a self-documenting format call.
(:wat::core::defn :t::fix-bare [] -> :wat::core::String
  (:wat::lint::lint-fix-file
    (:wat::source::File :path "t.wat"
      :source "(:wat::core::defn :u::g [a <- :wat::core::String b <- :wat::core::String] -> :wat::core::String (:wat::string::concat \"x: \" a \" y: \" b))")))

;; COMPOUND slot → NO auto-fix (report-only; naming is a judgment deferred to the RETE map).
(:wat::core::defn :t::fix-compound [] -> :wat::core::String
  (:wat::lint::lint-fix-file
    (:wat::source::File :path "t.wat"
      :source "(:wat::core::defn :u::h [n <- :wat::core::i64] -> :wat::core::String (:wat::string::concat \"n=\" (:wat::core::i64::to-string n)))")))
