;; Scratch probe — arc 255 Stone P6-c-W2, acceptance row 2.
;;
;; `(:wat::runtime::metadata-of <verb>)` for the five W2 candidates, before and after
;; homing. Must report the REAL arity (0 / 2 / 1 / 0 / 0) — not -1 (unregistered) and
;; not variadic (a fictional arity for a verb declaring `&[WatAST]` only to reject).
;; Direct FQDN-keyword call sites, not routed through a helper fn (mirrors
;; `255-p6c-w1-config-metadata.wat` / `255-stone-p2-intrinsic-untouched.wat`).

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::do
    (:wat::core::match (:wat::runtime::metadata-of :wat::stream::empty)
      ((:wat::core::Some hm) (:wat::kernel::println (:wat::string::concat "stream::empty :arity= " (:wat::edn::write (:wat::hashmap::get hm :arity)))))
      (:None (:wat::kernel::println "stream::empty :arity= NONE")))
    (:wat::core::match (:wat::runtime::metadata-of :wat::stream::cons)
      ((:wat::core::Some hm) (:wat::kernel::println (:wat::string::concat "stream::cons :arity= " (:wat::edn::write (:wat::hashmap::get hm :arity)))))
      (:None (:wat::kernel::println "stream::cons :arity= NONE")))
    (:wat::core::match (:wat::runtime::metadata-of :wat::stream::next)
      ((:wat::core::Some hm) (:wat::kernel::println (:wat::string::concat "stream::next :arity= " (:wat::edn::write (:wat::hashmap::get hm :arity)))))
      (:None (:wat::kernel::println "stream::next :arity= NONE")))
    (:wat::core::match (:wat::runtime::metadata-of :wat::program::env)
      ((:wat::core::Some hm) (:wat::kernel::println (:wat::string::concat "program::env :arity= " (:wat::edn::write (:wat::hashmap::get hm :arity)))))
      (:None (:wat::kernel::println "program::env :arity= NONE")))
    (:wat::core::match (:wat::runtime::metadata-of :wat::stdlib::sources)
      ((:wat::core::Some hm) (:wat::kernel::println (:wat::string::concat "stdlib::sources :arity= " (:wat::edn::write (:wat::hashmap::get hm :arity)))))
      (:None (:wat::kernel::println "stdlib::sources :arity= NONE")))))
