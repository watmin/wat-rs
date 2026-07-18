;; tests/resolve/probe_arc283_source_file_lift.wat
;; just-eval fixture for probe_arc283_source_file_lift.rs — constructs a
;; :wat::source::File and reads its path back through the accessor.
(:wat::core::defn :user::compute [] -> :wat::core::String
  (:wat::source::File/path (:wat::source::File' "t.wat" "(:t::f)")))
