;; tests/function/probe_arc259_s2cii0_record_dispatch.wat
;; Arc 259 S2c-ii.0 — defclause dispatch on a record's specific class (class_fqdn).
;; Co-located fixture for the sibling probe (.rs), slurped via startup_beside(file!()).

(:wat::core::defrecord :user::Tag [])
(:wat::core::defclause :user::id-tag
  ([t <- :user::Tag] -> :wat::core::i64 7))
(:wat::core::defn :user::compute [] -> :wat::core::i64
  (:user::id-tag (:user::Tag)))
