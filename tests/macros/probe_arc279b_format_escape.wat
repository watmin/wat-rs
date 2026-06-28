;; tests/macros/probe_arc279b_format_escape.wat — co-located fixture for
;; probe_arc279b_format_escape.rs, slurped via startup_beside(file!()).
;;
;; Three named probe functions, one per escape test.
;; probe-1: {{ and }} with no placeholder -> literal braces.
;; probe-2: doubled braces mixed with a real placeholder.
;; probe-3: trailing literal close brace after a placeholder.

(:wat::core::defn :user::probe-1 [] -> :wat::core::String
  (:wat::core::format "{{literal}}"))

(:wat::core::defn :user::probe-2 [] -> :wat::core::String
  (:wat::core::format "{{x}} = {name}" :name "v"))

(:wat::core::defn :user::probe-3 [] -> :wat::core::String
  (:wat::core::format "{name}}}" :name "v"))

