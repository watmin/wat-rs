;; tests/rete/probe_arc278_P4c_native_retraction.wat — co-located fixture for the sibling probe (.rs),
;; slurped via startup_beside(file!()). Defines weather records for the native retraction differential.

(:wat::core::defrecord :weather::Temperature [celsius  <- :wat::core::i64  location <- :wat::core::String])
(:wat::core::defrecord :weather::WindSpeed    [kph      <- :wat::core::i64  location <- :wat::core::String])
(:wat::core::defrecord :weather::ColdAndWindy [location <- :wat::core::String])
(:wat::core::defrecord :weather::WeatherAlert [location <- :wat::core::String])
