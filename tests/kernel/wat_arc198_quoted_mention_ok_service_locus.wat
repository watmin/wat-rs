;; The SECOND witness — independent of `:user::main`, and one body wider.
;; `:user::spawn::service-locus` is quoted by the same nine `…::service-forms`
;; templates PLUS `:wat::spawn::ProcessOpts/launch`: 10 stdlib bodies reddened by
;; one user metadata-map. Must load clean.
(:wat::core::defn :user::spawn::service-locus {:restricted-to [:my::]} [] -> :wat::core::i64 1)

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::kernel::println "hi"))
