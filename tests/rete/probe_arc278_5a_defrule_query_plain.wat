;; tests/rete/probe_arc278_5a_defrule_query_plain.wat — records-only fixture (no defrule) for the
;; probe_arc278_5a_defrule_query probe; loaded via startup_from_file for the query-only tests.

(:wat::core::defrecord :weather::Temperature [celsius  <- :wat::core::i64  location <- :wat::core::String])
(:wat::core::defrecord :weather::WindSpeed    [kph      <- :wat::core::i64  location <- :wat::core::String])
(:wat::core::defrecord :weather::ColdAndWindy [location <- :wat::core::String])
(:wat::core::defn :user::main [] -> :wat::core::nil nil)
