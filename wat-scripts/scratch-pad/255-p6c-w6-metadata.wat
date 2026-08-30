;; Scratch probe — arc 255 Stone P6-c-W6, acceptance rows 0/3.
;;
;; `(:wat::runtime::metadata-of <verb>)` for the seven W6 candidates, AFTER homing. Must report
;; the REAL arity (1/1/2/1/1/1/2 — length/empty?/nth/last/rest/reverse/range) — not -1
;; (unregistered) and not variadic. BEFORE homing none of the seven were in the
;; `#[wat_intrinsic]` registry (hand-rolled giant-match arms only in `runtime.rs`/
;; `collection/eval.rs`/`collection/transform.rs`), so `metadata-of` returned `:None` for the
;; pre-image of all seven — confirmed against a real HEAD clone (`git clone --local` into
;; scratch, built, run), not a `git stash`. Direct FQDN-keyword call sites, mirrors
;; `255-p6c-w5c-metadata.wat`.

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::do
    (:wat::core::match (:wat::runtime::metadata-of :wat::core::length)
      ((:wat::core::Some hm) (:wat::kernel::println (:wat::string::concat "core::length :arity= " (:wat::edn::write (:wat::hashmap::get hm :arity)))))
      (:None (:wat::kernel::println "core::length :arity= NONE")))
    (:wat::core::match (:wat::runtime::metadata-of :wat::core::empty?)
      ((:wat::core::Some hm) (:wat::kernel::println (:wat::string::concat "core::empty? :arity= " (:wat::edn::write (:wat::hashmap::get hm :arity)))))
      (:None (:wat::kernel::println "core::empty? :arity= NONE")))
    (:wat::core::match (:wat::runtime::metadata-of :wat::core::nth)
      ((:wat::core::Some hm) (:wat::kernel::println (:wat::string::concat "core::nth :arity= " (:wat::edn::write (:wat::hashmap::get hm :arity)))))
      (:None (:wat::kernel::println "core::nth :arity= NONE")))
    (:wat::core::match (:wat::runtime::metadata-of :wat::core::last)
      ((:wat::core::Some hm) (:wat::kernel::println (:wat::string::concat "core::last :arity= " (:wat::edn::write (:wat::hashmap::get hm :arity)))))
      (:None (:wat::kernel::println "core::last :arity= NONE")))
    (:wat::core::match (:wat::runtime::metadata-of :wat::core::rest)
      ((:wat::core::Some hm) (:wat::kernel::println (:wat::string::concat "core::rest :arity= " (:wat::edn::write (:wat::hashmap::get hm :arity)))))
      (:None (:wat::kernel::println "core::rest :arity= NONE")))
    (:wat::core::match (:wat::runtime::metadata-of :wat::core::reverse)
      ((:wat::core::Some hm) (:wat::kernel::println (:wat::string::concat "core::reverse :arity= " (:wat::edn::write (:wat::hashmap::get hm :arity)))))
      (:None (:wat::kernel::println "core::reverse :arity= NONE")))
    (:wat::core::match (:wat::runtime::metadata-of :wat::core::range)
      ((:wat::core::Some hm) (:wat::kernel::println (:wat::string::concat "core::range :arity= " (:wat::edn::write (:wat::hashmap::get hm :arity)))))
      (:None (:wat::kernel::println "core::range :arity= NONE")))))
