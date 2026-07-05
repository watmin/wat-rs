;; Arc 293 — surface-splice acceptance probe (RED at HEAD, GREEN when splice ships).
;;
;; THE GAP: a `defrecord` field vector may splice a surface's ATTRIBUTES via `~@:Surface`,
;; inlining them flat before the own fields (AGGREGATE-MODEL principle 4 / DESIGN.md:130 —
;; "spliceable into bodies for DRY `[~@:geo::Planar radius <- :f64]`"). Designed, never built:
;; the field-vector parser (`parse_aggregate_fields`) has zero `~@` handling, so the reader's
;; `(:wat::core::unquote-splicing :Surface)` node trips "name must be a plain symbol".
;;
;; GREEN when: `~@:Surface` in an aggregate field vector expands to the surface's Field members
;; (in declared order) inlined before the own fields; the record then constructs positionally
;; over the merged field list, and each spliced field gets its own `:Rec/field` accessor.

;; a Scope-shaped surface: attributes only (namespace, uuid, time-ns).
(:wat::core::defsurface :probe::Scope :holder :wat::core::Record
  :features [namespace <- :wat::core::String
             uuid      <- :wat::core::String
             time-ns   <- :wat::core::i64])

;; a second surface, to prove MULTIPLE splices merge (a record satisfying both).
(:wat::core::defsurface :probe::Named :holder :wat::core::Record
  :features [name <- :wat::core::String])

;; a record that splices BOTH surfaces' attributes, then adds its own field.
;; merged field order (first-occurrence): namespace, uuid, time-ns, name, value.
(:wat::core::defrecord :probe::Metric
  [~@:probe::Scope
   ~@:probe::Named
   value <- :wat::core::i64])

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::let [m (:probe::Metric "market-eval" "u-123" 456 "requests" 7)]
    ;; a spliced field (from Scope) + a spliced field (from Named) + the own field
    (:wat::kernel::println (:probe::Metric/namespace m))
    (:wat::kernel::println (:probe::Metric/name m))
    (:wat::kernel::println (:probe::Metric/value m))))
