;; Scratch probe — arc 255 Stone P6-c-W5b, acceptance rows 2/3/4.
;;
;; `(:wat::runtime::metadata-of <verb>)` for the six W5b candidates, AFTER homing. Must report
;; the REAL arity (1/1/1/1/2/2) — not -1 (unregistered) and not variadic (a fictional arity for
;; a verb that declared `&[WatAST]` only to reject via a hand-rolled length check). BEFORE
;; homing all six were NOT in the `#[wat_intrinsic]` registry (hand-rolled giant-match arms
;; only), so `metadata-of` returned `:None` for all six pre-image — confirmed structurally
;; (zero `#[wat_intrinsic]` attributes on any of the six handlers at HEAD), not by re-running an
;; old binary (no `git stash` permitted). Direct FQDN-keyword call sites, mirrors
;; `255-p6c-w5a-metadata.wat`.

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::do
    (:wat::core::match (:wat::runtime::metadata-of :wat::rete::arm-session)
      ((:wat::core::Some hm) (:wat::kernel::println (:wat::string::concat "rete::arm-session :arity= " (:wat::edn::write (:wat::hashmap::get hm :arity)))))
      (:None (:wat::kernel::println "rete::arm-session :arity= NONE")))
    (:wat::core::match (:wat::runtime::metadata-of :wat::rete::release-session)
      ((:wat::core::Some hm) (:wat::kernel::println (:wat::string::concat "rete::release-session :arity= " (:wat::edn::write (:wat::hashmap::get hm :arity)))))
      (:None (:wat::kernel::println "rete::release-session :arity= NONE")))
    (:wat::core::match (:wat::runtime::metadata-of :wat::rete::export)
      ((:wat::core::Some hm) (:wat::kernel::println (:wat::string::concat "rete::export :arity= " (:wat::edn::write (:wat::hashmap::get hm :arity)))))
      (:None (:wat::kernel::println "rete::export :arity= NONE")))
    (:wat::core::match (:wat::runtime::metadata-of :wat::rete::import)
      ((:wat::core::Some hm) (:wat::kernel::println (:wat::string::concat "rete::import :arity= " (:wat::edn::write (:wat::hashmap::get hm :arity)))))
      (:None (:wat::kernel::println "rete::import :arity= NONE")))
    (:wat::core::match (:wat::runtime::metadata-of :wat::rete::eval-insert)
      ((:wat::core::Some hm) (:wat::kernel::println (:wat::string::concat "rete::eval-insert :arity= " (:wat::edn::write (:wat::hashmap::get hm :arity)))))
      (:None (:wat::kernel::println "rete::eval-insert :arity= NONE")))
    (:wat::core::match (:wat::runtime::metadata-of :wat::rete::eval-test)
      ((:wat::core::Some hm) (:wat::kernel::println (:wat::string::concat "rete::eval-test :arity= " (:wat::edn::write (:wat::hashmap::get hm :arity)))))
      (:None (:wat::kernel::println "rete::eval-test :arity= NONE")))))
