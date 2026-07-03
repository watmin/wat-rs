;; tests/rete/probe_arc278_return_type_of.wat — co-located fixture for the sibling probe (.rs),
;; slurped via startup_beside(file!()). Defines :weather::ColdAndWindy for return-type-of tests.

(:wat::core::defrecord :weather::ColdAndWindy [location <- :wat::core::String])
