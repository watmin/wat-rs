;; `watast_to_holon` builds Map/Set as `Bind(String("Map"), ...)`.
;; `extract_classifier` requires `Bind(Atom(String(s)), _)` — an ATOM wrapper.
;; `to_holon_inner`'s Vec arm DOES Atom-wrap. So: does the round-trip still work (a private
;; convention read by a different reader), or is the classifier simply unreadable (a bug)?
(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::let
    [m #holon {:a 1}]
    (:wat::core::do
      (:wat::kernel::println "round-trip #holon {:a 1} through from-holon:")
      (:wat::kernel::println (:wat::holon::from-holon m))
      (:wat::kernel::println "extract-classifier says:")
      (:wat::kernel::println (:wat::holon::extract-classifier m)))))
