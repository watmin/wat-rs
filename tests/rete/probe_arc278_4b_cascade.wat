;; tests/rete/probe_arc278_4b_cascade.wat — co-located fixture for the sibling probe (.rs),
;; slurped via startup_beside(file!()). Defines the weather records for cascade-to-fixpoint tests.

(:wat::core::defrecord :weather::Temperature [celsius  <- :wat::core::i64  location <- :wat::core::String])
(:wat::core::defrecord :weather::WindSpeed    [kph      <- :wat::core::i64  location <- :wat::core::String])
(:wat::core::defrecord :weather::ColdAndWindy [location <- :wat::core::String])
(:wat::core::defrecord :weather::WeatherAlert [location <- :wat::core::String])
