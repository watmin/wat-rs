;; Board specimen — the-little-wat F-090: rational arithmetic COLLAPSES to a bigint that the
;; declared return type did not announce, and `rational/to-f64` then dies on it.
;;
;; SHAPE: LENIENT — the checker ACCEPTS, the runtime KILLS.
;; ⛔ Do NOT "fix" this file. `1/2 + 1/2` collapsing to `1` is the defect under measurement.
(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::kernel::println (:wat::edn::write (:wat::rational::to-f64 (:wat::rational::+ 1/2 1/2)))))
