;; Co-located fixture for tracked_wat_dir_is_stdlib_sources.rs — slurped via call_beside_value(file!()).
;; The baked stdlib's own path list, asked of the running substrate (`STDLIB_FILES` order).

(:wat::core::defn :user::stdlib-source-paths [] -> (:wat::core::Vector :- [:wat::core::String])
  (:wat::core::mapv
    (:wat::core::fn [pair <- (:wat::core::Vector :- [:wat::core::String])] -> :wat::core::String
      (:wat::core::first pair))
    (:wat::stdlib::sources)))
