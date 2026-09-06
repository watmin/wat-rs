;; Row 6 — a non-Row record with a multi-line string stays escaped.
(:wat::core::defrecord :probe::Note
  [text <- :wat::core::String])

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::kernel::pprintln
    (:probe::Note :text "line one\nline two")))
