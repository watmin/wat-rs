;; benches/perf_arc278_fire_baseline.wat — co-located fixture for the sibling benchmark (.rs),
;; slurped via startup_beside(file!()). Defines weather records for the fire throughput baseline.
;;
;; Relocated verbatim from tests/rete/perf_arc278_fire_baseline.wat (296 Stone K, move 1).

(:wat::core::defrecord :weather::Temperature [celsius  <- wat.type/i64  location <- wat.type/String])
(:wat::core::defrecord :weather::WindSpeed    [kph      <- wat.type/i64  location <- wat.type/String])
(:wat::core::defrecord :weather::ColdAndWindy [location <- wat.type/String])
