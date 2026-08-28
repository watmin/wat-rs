;; wat-scripts/scratch-pad/255-stone-o-iv-d-metadata.wat — arc 255 Stone O-iv-d,
;; acceptance row 5. `(metadata-of <fqdn>)` for the two nondeterministic verbs and one
;; mutating reset-*! verb this rider migrated, before and after, to prove @Purity /
;; @Determinism still read correctly post-migration.

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::do
    (:wat::kernel::println "── :wat::uuid::v4 ──")
    (:wat::kernel::pprintln (:wat::runtime::metadata-of :wat::uuid::v4))
    (:wat::kernel::println "── :wat::time::now ──")
    (:wat::kernel::pprintln (:wat::runtime::metadata-of :wat::time::now))
    (:wat::kernel::println "── :wat::kernel::reset-sigusr1! ──")
    (:wat::kernel::pprintln (:wat::runtime::metadata-of :wat::kernel::reset-sigusr1!))))
