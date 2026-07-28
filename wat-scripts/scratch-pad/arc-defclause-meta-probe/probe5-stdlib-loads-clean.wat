;; probe: trivial program forces the full stdlib (wat/core.wat, wat/sqlite.wat, wat/cache.wat,
;; wat/bracket.wat, wat/spawn.wat, ...) to load + type-check. Every EXISTING defclause in the
;; corpus has no metadata-map, so Part 1's optional-metadata parsing must be a complete no-op
;; for all of them — this must stay clean.
(:wat::core::defn :user::noop [] -> :wat::core::i64 (:wat::core::+ 1 1))
