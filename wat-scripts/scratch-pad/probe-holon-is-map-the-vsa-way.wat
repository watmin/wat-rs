;; Arc 226's ACTUAL mission: `(is-Map? x) ≡ similarity(extract-classifier(x), prototype-of("Map"))`.
;; What shipped is a string compare. Is the VSA route buildable from primitives that exist NOW?
;;   is-Map?(h)  ≡  coincident?(Bind/left(h), Atom("Map"))
(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::let
    [m        (:wat::holon::Map (:wat::core::Vector :wat::holon::HolonAST #holon :a #holon 1))
     v        (:wat::holon::Vector (:wat::core::Vector :wat::holon::HolonAST #holon 1))
     marker   (:wat::holon::Atom (:wat::holon::leaf "Map"))]
    (:wat::core::do
      (:wat::kernel::println "-- the STRUCTURAL predicate that shipped --")
      (:wat::kernel::println (:wat::holon::is-Map? m))
      (:wat::kernel::println (:wat::holon::is-Map? v))
      (:wat::kernel::println "-- the VSA route: coincident?(Bind/left(h), Atom(\"Map\")) --")
      (:wat::kernel::println
        (:wat::holon::coincident? (:wat::core::Option/expect (:wat::holon::Bind/left m) "classified") marker))
      (:wat::kernel::println
        (:wat::holon::coincident? (:wat::core::Option/expect (:wat::holon::Bind/left v) "classified") marker))
      (:wat::kernel::println "-- and the CONTINUOUS answer the design asked for --")
      (:wat::kernel::println
        (:wat::holon::cosine (:wat::core::Option/expect (:wat::holon::Bind/left m) "classified") marker))
      (:wat::kernel::println
        (:wat::holon::cosine (:wat::core::Option/expect (:wat::holon::Bind/left v) "classified") marker)))))
