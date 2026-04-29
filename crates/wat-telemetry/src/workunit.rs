//! `:rust::telemetry::WorkUnit` — measurement-scope state.
//!
//! Arc 091 slice 3. Holds the four pieces every measurement scope
//! tracks: counters (HashMap<Value, i64>), durations
//! (HashMap<Value, Vec<f64>>), `started: Instant`, and `uuid:
//! String`. `started` is internal state; the elapsed-time
//! computation lands in slice 4 (`WorkUnit/scope` HOF).
//!
//! Mutation primitives — `incr!`, `append-dt!` — bump the counters
//! and durations in place via the `#[wat_dispatch]`-managed
//! `ThreadOwnedCell<WatMeasureWorkUnit>`. Same Tier-2 zero-mutex
//! pattern wat-lru's `LocalCache` uses; the cell binds to the
//! calling thread, the borrow-checker proves there can't be a
//! second mutable reference.
//!
//! Read primitives — `uuid`, `counter`, `durations` — return
//! cloned snapshots. `counter` returns 0 for absent keys;
//! `durations` returns an empty Vec. Friendly defaults; slice 4's
//! ship walker iterates known keys directly.
//!
//! Keys are `Value`-typed at the Rust boundary; canonicalization
//! goes through `wat::runtime::hashmap_key` (per arc 057 — accepts
//! primitives plus HolonAST). Storage shape mirrors wat-lru's
//! `LruCache<String, (Value, _)>`: a canonical-string key into a
//! pair of `(original-key-Value, payload)` so reads can return the
//! original key Value at iteration time.

use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};

use wat::rust_deps::RustDepsBuilder;
use wat::runtime::{hashmap_key, Value};
use wat_macros::wat_dispatch;

/// Register the `:rust::telemetry::WorkUnit` shim into the deps
/// builder. Forwards to the macro-generated register fn. Lives in
/// this module (not `lib.rs`) because the macro-generated module
/// is visibility-private to its declaring module — same pattern
/// wat-lru's `shim::register` uses.
pub fn register(builder: &mut RustDepsBuilder) {
    __wat_dispatch_WatMeasureWorkUnit::register(builder);
}

/// The opaque carried as `:rust::telemetry::WorkUnit` at the wat
/// level. Each instance is single-thread-owned via the macro's
/// `scope = "thread_owned"` wrapping.
pub struct WatMeasureWorkUnit {
    /// Bumps via `incr!`. Map keys are canonical strings;
    /// `(Value, i64)` carries the original-key-Value alongside
    /// the count so iteration can hand back the wat-shaped key.
    counters: HashMap<String, (Value, i64)>,
    /// Appends via `append-dt!`. Same canonical-key + original-Value
    /// shape; payload is a `Vec<f64>` of seconds-deltas.
    durations: HashMap<String, (Value, Vec<f64>)>,
    /// **Immutable** for the scope's lifetime. Declared upfront at
    /// `new(tags)` and read out at ship-time to attach to every
    /// emitted Event row (Metric AND Log). The immutability is
    /// load-bearing: every Log line emitted within the scope must
    /// share the same tag set so the rows correlate via a stable
    /// queryable shape. assoc/disassoc would let log-time-N differ
    /// from log-time-M and break that invariant. Carried as a
    /// `Value::wat__std__HashMap` so wat-side code reads the map
    /// natively (no Rust-side walking).
    tags: Value,
    /// **Immutable** — declared at `new(namespace, tags)` adjacent
    /// to tags. The producing fn's fqdn keyword (e.g. `:my::ns`),
    /// stored as a HolonAST. Every Event::Metric and Event::Log
    /// row emitted from this wu carries this same namespace as the
    /// row's `namespace` column. Logs in slice 5 also pull from
    /// here so emit-sites don't repeat it.
    namespace: Value,
    /// Wall-clock epoch nanoseconds at scope-open. Captured via
    /// `chrono::Utc::now()` since `Instant` is monotonic-only and
    /// can't anchor to wall-clock for the metric table's
    /// `start-time-ns` column. Slice 4's `WorkUnit/scope` HOF
    /// reads this at scope-close to populate `start-time-ns` on
    /// every emitted Event::Metric row.
    started_epoch_nanos: i64,
    /// Canonical 8-4-4-4-12 hyphenated v4 UUID. Minted via
    /// `wat_edn::new_uuid_v4()` (arc 092) at construction.
    uuid: String,
}

#[wat_dispatch(
    path = ":rust::telemetry::WorkUnit",
    scope = "thread_owned"
)]
impl WatMeasureWorkUnit {
    /// `:rust::telemetry::WorkUnit::new namespace tags` — fresh
    /// scope. `namespace` MUST be a HolonAST value (the producing
    /// fn's fqdn keyword); `tags` MUST be a HashMap<HolonAST,
    /// HolonAST>. Both are immutable for the scope's lifetime.
    /// Adjacent at construction per the user's direction
    /// 2026-04-29: namespace IS the wu's identity, not a queryable
    /// tag.
    pub fn new(namespace: Value, tags: Value) -> Self {
        if !matches!(namespace, Value::holon__HolonAST(_)) {
            panic!(
                ":rust::telemetry::WorkUnit::new: namespace must be a HolonAST value; got {}",
                namespace.type_name()
            );
        }
        if !matches!(tags, Value::wat__std__HashMap(_)) {
            panic!(
                ":rust::telemetry::WorkUnit::new: tags must be a HashMap value; got {}",
                tags.type_name()
            );
        }
        // Wall-clock epoch nanos at scope open. `SystemTime` can in
        // theory go before UNIX_EPOCH (manual clock skew during NTP
        // sync); treat that case as zero so the field is always
        // monotone-non-negative for downstream SQL math.
        let started_epoch_nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_nanos() as i64)
            .unwrap_or(0);
        WatMeasureWorkUnit {
            counters: HashMap::new(),
            durations: HashMap::new(),
            tags,
            namespace,
            started_epoch_nanos,
            uuid: wat_edn::new_uuid_v4().to_string(),
        }
    }

    /// `:rust::telemetry::WorkUnit::namespace wu` — the immutable
    /// HolonAST namespace declared at `new()`. Read by slice 4's
    /// ship walker for each emitted row's `namespace` column;
    /// read by slice 5's log emitters for the same column on
    /// Event::Log.
    pub fn namespace(&self) -> Value {
        self.namespace.clone()
    }

    /// `:rust::telemetry::WorkUnit::started-epoch-nanos wu` — wall-clock
    /// nanos at scope open. Slice 4's `WorkUnit/scope` reads this
    /// alongside `(:wat::time::epoch-nanos (:wat::time::now))` at
    /// scope-close to populate the metric row's `start-time-ns` and
    /// `end-time-ns` columns.
    pub fn started_epoch_nanos(&self) -> i64 {
        self.started_epoch_nanos
    }

    /// `:rust::telemetry::WorkUnit::counters-keys wu` — the original
    /// key Values for every counter that was ever bumped. Slice 4's
    /// ship walker iterates this to emit one Event::Metric row per
    /// counter (CloudWatch model: each counter is a single data
    /// point, value = leaf(count)).
    pub fn counters_keys(&self) -> Vec<Value> {
        self.counters.values().map(|(k, _)| k.clone()).collect()
    }

    /// `:rust::telemetry::WorkUnit::durations-keys wu` — the original
    /// key Values for every duration name that ever had a sample
    /// appended. Slice 4's ship walker pairs this with
    /// `WorkUnit/durations` to emit ONE Event::Metric row PER
    /// SAMPLE — N rows for a name with N samples (CloudWatch
    /// model). metric_value stays a primitive HolonAST leaf this
    /// way; Bundle/operator-tag preservation in NoTag (per arc 086)
    /// never enters the picture.
    pub fn durations_keys(&self) -> Vec<Value> {
        self.durations.values().map(|(k, _)| k.clone()).collect()
    }

    /// `:rust::telemetry::WorkUnit::tags wu` — the immutable tag map
    /// declared at `new()`. Returns the same `Value::wat__std__HashMap`
    /// that the constructor was passed; wat-side code reads it
    /// natively via `:wat::core::get`, `:wat::core::keys`, etc. The
    /// ship walker pulls this once per row to attach the
    /// HashMap<HolonAST,HolonAST> as the row's queryable
    /// EDN-map TEXT column.
    pub fn tags(&self) -> Value {
        self.tags.clone()
    }

    /// `:rust::telemetry::WorkUnit::uuid wu` — returns the scope's
    /// canonical hex uuid. Read-only; cloned to a fresh String so
    /// the caller can carry it freely.
    pub fn uuid(&self) -> String {
        self.uuid.clone()
    }

    /// `:rust::telemetry::WorkUnit::incr wu name` — bumps
    /// `counters[name]` by 1. If the key is absent, initializes
    /// to 1 with the original-key-Value stored alongside.
    ///
    /// Panics if `name` is not a hashable Value (lambda, channel,
    /// opaque handle). HolonAST + primitives all hash; this is the
    /// arc-057 contract.
    pub fn incr(&mut self, name: Value) {
        let key = hashmap_key(":rust::telemetry::WorkUnit::incr", &name).unwrap_or_else(|_| {
            panic!(
                ":rust::telemetry::WorkUnit::incr: name must be a hashable Value \
                 (primitive or HolonAST); got {}",
                name.type_name()
            )
        });
        let entry = self.counters.entry(key).or_insert_with(|| (name.clone(), 0));
        entry.1 += 1;
    }

    /// `:rust::telemetry::WorkUnit::append-dt wu name secs` — appends
    /// `secs` to `durations[name]`. Initializes to a single-element
    /// Vec on first append for a given key.
    pub fn append_dt(&mut self, name: Value, secs: f64) {
        let key = hashmap_key(":rust::telemetry::WorkUnit::append-dt", &name).unwrap_or_else(|_| {
            panic!(
                ":rust::telemetry::WorkUnit::append-dt: name must be a hashable Value \
                 (primitive or HolonAST); got {}",
                name.type_name()
            )
        });
        let entry = self
            .durations
            .entry(key)
            .or_insert_with(|| (name.clone(), Vec::new()));
        entry.1.push(secs);
    }

    /// `:rust::telemetry::WorkUnit::counter wu name` — returns the
    /// current count for `name`, or 0 if the key is absent. The
    /// "absent → 0" default is intentional: callers that want
    /// presence-aware behavior can pair with `counters-keys`.
    pub fn counter(&self, name: Value) -> i64 {
        let key = hashmap_key(":rust::telemetry::WorkUnit::counter", &name).unwrap_or_else(|_| {
            panic!(
                ":rust::telemetry::WorkUnit::counter: name must be a hashable Value \
                 (primitive or HolonAST); got {}",
                name.type_name()
            )
        });
        self.counters.get(&key).map(|(_, n)| *n).unwrap_or(0)
    }

    /// `:rust::telemetry::WorkUnit::durations wu name` — returns a
    /// cloned Vec of the duration samples for `name`. Empty Vec
    /// for absent keys. Slice 4's ship walker iterates this at
    /// scope-end to build the metric rows.
    pub fn durations(&self, name: Value) -> Vec<f64> {
        let key = hashmap_key(":rust::telemetry::WorkUnit::durations", &name).unwrap_or_else(|_| {
            panic!(
                ":rust::telemetry::WorkUnit::durations: name must be a hashable Value \
                 (primitive or HolonAST); got {}",
                name.type_name()
            )
        });
        self.durations
            .get(&key)
            .map(|(_, v)| v.clone())
            .unwrap_or_default()
    }
}
