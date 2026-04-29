;; :wat::telemetry::WorkUnit — measurement-scope state surface.
;;
;; The Rust shim at :rust::telemetry::WorkUnit holds the four pieces
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

(:wat::core::use! :rust::telemetry::WorkUnit)

;; `:wat::telemetry::Tag` and `:wat::telemetry::Tags` typealiases live
;; in `wat/measure/types.wat`, registered ahead of this file in
;; the crate's `wat_sources()`.
(:wat::core::typealias :wat::telemetry::WorkUnit :rust::telemetry::WorkUnit)


(:wat::core::define
  (:wat::telemetry::WorkUnit::new
    (tags :wat::telemetry::Tags)
    -> :wat::telemetry::WorkUnit)
  (:rust::telemetry::WorkUnit::new tags))


(:wat::core::define
  (:wat::telemetry::WorkUnit/uuid
    (wu :wat::telemetry::WorkUnit) -> :String)
  (:rust::telemetry::WorkUnit::uuid wu))


(:wat::core::define
  (:wat::telemetry::WorkUnit/incr!
    (wu :wat::telemetry::WorkUnit)
    (name :wat::holon::HolonAST)
    -> :())
  (:rust::telemetry::WorkUnit::incr wu name))


(:wat::core::define
  (:wat::telemetry::WorkUnit/append-dt!
    (wu :wat::telemetry::WorkUnit)
    (name :wat::holon::HolonAST)
    (secs :f64)
    -> :())
  ;; The Rust shim's path mirrors its Rust ident verbatim — the
  ;; #[wat_dispatch] macro uses `method.sig.ident` directly, so the
  ;; path is `append_dt` (underscore), not `append-dt` (kebab). The
  ;; wat-side wrapper here owns the kebab name; the rust path is
  ;; an internal detail.
  (:rust::telemetry::WorkUnit::append_dt wu name secs))


(:wat::core::define
  (:wat::telemetry::WorkUnit/counter
    (wu :wat::telemetry::WorkUnit)
    (name :wat::holon::HolonAST)
    -> :i64)
  (:rust::telemetry::WorkUnit::counter wu name))


(:wat::core::define
  (:wat::telemetry::WorkUnit/durations
    (wu :wat::telemetry::WorkUnit)
    (name :wat::holon::HolonAST)
    -> :Vec<f64>)
  (:rust::telemetry::WorkUnit::durations wu name))


;; ─── Slice 4 accessors — read state needed by WorkUnit/scope ────

(:wat::core::define
  (:wat::telemetry::WorkUnit/started-epoch-nanos
    (wu :wat::telemetry::WorkUnit) -> :i64)
  ;; Rust ident `started_epoch_nanos`; the macro registers with
  ;; underscore (cf. slice-3's append_dt). The wat-side keeps the
  ;; kebab name.
  (:rust::telemetry::WorkUnit::started_epoch_nanos wu))


(:wat::core::define
  (:wat::telemetry::WorkUnit/counters-keys
    (wu :wat::telemetry::WorkUnit) -> :Vec<wat::holon::HolonAST>)
  (:rust::telemetry::WorkUnit::counters_keys wu))


(:wat::core::define
  (:wat::telemetry::WorkUnit/durations-keys
    (wu :wat::telemetry::WorkUnit) -> :Vec<wat::holon::HolonAST>)
  (:rust::telemetry::WorkUnit::durations_keys wu))


;; ─── WorkUnit/scope<T> — measurement HOF ─────────────────────────
;;
;; Opens a fresh WorkUnit, runs body with it, returns body's value.
;; Body is `:fn(WorkUnit) -> T` — 1-arity, receives the wu so it
;; can incr! / append-dt! / read tags / etc. The scope HOF is
;; pure-wat; no Rust-side eval needed.
;;
;; Slice 4 ships this bare shape. The companion slice (4-ship)
;; adds the auto-ship at scope-close that walks counters +
;; durations into Vec<Event::Metric> rows and batch-logs them
;; through SinkHandles. Splitting the slice keeps each stepping
;; stone testable independently.

(:wat::core::define
  (:wat::telemetry::WorkUnit/scope<T>
    (tags :wat::telemetry::Tags)
    (body :fn(wat::telemetry::WorkUnit)->T)
    -> :T)
  (:wat::core::let*
    (((wu     :wat::telemetry::WorkUnit) (:wat::telemetry::WorkUnit::new tags))
     ((result :T)                       (body wu)))
    result))


;; ─── Slice 4-ship helpers — build Event::Metric rows ────────────
;;
;; Each counter that the scope tracked emits ONE Event::Metric at
;; scope-close (CloudWatch model: a counter ending at 7 → one row,
;; metric-value = leaf 7). Each duration sample emits ONE
;; Event::Metric row of its own — N samples means N rows.
;; metric-value is uniformly a primitive HolonAST leaf in NoTag —
;; never a Bundle.
;;
;; build-counter-metric / build-duration-metric live as separate
;; helpers because the constructor's 8 args degrade readability when
;; inlined into the foldl bodies, and the helper's signature is its
;; own contract.

(:wat::core::define
  (:wat::telemetry::WorkUnit/scope::build-counter-metric
    (start-time-ns :i64)
    (end-time-ns   :i64)
    (namespace     :wat::holon::HolonAST)
    (uuid          :String)
    (tags          :wat::telemetry::Tags)
    (name          :wat::holon::HolonAST)
    (count         :i64)
    -> :wat::telemetry::Event)
  (:wat::telemetry::Event::Metric
    start-time-ns
    end-time-ns
    (:wat::edn::NoTag/new namespace)
    uuid
    tags
    (:wat::edn::NoTag/new name)
    (:wat::edn::NoTag/new (:wat::holon::leaf count))
    (:wat::edn::NoTag/new (:wat::holon::leaf :count))))


;; ─── Tags — the third concern, IMMUTABLE for the scope ─────────
;;
;; Tags are declared upfront at WorkUnit::new and are immutable
;; for the scope's lifetime. There is NO assoc/disassoc — every
;; Log line emitted in the scope must carry the same tag set so
;; rows correlate via a stable queryable shape. The user can read
;; the map natively with `:wat::core::get`, `:wat::core::keys`,
;; etc. — no per-key accessor needed.
;;
;; The map serializes to the SQL `tags` column as a clean EDN
;; map: `{:asset :BTC, :stage :market-eval}`. Slice 4 picks the
;; field-type shape that drives that rendering.

(:wat::core::define
  (:wat::telemetry::WorkUnit/tags
    (wu :wat::telemetry::WorkUnit) -> :wat::telemetry::Tags)
  (:rust::telemetry::WorkUnit::tags wu))
