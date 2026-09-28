;; Co-located fixture for tracked_wat_dir_is_stdlib_sources.rs — slurped via call_beside_value(file!()).
;; The baked stdlib's own path list, asked of the running substrate (`STDLIB_FILES` order).

(:wat::core::defn :user::stdlib-source-paths [] -> (wat.type/Vector :- [wat.type/String])
  (:wat::core::mapv
    (:wat::core::fn [pair <- (wat.type/Vector :- [wat.type/String])] -> wat.type/String
      (:wat::core::first pair))
    (:wat::stdlib::sources)))
