;; tests/cli/probe_excursus003_strike_a_ga1_one_death_shape_main_thread.wat
;; GA1 (excursus 003 strike A) — an unhandled assertion on the main thread of a
;; top-level `wat` run. Driven by the .rs test through the real binary
;; (EDN-over-stdio: this file IS the whole program, given as the entry file).
(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::test::assert-eq 1 2))
