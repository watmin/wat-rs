;; Scratch probe — arc 255 Stone P6-c-W5c, acceptance rows 2/3/4.
;;
;; `(:wat::runtime::metadata-of <verb>)` for the four W5c candidates, AFTER homing. Must report
;; the REAL arity (1/1/5/2) — not -1 (unregistered) and not variadic. BEFORE homing all four were
;; NOT in the `#[wat_intrinsic]` registry (hand-rolled giant-match arms only), so `metadata-of`
;; returned `:None` for all four pre-image — confirmed against a real HEAD clone (`git clone
;; --local` into scratch, built, run), not a `git stash`. Direct FQDN-keyword call sites, mirrors
;; `255-p6c-w5b-metadata.wat`.

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::do
    (:wat::core::match (:wat::runtime::metadata-of :wat::rete::lower)
      ((:wat::core::Some hm) (:wat::kernel::println (:wat::string::concat "rete::lower :arity= " (:wat::edn::write (:wat::hashmap::get hm :arity)))))
      (:None (:wat::kernel::println "rete::lower :arity= NONE")))
    (:wat::core::match (:wat::runtime::metadata-of :wat::rete::collect-rules)
      ((:wat::core::Some hm) (:wat::kernel::println (:wat::string::concat "rete::collect-rules :arity= " (:wat::edn::write (:wat::hashmap::get hm :arity)))))
      (:None (:wat::kernel::println "rete::collect-rules :arity= NONE")))
    (:wat::core::match (:wat::runtime::metadata-of :wat::rete::step-payload)
      ((:wat::core::Some hm) (:wat::kernel::println (:wat::string::concat "rete::step-payload :arity= " (:wat::edn::write (:wat::hashmap::get hm :arity)))))
      (:None (:wat::kernel::println "rete::step-payload :arity= NONE")))
    (:wat::core::match (:wat::runtime::metadata-of :wat::rete::axis-violation)
      ((:wat::core::Some hm) (:wat::kernel::println (:wat::string::concat "rete::axis-violation :arity= " (:wat::edn::write (:wat::hashmap::get hm :arity)))))
      (:None (:wat::kernel::println "rete::axis-violation :arity= NONE")))))
