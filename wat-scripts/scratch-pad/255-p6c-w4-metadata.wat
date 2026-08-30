;; Scratch probe — arc 255 Stone P6-c-W4, acceptance row 3.
;;
;; `(:wat::runtime::metadata-of <verb>)` self-reporting arity for all three W4 verbs,
;; before and after homing. Before homing none of the three is a registered Rust
;; intrinsic (hand-rolled match arms only) so every call here reports NONE regardless
;; of which of the three verbs is asked about — including `metadata-of` asked about
;; itself. After homing all three report `Some` with `:arity 1`.

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::do
    (:wat::core::match (:wat::runtime::metadata-of :wat::runtime::metadata-of)
      ((:wat::core::Some hm) (:wat::kernel::println (:wat::string::concat "metadata-of :arity= " (:wat::edn::write (:wat::hashmap::get hm :arity)))))
      (:None (:wat::kernel::println "metadata-of :arity= NONE")))
    (:wat::core::match (:wat::runtime::metadata-of :wat::runtime::field-names-of)
      ((:wat::core::Some hm) (:wat::kernel::println (:wat::string::concat "field-names-of :arity= " (:wat::edn::write (:wat::hashmap::get hm :arity)))))
      (:None (:wat::kernel::println "field-names-of :arity= NONE")))
    (:wat::core::match (:wat::runtime::metadata-of :wat::runtime::field-types-of)
      ((:wat::core::Some hm) (:wat::kernel::println (:wat::string::concat "field-types-of :arity= " (:wat::edn::write (:wat::hashmap::get hm :arity)))))
      (:None (:wat::kernel::println "field-types-of :arity= NONE")))))
