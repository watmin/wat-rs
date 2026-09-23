;; SPECIMEN — the-little-wat F-114. A `Pure` aggregate may hold only pure fields, and a function
;; type is not pure, so the containment rule (arc 293.W) refuses this declaration. Before
;; excursus 003 stone B the refusal's :location named WAT-RS'S OWN src/check.rs; the walk runs
;; AFTER registration, over a `TypeEnv` that had thrown the declaration's span away.
;; ⛔ Do not "fix" this record — the record being illegal is the point; the defect was WHERE the
;; diagnostic said the illegality was.
(:wat::core::defrecord :t::R [f <- [:wat::core::i64 :-> :wat::core::i64]])
(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::kernel::println "unreachable — startup refuses first"))
