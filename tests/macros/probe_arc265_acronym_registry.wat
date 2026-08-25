;; tests/macros/probe_arc265_acronym_registry.wat — co-located fixture for
;; probe_arc265_acronym_registry.rs (CONV program), slurped via startup_beside(file!()).
;;
;; Namespace-scoped acronym registry: :my::aws has ACL declared.
(:wat::string::declare-acronyms :my::aws ["ACL"])
(:wat::string::declare-acronyms :other::ns [])

(:wat::core::defn :user::fwd [s <- :wat::core::String] -> :wat::core::String
  (:wat::string::pascal->kebab-in :my::aws s))
(:wat::core::defn :user::rev [s <- :wat::core::String] -> :wat::core::String
  (:wat::string::kebab->pascal-in :my::aws s))
(:wat::core::defn :user::rev-default [s <- :wat::core::String] -> :wat::core::String
  (:wat::string::kebab->pascal-in :other::ns s))
(:wat::core::defn :user::roundtrip [s <- :wat::core::String] -> :wat::core::String
  (:wat::string::kebab->pascal-in :my::aws (:wat::string::pascal->kebab-in :my::aws s)))
