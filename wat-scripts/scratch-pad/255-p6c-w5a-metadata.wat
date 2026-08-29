;; Scratch probe — arc 255 Stone P6-c-W5a, acceptance rows 2/3/4.
;;
;; `(:wat::runtime::metadata-of <verb>)` for the nine W5a candidates, AFTER homing. Must report
;; the REAL arity (1/1/1/1/1/1/2/2/3) — not -1 (unregistered) and not variadic (a fictional
;; arity for a verb that declared `&[WatAST]` only to reject via a hand-rolled length check).
;; BEFORE homing all nine were NOT in the `#[wat_intrinsic]` registry (hand-rolled giant-match
;; arms only), so `metadata-of` returned `:None` for all nine pre-image — confirmed structurally
;; (zero `#[wat_intrinsic]` attributes on any of the nine handlers at HEAD), not by re-running an
;; old binary (no `git stash` permitted). Direct FQDN-keyword call sites, mirrors
;; `255-p6c-w2-metadata.wat`.

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::do
    (:wat::core::match (:wat::runtime::metadata-of :wat::rete::pure?)
      ((:wat::core::Some hm) (:wat::kernel::println (:wat::string::concat "rete::pure? :arity= " (:wat::edn::write (:wat::hashmap::get hm :arity)))))
      (:None (:wat::kernel::println "rete::pure? :arity= NONE")))
    (:wat::core::match (:wat::runtime::metadata-of :wat::rete::deterministic?)
      ((:wat::core::Some hm) (:wat::kernel::println (:wat::string::concat "rete::deterministic? :arity= " (:wat::edn::write (:wat::hashmap::get hm :arity)))))
      (:None (:wat::kernel::println "rete::deterministic? :arity= NONE")))
    (:wat::core::match (:wat::runtime::metadata-of :wat::rete::total?)
      ((:wat::core::Some hm) (:wat::kernel::println (:wat::string::concat "rete::total? :arity= " (:wat::edn::write (:wat::hashmap::get hm :arity)))))
      (:None (:wat::kernel::println "rete::total? :arity= NONE")))
    (:wat::core::match (:wat::runtime::metadata-of :wat::rete::primitive?)
      ((:wat::core::Some hm) (:wat::kernel::println (:wat::string::concat "rete::primitive? :arity= " (:wat::edn::write (:wat::hashmap::get hm :arity)))))
      (:None (:wat::kernel::println "rete::primitive? :arity= NONE")))
    (:wat::core::match (:wat::runtime::metadata-of :wat::rete::vocabulary-admitted?)
      ((:wat::core::Some hm) (:wat::kernel::println (:wat::string::concat "rete::vocabulary-admitted? :arity= " (:wat::edn::write (:wat::hashmap::get hm :arity)))))
      (:None (:wat::kernel::println "rete::vocabulary-admitted? :arity= NONE")))
    (:wat::core::match (:wat::runtime::metadata-of :wat::rete::cond-has-deferred-constraint?)
      ((:wat::core::Some hm) (:wat::kernel::println (:wat::string::concat "rete::cond-has-deferred-constraint? :arity= " (:wat::edn::write (:wat::hashmap::get hm :arity)))))
      (:None (:wat::kernel::println "rete::cond-has-deferred-constraint? :arity= NONE")))
    (:wat::core::match (:wat::runtime::metadata-of :wat::rete::alpha-match)
      ((:wat::core::Some hm) (:wat::kernel::println (:wat::string::concat "rete::alpha-match :arity= " (:wat::edn::write (:wat::hashmap::get hm :arity)))))
      (:None (:wat::kernel::println "rete::alpha-match :arity= NONE")))
    (:wat::core::match (:wat::runtime::metadata-of :wat::rete::alpha-match-local)
      ((:wat::core::Some hm) (:wat::kernel::println (:wat::string::concat "rete::alpha-match-local :arity= " (:wat::edn::write (:wat::hashmap::get hm :arity)))))
      (:None (:wat::kernel::println "rete::alpha-match-local :arity= NONE")))
    (:wat::core::match (:wat::runtime::metadata-of :wat::rete::alpha-match-under)
      ((:wat::core::Some hm) (:wat::kernel::println (:wat::string::concat "rete::alpha-match-under :arity= " (:wat::edn::write (:wat::hashmap::get hm :arity)))))
      (:None (:wat::kernel::println "rete::alpha-match-under :arity= NONE")))))
