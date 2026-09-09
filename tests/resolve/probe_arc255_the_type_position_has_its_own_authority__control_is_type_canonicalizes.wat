;; ★ GUARD — the ONE DOOR canonicalizes, and `:wat::runtime::is-type?` inherits it.
;;
;; Stone ② routed `is-type?` and `normalize`'s binder head through one `TypeEnv::is_known_type`.
;; That door canonicalizes `:wat::type::X` -> `:wat::core::X` FIRST, because every store it
;; consults answers false for a `:wat::type::` spelling.
;;
;; So `is-type?` gained an answer it did not have: measured before the stone, all four of these
;; printed false. The two spellings of one type now agree, which is the point — but it is a
;; BEHAVIOUR CHANGE to a shipped verb and is pinned here rather than left to be discovered.
;;
;; Expected: true true true false
(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::do
    (:wat::kernel::println (:wat::runtime::is-type? :wat::type::Tuple))
    (:wat::kernel::println (:wat::runtime::is-type? :wat::type::i64))
    (:wat::kernel::println (:wat::runtime::is-type? :wat::core::Tuple))
    (:wat::kernel::println (:wat::runtime::is-type? :wat::type::Infer))))
