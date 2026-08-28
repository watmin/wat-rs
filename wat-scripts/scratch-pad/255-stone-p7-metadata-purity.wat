;; Scratch probe — arc 255 Stone P7, acceptance row 8.
;; (:wat::runtime::metadata-of <verb>) for uuid::v4, time::now, and one reset-*!,
;; run BEFORE and AFTER the migration. Purity/Determinism must read the same both times —
;; the migration changes the handler's PARAM LIST, not its declared metadata.

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::do
    (:wat::kernel::println "== metadata-of uuid::v4 ==")
    (:wat::kernel::pprintln (:wat::runtime::metadata-of :wat::uuid::v4))
    (:wat::kernel::println "== metadata-of time::now ==")
    (:wat::kernel::pprintln (:wat::runtime::metadata-of :wat::time::now))
    (:wat::kernel::println "== metadata-of kernel::reset-sigusr1! ==")
    (:wat::kernel::pprintln (:wat::runtime::metadata-of :wat::kernel::reset-sigusr1!))))
