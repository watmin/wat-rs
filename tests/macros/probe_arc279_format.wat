;; tests/macros/probe_arc279_format.wat — co-located fixture for
;; probe_arc279_format.rs (positive test), slurped via startup_beside(file!()).
;;
;; Named placeholders, out-of-order kwargs, heterogeneous values (String + i64), static text.
;; The format call is inside a defn so it gets macro-expanded at startup.
(:wat::core::defn :user::test-format [] -> :wat::core::String
  (:wat::core::format "{greeting}, {name}! you have {count} messages"
    :name "ada" :greeting "hello" :count 3))
