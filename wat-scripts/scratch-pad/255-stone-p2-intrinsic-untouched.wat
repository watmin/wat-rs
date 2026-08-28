;; Scratch probe — arc 255 Stone P2, acceptance row 1.
;;
;; AN INTRINSIC IS UNTOUCHED. `show-source` and `metadata-of` on a registered
;; intrinsic (`:wat::i64::+`, and `:wat::map::length`, migrated by O-iv-b) must be
;; byte-identical before and after this stone's fold/reflect.rs changes.
;; (Direct FQDN-keyword call site, not routed through a helper fn — `show-source`/
;; `metadata-of` read the unevaluated WatAST arg, and a helper's typed param would
;; force the FQDN keyword to resolve to its function type before it got there.)

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::do
    (:wat::kernel::println (:wat::string::concat "i64+ show-source= " (:wat::core::show-source :wat::i64::+)))
    (:wat::core::match (:wat::runtime::metadata-of :wat::i64::+)
      ((:wat::core::Some hm) (:wat::kernel::println (:wat::string::concat "i64+ :arity= " (:wat::edn::write (:wat::hashmap::get hm :arity)))))
      (:None (:wat::kernel::println "i64+ :arity= NONE")))
    (:wat::kernel::println (:wat::string::concat "map-length show-source= " (:wat::core::show-source :wat::map::length)))
    (:wat::core::match (:wat::runtime::metadata-of :wat::map::length)
      ((:wat::core::Some hm) (:wat::kernel::println (:wat::string::concat "map-length :arity= " (:wat::edn::write (:wat::hashmap::get hm :arity)))))
      (:None (:wat::kernel::println "map-length :arity= NONE")))))
