;; SCRATCH — the dot flip's derivation mechanic, proven on four names.
;;
;; The codemod may not PATTERN-MATCH a variant: `::PeerKind::thread` is a variant and
;; `::Record::def` is a surface method, and they are textually identical. The substrate
;; is ASKED instead, per `:wat::runtime::variant-parent-of`.
;;
;; `keyword::from-string` refuses a leading colon; the answer carries one.

(:wat::core::defn :user::ask [bare <- :wat::core::String] -> :wat::core::String
  (:wat::core::match
    (:wat::runtime::variant-parent-of (:wat::keyword::from-string bare))
    [:wat::core::Option.Some {:value parent}
      (:wat::string::concat ":" bare "  VARIANT of " (:wat::keyword::to-string parent))]
    [:wat::core::Option.None {}
      (:wat::string::concat ":" bare "  -- not a variant")]))

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::do
    (:wat::kernel::println (:user::ask "wat::core::Option::Some"))
    (:wat::kernel::println (:user::ask "wat::program::PeerKind::thread"))
    (:wat::kernel::println (:user::ask "wat::core::Record::def"))
    (:wat::kernel::println (:user::ask "wat::cache::Cache::GetRequest"))))
