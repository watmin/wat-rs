;; tests/macros/probe_arc279_format_unused_kwarg.wat — NEGATIVE fixture for
;; probe_arc279_format.rs (format_strict_unused_kwarg_is_macro_error).
;; Template uses {x} but :y kwarg is also provided (unused). Must fail at startup.
(:wat::core::defn :user::f [] -> :wat::core::String
  (:wat::core::format "{x}" :x "hello" :y "extra"))
