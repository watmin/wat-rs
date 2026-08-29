;; `is-Tag?` — the ONLY construction site for a Tag holon is `to_holon_inner`'s Uuid arm:
;;   Uuid -> Bind(Tag("uuid"), String(hex))
;; so the Tag is the LEFT of a Bind, never the top node. Asking `is-Tag?` of the uuid holon
;; itself therefore CANNOT be true — which is exactly why its column read all-false.
(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::let
    [u  (:wat::core::Uuid/nil)
     h  (:wat::holon::to-holon u)]
    (:wat::core::do
      (:wat::kernel::println "is-Tag? of the uuid holon (the Bind):")
      (:wat::kernel::println (:wat::holon::is-Tag? h))
      (:wat::kernel::println "extract-classifier of it:")
      (:wat::kernel::println (:wat::holon::extract-classifier h))
      (:wat::kernel::println "is-Tag? of Bind/left (where the Tag actually lives):")
      (:wat::kernel::println
        (:wat::holon::is-Tag? (:wat::core::Option/expect (:wat::holon::Bind/left h) "a uuid holon is a Bind"))))))
