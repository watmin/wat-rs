;; tests/macros/probe_arc279_format_missing_kwarg.wat — NEGATIVE fixture for
;; probe_arc279_format.rs (format_strict_missing_kwarg_is_macro_error).
;; Template references {y} but no :y kwarg is given. Must fail at startup.
(:wat::core::defn :user::f [] -> :wat::core::String
  (:wat::core::format "{x} {y}" :x "hello"))
