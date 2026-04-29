;; :wat::measure::WorkUnit — measurement-scope state surface.
;;
;; The Rust shim at :rust::measure::WorkUnit holds the four pieces
;; every scope tracks: counters (HashMap<Value, i64>), durations
;; (HashMap<Value, Vec<f64>>), `started: Instant`, and `uuid:
;; String`. Mutation is in place via ThreadOwnedCell — same Tier-2
;; zero-mutex pattern wat-lru's LocalCache uses.
;;
;; Slice 3 ships the data primitives:
;;   - WorkUnit::new                                     -> WorkUnit
;;   - WorkUnit/uuid       wu                            -> String
;;   - WorkUnit/incr!      wu (name :HolonAST)           -> ()
;;   - WorkUnit/append-dt! wu (name :HolonAST) (s :f64)  -> ()
;;   - WorkUnit/counter    wu (name :HolonAST)           -> i64
;;   - WorkUnit/durations  wu (name :HolonAST)           -> Vec<f64>
;;
;; Slice 4 will add WorkUnit/scope<T> (the HOF that opens a fresh
;; wu, runs body, computes elapsed, walks counters+durations to
;; build LogEntry::Metric rows, ships them through the consumer's
;; Service handles, returns body's value).
;;
;; The `!` suffix on incr!/append-dt! follows wat's mutation
;; convention (cf. :wat::core::set!). Reads have no suffix.
;;
;; Keys are HolonAST. A wat keyword like `:requests` becomes
;; HolonAST via `(:wat::holon::Atom :requests)`. Per arc 057 the
;; substrate's hashmap_key accepts any hashable Value, so the
;; runtime is permissive — the wat-level type discipline is what
;; keeps slice 4's edn-write-notag rendering clean.
;;
;; Arc 091 slice 3.

(:wat::core::use! :rust::measure::WorkUnit)

(:wat::core::typealias :wat::measure::WorkUnit :rust::measure::WorkUnit)


(:wat::core::define
  (:wat::measure::WorkUnit::new -> :wat::measure::WorkUnit)
  (:rust::measure::WorkUnit::new))


(:wat::core::define
  (:wat::measure::WorkUnit/uuid
    (wu :wat::measure::WorkUnit) -> :String)
  (:rust::measure::WorkUnit::uuid wu))


(:wat::core::define
  (:wat::measure::WorkUnit/incr!
    (wu :wat::measure::WorkUnit)
    (name :wat::holon::HolonAST)
    -> :())
  (:rust::measure::WorkUnit::incr wu name))


(:wat::core::define
  (:wat::measure::WorkUnit/append-dt!
    (wu :wat::measure::WorkUnit)
    (name :wat::holon::HolonAST)
    (secs :f64)
    -> :())
  ;; The Rust shim's path mirrors its Rust ident verbatim — the
  ;; #[wat_dispatch] macro uses `method.sig.ident` directly, so the
  ;; path is `append_dt` (underscore), not `append-dt` (kebab). The
  ;; wat-side wrapper here owns the kebab name; the rust path is
  ;; an internal detail.
  (:rust::measure::WorkUnit::append_dt wu name secs))


(:wat::core::define
  (:wat::measure::WorkUnit/counter
    (wu :wat::measure::WorkUnit)
    (name :wat::holon::HolonAST)
    -> :i64)
  (:rust::measure::WorkUnit::counter wu name))


(:wat::core::define
  (:wat::measure::WorkUnit/durations
    (wu :wat::measure::WorkUnit)
    (name :wat::holon::HolonAST)
    -> :Vec<f64>)
  (:rust::measure::WorkUnit::durations wu name))
