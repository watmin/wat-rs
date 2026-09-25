;; SPECIMEN — excursus 003 stone P. Loads a sibling file that has a lex error. The OUTER
;; :location must name THIS file's load-file! call; the nested :cause's own :location (stone O)
;; must name the LOADED file's real line/col — two different files, two different jobs.
;; ⛔ Do not "fix" the target — the target being unlexable is the point.
(:wat::load-file! "probe_ex003_diagnostic_locates_the_user__load_lex_error_target.wat.bad")
(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::kernel::println "ran"))
