;; Arc 278 — telemetry records acceptance probe (RED until wat/telemetry.wat ships :wat::telemetry').
;; A defn over :wat::telemetry'::Metric reading a SPLICED field (namespace, from Scope) and an OWN
;; field (name) type-checks IFF Metric exists AND splices the Scope surface (so Metric/namespace is
;; minted). RED at HEAD (Metric undefined); GREEN when the records ship + splice expands + the ctor is
;; the unified one. (The fuller construct + EDN round-trip is the shadowdancer's deftest'.)
(:wat::core::defn :probe::metric-ns   [m <- :wat::telemetry'::Metric] -> :wat::core::String
  (:wat::telemetry'::Metric/namespace m))          ;; a SPLICED field (from Scope)
(:wat::core::defn :probe::metric-name [m <- :wat::telemetry'::Metric] -> :wat::core::Keyword
  (:wat::telemetry'::Metric/name m))               ;; an OWN field
(:wat::core::defn :probe::log-caller  [l <- :wat::telemetry'::Log] -> :wat::core::Keyword
  (:wat::telemetry'::Log/caller l))
(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::kernel::println "telemetry records present + splice Scope"))
