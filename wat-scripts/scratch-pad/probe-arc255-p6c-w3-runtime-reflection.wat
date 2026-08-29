;; Scratch probe — arc 255 Stone P6-c-W3 (runtime reflection wave). Dumps `metadata-of` for
;; each of the ten homed `:wat::runtime::` verbs, so before/after homing can be diffed
;; byte-for-byte (acceptance rows 3/4).

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::do
    (:wat::kernel::println (:wat::core::show (:wat::runtime::metadata-of :wat::runtime::lookup-define)))
    (:wat::kernel::println (:wat::core::show (:wat::runtime::metadata-of :wat::runtime::signature-of-defn)))
    (:wat::kernel::println (:wat::core::show (:wat::runtime::metadata-of :wat::runtime::signature-of-fn)))
    (:wat::kernel::println (:wat::core::show (:wat::runtime::metadata-of :wat::runtime::return-type-of)))
    (:wat::kernel::println (:wat::core::show (:wat::runtime::metadata-of :wat::runtime::body-of)))
    (:wat::kernel::println (:wat::core::show (:wat::runtime::metadata-of :wat::runtime::rename-callable-name)))
    (:wat::kernel::println (:wat::core::show (:wat::runtime::metadata-of :wat::runtime::extract-arg-names)))
    (:wat::kernel::println (:wat::core::show (:wat::runtime::metadata-of :wat::runtime::extract-arg-types)))
    (:wat::kernel::println (:wat::core::show (:wat::runtime::metadata-of :wat::runtime::argv)))
    (:wat::kernel::println (:wat::core::show (:wat::runtime::metadata-of :wat::runtime::current-thread)))))
