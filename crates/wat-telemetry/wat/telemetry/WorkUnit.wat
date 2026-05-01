;; :wat::telemetry::WorkUnit — measurement-scope state surface.
;;
;; The Rust shim at :rust::telemetry::WorkUnit holds the four pieces
;; every scope tracks: counters (wat::core::HashMap<Value, i64>), durations
;; (wat::core::HashMap<Value, wat::core::Vector<f64>>), `started: Instant`, and `uuid:
;; String`. Mutation is in place via ThreadOwnedCell — same Tier-2
;; zero-mutex pattern wat-lru's LocalCache uses.
;;
;; Slice 3 ships the data primitives:
;;   - WorkUnit::new                                     -> WorkUnit
;;   - WorkUnit/uuid       wu                            -> String
;;   - WorkUnit/incr!      wu (name :HolonAST)           -> ()
;;   - WorkUnit/append-dt! wu (name :HolonAST) (s :wat::core::f64)  -> ()
;;   - WorkUnit/counter    wu (name :HolonAST)           -> i64
;;   - WorkUnit/durations  wu (name :HolonAST)           -> wat::core::Vector<f64>
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
    (namespace :wat::holon::HolonAST)
    (tags      :wat::telemetry::Tags)
    -> :wat::telemetry::WorkUnit)
  (:rust::telemetry::WorkUnit::new namespace tags))


(:wat::core::define
  (:wat::telemetry::WorkUnit/namespace
    (wu :wat::telemetry::WorkUnit) -> :wat::holon::HolonAST)
  (:rust::telemetry::WorkUnit::namespace wu))


(:wat::core::define
  (:wat::telemetry::WorkUnit/uuid
    (wu :wat::telemetry::WorkUnit) -> :wat::core::String)
  (:rust::telemetry::WorkUnit::uuid wu))


(:wat::core::define
  (:wat::telemetry::WorkUnit/incr!
    (wu :wat::telemetry::WorkUnit)
    (name :wat::holon::HolonAST)
    -> :wat::core::unit)
  (:rust::telemetry::WorkUnit::incr wu name))


(:wat::core::define
  (:wat::telemetry::WorkUnit/append-dt!
    (wu :wat::telemetry::WorkUnit)
    (name :wat::holon::HolonAST)
    (secs :wat::core::f64)
    -> :wat::core::unit)
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
    -> :wat::core::i64)
  (:rust::telemetry::WorkUnit::counter wu name))


(:wat::core::define
  (:wat::telemetry::WorkUnit/durations
    (wu :wat::telemetry::WorkUnit)
    (name :wat::holon::HolonAST)
    -> :wat::core::Vector<wat::core::f64>)
  (:rust::telemetry::WorkUnit::durations wu name))


;; ─── Slice 4 accessors — read state needed by WorkUnit/scope ────

(:wat::core::define
  (:wat::telemetry::WorkUnit/started-epoch-nanos
    (wu :wat::telemetry::WorkUnit) -> :wat::core::i64)
  ;; Rust ident `started_epoch_nanos`; the macro registers with
  ;; underscore (cf. slice-3's append_dt). The wat-side keeps the
  ;; kebab name.
  (:rust::telemetry::WorkUnit::started_epoch_nanos wu))


(:wat::core::define
  (:wat::telemetry::WorkUnit/counters-keys
    (wu :wat::telemetry::WorkUnit) -> :wat::holon::Holons)
  (:rust::telemetry::WorkUnit::counters_keys wu))


(:wat::core::define
  (:wat::telemetry::WorkUnit/durations-keys
    (wu :wat::telemetry::WorkUnit) -> :wat::holon::Holons)
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
;; durations into wat::core::Vector<Event::Metric> rows and batch-logs them
;; through SinkHandles. Splitting the slice keeps each stepping
;; stone testable independently.

(:wat::core::define
  (:wat::telemetry::WorkUnit/scope<T>
    (namespace :wat::holon::HolonAST)
    (tags      :wat::telemetry::Tags)
    (body      :fn(wat::telemetry::WorkUnit)->T)
    -> :T)
  (:wat::core::let*
    (((wu     :wat::telemetry::WorkUnit) (:wat::telemetry::WorkUnit::new namespace tags))
     ((result :T)                        (body wu)))
    result))


;; ─── WorkUnit/make-scope — closure factory + auto-ship ──────────
;;
;; The user's direction (2026-04-29): "we want our deps to vanish
;; as fast as possible. (make-unit-work-maker handle namespace) ->
;; produces a func who does what (WorkUnit/scope ...) is maybe
;; trying to do." Captures (handle, namespace) once. The returned
;; closure takes only (tags, body) — tags vary per scope-call,
;; namespace is fixed per producer. Body's T flows back to the
;; caller; metrics ship at scope-close via batch-log on the
;; captured handle.
;;
;; Returns :Scope<T> per the typealias in types.wat.
(:wat::core::define
  (:wat::telemetry::WorkUnit/make-scope<T>
    (handle    :wat::telemetry::SinkHandles)
    (namespace :wat::holon::HolonAST)
    -> :wat::telemetry::WorkUnit::Scope<T>)
  (:wat::core::lambda
    ((tags :wat::telemetry::Tags)
     (body :wat::telemetry::WorkUnit::Body<T>)
     -> :T)
    (:wat::core::let*
      (((wu     :wat::telemetry::WorkUnit) (:wat::telemetry::WorkUnit::new namespace tags))
       ((result :T)                        (body wu))
       ((start  :wat::core::i64) (:wat::telemetry::WorkUnit/started-epoch-nanos wu))
       ((end    :wat::core::i64) (:wat::time::epoch-nanos (:wat::time::now)))
       ((events :wat::core::Vector<wat::telemetry::Event>)
        (:wat::telemetry::WorkUnit/scope::collect-metric-events wu start end))
       ((req-tx :wat::telemetry::Service::ReqTx<wat::telemetry::Event>)
        (:wat::core::first handle))
       ((ack-rx :wat::telemetry::Service::AckRx) (:wat::core::second handle))
       ((_ship  :wat::core::unit)
        (:wat::telemetry::Service/batch-log req-tx ack-rx events)))
      result)))


;; ─── WorkUnit/timed — bump + measure-around body ────────────────
;;
;; Composes `incr!` + epoch-nanos delta + `append-dt!` at the wat
;; surface (no Rust required). One call:
;;
;;   - bumps `name`'s counter by 1
;;   - captures wall-clock nanos before the body
;;   - runs (body) — returns its T verbatim through this call
;;   - captures wall-clock nanos after the body
;;   - appends `(end - start) / 1e9` seconds to `name`'s duration list
;;
;; Single-name discipline (counter and duration share the key) keeps
;; the row count predictable: N timed calls under one name ⇒ ONE
;; counter row at scope-close (CloudWatch model: counter = N) plus
;; N duration rows (one per sample).
(:wat::core::define
  (:wat::telemetry::WorkUnit/timed<T>
    (wu   :wat::telemetry::WorkUnit)
    (name :wat::holon::HolonAST)
    (body :fn()->T)
    -> :T)
  (:wat::core::let*
    (((_bump      :wat::core::unit)  (:wat::telemetry::WorkUnit/incr! wu name))
     ((start      :wat::core::i64) (:wat::time::epoch-nanos (:wat::time::now)))
     ((result     :T)   (body))
     ((end        :wat::core::i64) (:wat::time::epoch-nanos (:wat::time::now)))
     ((delta-ns   :wat::core::i64) (:wat::core::- end start))
     ((delta-ns-f :wat::core::f64) (:wat::core::i64::to-f64 delta-ns))
     ((secs       :wat::core::f64) (:wat::core::/ delta-ns-f 1000000000.0))
     ((_dt        :wat::core::unit)  (:wat::telemetry::WorkUnit/append-dt! wu name secs)))
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
    (start-time-ns :wat::core::i64)
    (end-time-ns   :wat::core::i64)
    (namespace     :wat::holon::HolonAST)
    (uuid          :wat::core::String)
    (tags          :wat::telemetry::Tags)
    (name          :wat::holon::HolonAST)
    (count         :wat::core::i64)
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


;; ONE sample → ONE row. The scope's foldl over durations-keys
;; iterates each named timer; for each name it foldls over its
;; samples Vec, calling this helper per sample. metric-value is
;; the f64 lifted to HolonAST::F64 via leaf; unit is :seconds.
(:wat::core::define
  (:wat::telemetry::WorkUnit/scope::build-duration-metric
    (start-time-ns :wat::core::i64)
    (end-time-ns   :wat::core::i64)
    (namespace     :wat::holon::HolonAST)
    (uuid          :wat::core::String)
    (tags          :wat::telemetry::Tags)
    (name          :wat::holon::HolonAST)
    (sample        :wat::core::f64)
    -> :wat::telemetry::Event)
  (:wat::telemetry::Event::Metric
    start-time-ns
    end-time-ns
    (:wat::edn::NoTag/new namespace)
    uuid
    tags
    (:wat::edn::NoTag/new name)
    (:wat::edn::NoTag/new (:wat::holon::leaf sample))
    (:wat::edn::NoTag/new (:wat::holon::leaf :seconds))))


;; Per-name duration fanout — one Event::Metric row per sample.
;; Helper extracted so collect-metric-events can stay one outer
;; let* (per the "simple forms per func" feedback rule). The
;; outer walker calls this once per duration-name; inside we
;; foldl over that name's samples Vec.
(:wat::core::define
  (:wat::telemetry::WorkUnit/scope::collect-duration-events-for-name
    (start-time-ns :wat::core::i64)
    (end-time-ns   :wat::core::i64)
    (namespace     :wat::holon::HolonAST)
    (uuid          :wat::core::String)
    (tags          :wat::telemetry::Tags)
    (name          :wat::holon::HolonAST)
    (samples       :wat::core::Vector<wat::core::f64>)
    -> :wat::core::Vector<wat::telemetry::Event>)
  (:wat::core::foldl samples
    (:wat::core::Vector :wat::telemetry::Event)
    (:wat::core::lambda
      ((acc    :wat::core::Vector<wat::telemetry::Event>)
       (sample :wat::core::f64)
       -> :wat::core::Vector<wat::telemetry::Event>)
      (:wat::core::concat acc
        (:wat::core::Vector :wat::telemetry::Event
          (:wat::telemetry::WorkUnit/scope::build-duration-metric
            start-time-ns end-time-ns namespace uuid tags name sample))))))


;; collect-metric-events — at scope-close, walks the wu's counters
;; and durations into a flat wat::core::Vector<Event>. Slice 4-ship's central
;; piece. Counters: ONE row per name (final count). Durations:
;; ONE row per sample (CloudWatch fanout). Namespace pulled
;; from wu (per the user's "namespace adjacent to tags" rule).
(:wat::core::define
  (:wat::telemetry::WorkUnit/scope::collect-metric-events
    (wu            :wat::telemetry::WorkUnit)
    (start-time-ns :wat::core::i64)
    (end-time-ns   :wat::core::i64)
    -> :wat::core::Vector<wat::telemetry::Event>)
  (:wat::core::let*
    (((namespace      :wat::holon::HolonAST)        (:wat::telemetry::WorkUnit/namespace wu))
     ((uuid           :wat::core::String)                     (:wat::telemetry::WorkUnit/uuid wu))
     ((tags           :wat::telemetry::Tags)        (:wat::telemetry::WorkUnit/tags wu))
     ((counter-keys   :wat::holon::Holons)   (:wat::telemetry::WorkUnit/counters-keys wu))
     ((duration-keys  :wat::holon::Holons)   (:wat::telemetry::WorkUnit/durations-keys wu))
     ((counter-events :wat::core::Vector<wat::telemetry::Event>)
      (:wat::core::foldl counter-keys
        (:wat::core::Vector :wat::telemetry::Event)
        (:wat::core::lambda
          ((acc :wat::core::Vector<wat::telemetry::Event>)
           (key :wat::holon::HolonAST)
           -> :wat::core::Vector<wat::telemetry::Event>)
          (:wat::core::let*
            (((count :wat::core::i64) (:wat::telemetry::WorkUnit/counter wu key))
             ((event :wat::telemetry::Event)
              (:wat::telemetry::WorkUnit/scope::build-counter-metric
                start-time-ns end-time-ns namespace uuid tags key count)))
            (:wat::core::concat acc
              (:wat::core::Vector :wat::telemetry::Event event))))))
     ((duration-events :wat::core::Vector<wat::telemetry::Event>)
      (:wat::core::foldl duration-keys
        (:wat::core::Vector :wat::telemetry::Event)
        (:wat::core::lambda
          ((acc :wat::core::Vector<wat::telemetry::Event>)
           (key :wat::holon::HolonAST)
           -> :wat::core::Vector<wat::telemetry::Event>)
          (:wat::core::let*
            (((samples :wat::core::Vector<wat::core::f64>)
              (:wat::telemetry::WorkUnit/durations wu key))
             ((per-name :wat::core::Vector<wat::telemetry::Event>)
              (:wat::telemetry::WorkUnit/scope::collect-duration-events-for-name
                start-time-ns end-time-ns namespace uuid tags key samples)))
            (:wat::core::concat acc per-name))))))
    (:wat::core::concat counter-events duration-events)))


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
