;; tests/program/probe_arc259_program_cpu_count.wat — co-located fixture for probe_arc259_program_cpu_count.rs,
;; slurped via startup_beside(file!()).

;; compute: invoke the live cpu-count verb — no installed program env required.
(:wat::core::defn :probe::compute [] -> :wat::core::i64
  (:wat::program::cpu-count))

