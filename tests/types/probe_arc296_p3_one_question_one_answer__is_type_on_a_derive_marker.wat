;; HALF 2 SUBJECT — :wat::spawn::Spawned exists ONLY as a derive parent
;; (wat/spawn.wat:235-236, "typesub/derive axis; no methods"). The annotation
;; wall accepts it via subtype_edges; is-type? must not disagree.
(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::kernel::println (:wat::runtime::is-type? :wat::spawn::Spawned)))
