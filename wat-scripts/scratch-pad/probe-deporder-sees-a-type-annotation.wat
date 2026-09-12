;; Does the stdlib load-order gate see a TYPE reference, or only a callable?
;;
;; NOTE-a-surface-cannot-see-a-type-declared-after-it.md asks for a census of
;; "which stdlib types are declared later than the surfaces that need them"
;; and says nobody has run it. :wat::deporder::verify is a PURE function, so
;; the question can be asked directly instead of reasoned from collect-kwds.
;;
;; Two cells, because one cell is not an instrument:
;;   A (MUST fire)     a.wat@0 names :t::b::Thing in a RETURN TYPE; b.wat@1 defenums it
;;   B (MUST NOT fire) the same two files in the opposite order
;;
;; deporder only PARSES, so the synthetic sources need not type-check.
;;
;; Run:
;;   ./target/release/wat wat-scripts/scratch-pad/probe-deporder-sees-a-type-annotation.wat

(:wat::core::defn :probe::a-src [] -> :wat::core::String
  "(:wat::core::defn :t::a::f [] -> :t::b::Thing 0)")

(:wat::core::defn :probe::b-src [] -> :wat::core::String
  "(:wat::core::defenum :t::b::Thing :wat::enum::Pure :Ok [])")

(:wat::core::defn :probe::file
  [path <- :wat::core::String  src <- :wat::core::String]
  -> :wat::source::File
  (:wat::source::File :path path :source src))

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::let
    [a     (:probe::file "a.wat" (:probe::a-src))
     b     (:probe::file "b.wat" (:probe::b-src))
     cellA (:wat::deporder::verify
             (:wat::core::Vector :- [:wat::source::File] a b))
     cellB (:wat::deporder::verify
             (:wat::core::Vector :- [:wat::source::File] b a))
     _ (:wat::kernel::println "CELL-A (must fire) violations:")
     _ (:wat::kernel::println (:wat::core::length cellA))
     _ (:wat::kernel::println cellA)
     _ (:wat::kernel::println "CELL-B (must not fire) violations:")
     _ (:wat::kernel::println (:wat::core::length cellB))]
    nil))
