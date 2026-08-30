;; Scratch probe — arc 255 Stone P6-c-W1, acceptance row 2.
;;
;; `(:wat::runtime::metadata-of :wat::config::<verb>)` for all four config readers,
;; before and after homing them into the `#[wat_intrinsic]` registry. Must report
;; arity 0 both times — not -1 (unregistered) and not variadic (a fictional arity
;; for a verb that is actually nullary). Direct FQDN-keyword call sites, not routed
;; through a helper fn — `metadata-of` reads the unevaluated WatAST arg, and a
;; helper's typed param would force the FQDN keyword to resolve to its function
;; type before it got there (mirrors `255-stone-p2-intrinsic-untouched.wat`).

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::do
    (:wat::core::match (:wat::runtime::metadata-of :wat::config::dim-count)
      ((:wat::core::Some hm) (:wat::kernel::println (:wat::string::concat "dim-count :arity= " (:wat::edn::write (:wat::hashmap::get hm :arity)))))
      (:None (:wat::kernel::println "dim-count :arity= NONE")))
    (:wat::core::match (:wat::runtime::metadata-of :wat::config::dim-capacity)
      ((:wat::core::Some hm) (:wat::kernel::println (:wat::string::concat "dim-capacity :arity= " (:wat::edn::write (:wat::hashmap::get hm :arity)))))
      (:None (:wat::kernel::println "dim-capacity :arity= NONE")))
    (:wat::core::match (:wat::runtime::metadata-of :wat::config::global-seed)
      ((:wat::core::Some hm) (:wat::kernel::println (:wat::string::concat "global-seed :arity= " (:wat::edn::write (:wat::hashmap::get hm :arity)))))
      (:None (:wat::kernel::println "global-seed :arity= NONE")))
    (:wat::core::match (:wat::runtime::metadata-of :wat::config::noise-floor)
      ((:wat::core::Some hm) (:wat::kernel::println (:wat::string::concat "noise-floor :arity= " (:wat::edn::write (:wat::hashmap::get hm :arity)))))
      (:None (:wat::kernel::println "noise-floor :arity= NONE")))))
