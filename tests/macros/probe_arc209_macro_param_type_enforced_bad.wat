;; tests/macros/probe_arc209_macro_param_type_enforced_bad.wat — NEGATIVE fixture for
;; probe_arc209_macro_param_type_enforced.rs, loaded via startup_from_file (must fail).
;;
;; A macro whose param claims :wat::core::i64 — a lie: a macro param is always a form.
;; Arc 251.5 / 209: macro-def now REJECTS a lying <- :i64 at definition time.
(:wat::core::defmacro :user::bad [x <- :wat::core::i64] -> :wat::WatAST x)
(:wat::core::defn :user::main [] -> :wat::core::nil nil)
