(:wat::core::defenum :probe::Color :wat::enum::Pure :Red [] :Green [])
(:wat::core::defn :user::pick [] -> :probe::Color (:probe::Color.Red {}))
(wat.core/defn user/main [] :- wat.type/nil (wat.kernel/println "ok"))
