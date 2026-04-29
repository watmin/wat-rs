//! `:rust::measure::WorkUnit` — measurement-scope state.
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
use std::time::Instant;

use wat::rust_deps::RustDepsBuilder;
use wat::runtime::{hashmap_key, Value};
use wat_macros::wat_dispatch;

/// Register the `:rust::measure::WorkUnit` shim into the deps
/// builder. Forwards to the macro-generated register fn. Lives in
/// this module (not `lib.rs`) because the macro-generated module
/// is visibility-private to its declaring module — same pattern
/// wat-lru's `shim::register` uses.
pub fn register(builder: &mut RustDepsBuilder) {
    __wat_dispatch_WatMeasureWorkUnit::register(builder);
}

/// The opaque carried as `:rust::measure::WorkUnit` at the wat
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
    /// Wall-clock at scope-open. Used by slice 4's `WorkUnit/scope`
    /// to compute elapsed at scope-end. Slice 3 stores it but
    /// exposes no accessor.
    #[allow(dead_code)]
    started: Instant,
    /// Canonical 8-4-4-4-12 hyphenated v4 UUID. Minted via
    /// `wat_edn::new_uuid_v4()` (arc 092) at construction.
    uuid: String,
}

#[wat_dispatch(
    path = ":rust::measure::WorkUnit",
    scope = "thread_owned"
)]
impl WatMeasureWorkUnit {
    /// `:rust::measure::WorkUnit::new` — fresh scope. New uuid,
    /// `Instant::now()` for `started`, empty maps. The opaque
    /// returned wraps in a `ThreadOwnedCell` (macro `scope =
    /// "thread_owned"`); the cell binds to this thread.
    pub fn new() -> Self {
        WatMeasureWorkUnit {
            counters: HashMap::new(),
            durations: HashMap::new(),
            started: Instant::now(),
            uuid: wat_edn::new_uuid_v4().to_string(),
        }
    }

    /// `:rust::measure::WorkUnit::uuid wu` — returns the scope's
    /// canonical hex uuid. Read-only; cloned to a fresh String so
    /// the caller can carry it freely.
    pub fn uuid(&self) -> String {
        self.uuid.clone()
    }

    /// `:rust::measure::WorkUnit::incr wu name` — bumps
    /// `counters[name]` by 1. If the key is absent, initializes
    /// to 1 with the original-key-Value stored alongside.
    ///
    /// Panics if `name` is not a hashable Value (lambda, channel,
    /// opaque handle). HolonAST + primitives all hash; this is the
    /// arc-057 contract.
    pub fn incr(&mut self, name: Value) {
        let key = hashmap_key(":rust::measure::WorkUnit::incr", &name).unwrap_or_else(|_| {
            panic!(
                ":rust::measure::WorkUnit::incr: name must be a hashable Value \
                 (primitive or HolonAST); got {}",
                name.type_name()
            )
        });
        let entry = self.counters.entry(key).or_insert_with(|| (name.clone(), 0));
        entry.1 += 1;
    }

    /// `:rust::measure::WorkUnit::append-dt wu name secs` — appends
    /// `secs` to `durations[name]`. Initializes to a single-element
    /// Vec on first append for a given key.
    pub fn append_dt(&mut self, name: Value, secs: f64) {
        let key = hashmap_key(":rust::measure::WorkUnit::append-dt", &name).unwrap_or_else(|_| {
            panic!(
                ":rust::measure::WorkUnit::append-dt: name must be a hashable Value \
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

    /// `:rust::measure::WorkUnit::counter wu name` — returns the
    /// current count for `name`, or 0 if the key is absent. The
    /// "absent → 0" default is intentional: callers that want
    /// presence-aware behavior can pair with `counters-keys`.
    pub fn counter(&self, name: Value) -> i64 {
        let key = hashmap_key(":rust::measure::WorkUnit::counter", &name).unwrap_or_else(|_| {
            panic!(
                ":rust::measure::WorkUnit::counter: name must be a hashable Value \
                 (primitive or HolonAST); got {}",
                name.type_name()
            )
        });
        self.counters.get(&key).map(|(_, n)| *n).unwrap_or(0)
    }

    /// `:rust::measure::WorkUnit::durations wu name` — returns a
    /// cloned Vec of the duration samples for `name`. Empty Vec
    /// for absent keys. Slice 4's ship walker iterates this at
    /// scope-end to build the metric rows.
    pub fn durations(&self, name: Value) -> Vec<f64> {
        let key = hashmap_key(":rust::measure::WorkUnit::durations", &name).unwrap_or_else(|_| {
            panic!(
                ":rust::measure::WorkUnit::durations: name must be a hashable Value \
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
