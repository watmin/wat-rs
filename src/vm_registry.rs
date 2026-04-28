//! Per-dim encoder registry.
//!
//! Arc 037 slice 3 / arc 077 — registry was originally per-router-d;
//! arc 077 retired the router so the registry now only ever holds
//! one entry, the encoder at the program's `dim_count` (carried on
//! `EncodingCtx`). The HashMap shape stays as a small caching detail.
//!
//! [`EncoderRegistry`] wraps the upstream types (`holon::VectorManager`,
//! `holon::ScalarEncoder`) in a lazy HashMap keyed by d, all sharing
//! the same `global_seed`. Same atom at d=256 produces a different-
//! but-deterministic vector than at d=4096; the registry ensures
//! every node in a distributed cloud agrees on both.
//!
//! [`Encoders`] holds the (VM, Scalar) pair at a single d — the unit
//! the `holon::encode` function consumes. `registry.get(d)` returns
//! an `Arc<Encoders>` — cheap to clone, shared across the enterprise.

use holon::{ScalarEncoder, VectorManager};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

/// The (VM, ScalarEncoder) pair at a single dim, plus lazy-computed
/// floors. `holon::encode` consumes vm + scalar alongside the
/// dim-agnostic AtomTypeRegistry; `presence?` / `coincident?` read
/// the floors via [`Encoders::presence_floor`] /
/// [`Encoders::coincident_floor`], which consult the ambient
/// sigma-fn capabilities on [`crate::runtime::SymbolTable`] on first
/// access and memoize via `OnceLock`.
///
/// Arc 037 slice 6: floors are functions of both d and the
/// user-configurable sigma function. They're computed on first
/// presence? / coincident? call at this d (one wat-lambda invocation
/// per tier per sigma-setter, max) and cached thereafter as a simple
/// field load. O(tiers) sigma invocations ever.
pub struct Encoders {
    pub vm: VectorManager,
    pub scalar: ScalarEncoder,
    /// The encoding d this pair serves.
    pub dims: usize,
    /// Lazily-computed `presence_sigma_fn(d) / sqrt(d)`. Populated
    /// on first call to [`Encoders::presence_floor`].
    presence_floor_cache: std::sync::OnceLock<f64>,
    /// Lazily-computed `coincident_sigma_fn(d) / sqrt(d)`.
    coincident_floor_cache: std::sync::OnceLock<f64>,
}

impl Encoders {
    /// Presence-floor at this d: `σ(d) / sqrt(d)`, where σ is
    /// determined by the ambient `presence_sigma_fn` on `sym`.
    /// Memoized per (Encoders, d).
    pub fn presence_floor(&self, sym: &crate::runtime::SymbolTable) -> f64 {
        *self.presence_floor_cache.get_or_init(|| {
            let sigma = sym
                .presence_sigma_fn()
                .map(|f| f.sigma_at(self.dims, sym))
                .unwrap_or(1);
            (sigma as f64) / (self.dims as f64).sqrt()
        })
    }

    /// Coincident-floor at this d: `σ(d) / sqrt(d)`, where σ is
    /// determined by the ambient `coincident_sigma_fn` on `sym`.
    pub fn coincident_floor(&self, sym: &crate::runtime::SymbolTable) -> f64 {
        *self.coincident_floor_cache.get_or_init(|| {
            let sigma = sym
                .coincident_sigma_fn()
                .map(|f| f.sigma_at(self.dims, sym))
                .unwrap_or(1);
            (sigma as f64) / (self.dims as f64).sqrt()
        })
    }
}

impl std::fmt::Debug for Encoders {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Encoders")
            .field("dims", &self.dims)
            .field("presence_floor_cached", &self.presence_floor_cache.get())
            .field("coincident_floor_cached", &self.coincident_floor_cache.get())
            .finish()
    }
}

/// Lazy per-dim registry of encoder pairs. All instances share the
/// same `global_seed`, so the same atom at the same d produces the
/// same vector on every node in the cloud.
///
/// Typical usage inside the runtime: the ambient dim router picks a
/// d per construction; this registry is consulted for the
/// [`Encoders`] at that d; `holon::encode` materializes the vector.
/// Subsequent calls at the same d are free — the upstream types hold
/// their own atom-vector caches internally.
pub struct EncoderRegistry {
    global_seed: u64,
    encoders: RwLock<HashMap<usize, Arc<Encoders>>>,
}

impl EncoderRegistry {
    /// Build an empty registry with the given seed. No encoders are
    /// materialized until [`EncoderRegistry::get`] is called.
    pub fn new(global_seed: u64) -> Self {
        Self {
            global_seed,
            encoders: RwLock::new(HashMap::new()),
        }
    }

    /// The seed every encoder in this registry shares.
    pub fn global_seed(&self) -> u64 {
        self.global_seed
    }

    /// Fetch (or lazily build) the [`Encoders`] at `dims`. Returns an
    /// `Arc<Encoders>` that can be cheaply cloned and stored.
    pub fn get(&self, dims: usize) -> Arc<Encoders> {
        // Read path: cache hit → clone Arc and return.
        {
            let map = self.encoders.read().unwrap();
            if let Some(enc) = map.get(&dims) {
                return Arc::clone(enc);
            }
        }

        // Miss: build under the write lock. Re-check in case another
        // thread beat us to the insert (birthday concurrency).
        let mut map = self.encoders.write().unwrap();
        if let Some(enc) = map.get(&dims) {
            return Arc::clone(enc);
        }
        let enc = Arc::new(Encoders {
            vm: VectorManager::with_seed(dims, self.global_seed),
            scalar: ScalarEncoder::with_seed(dims, self.global_seed),
            dims,
            presence_floor_cache: std::sync::OnceLock::new(),
            coincident_floor_cache: std::sync::OnceLock::new(),
        });
        map.insert(dims, Arc::clone(&enc));
        enc
    }

    /// How many distinct dims have been materialized so far.
    /// Useful for diagnostics / tests.
    pub fn size(&self) -> usize {
        self.encoders.read().unwrap().len()
    }
}

impl std::fmt::Debug for EncoderRegistry {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let dims: Vec<usize> = self
            .encoders
            .read()
            .unwrap()
            .keys()
            .copied()
            .collect();
        f.debug_struct("EncoderRegistry")
            .field("global_seed", &self.global_seed)
            .field("materialized_dims", &dims)
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn registry_is_empty_on_construction() {
        let reg = EncoderRegistry::new(42);
        assert_eq!(reg.size(), 0);
    }

    #[test]
    fn get_materializes_lazily() {
        let reg = EncoderRegistry::new(42);
        let _a = reg.get(256);
        assert_eq!(reg.size(), 1);
        let _b = reg.get(4096);
        assert_eq!(reg.size(), 2);
    }

    #[test]
    fn repeated_get_at_same_d_returns_shared_arc() {
        let reg = EncoderRegistry::new(42);
        let a = reg.get(1024);
        let b = reg.get(1024);
        assert!(Arc::ptr_eq(&a, &b), "same d should return shared Arc");
        assert_eq!(reg.size(), 1);
    }

    #[test]
    fn encoders_have_matching_dims() {
        let reg = EncoderRegistry::new(42);
        let enc = reg.get(4096);
        assert_eq!(enc.vm.dimensions(), 4096);
    }

    #[test]
    fn shared_seed_across_dims() {
        let reg = EncoderRegistry::new(1337);
        let a = reg.get(256);
        let b = reg.get(4096);
        assert_eq!(a.vm.global_seed(), 1337);
        assert_eq!(b.vm.global_seed(), 1337);
    }

    #[test]
    fn same_seed_same_atom_at_same_d_yields_same_vector() {
        // Cross-node determinism: two registries with the same seed
        // produce the same vector for the same atom at the same d.
        let r1 = EncoderRegistry::new(42);
        let r2 = EncoderRegistry::new(42);
        let v1 = r1.get(1024).vm.get_vector("alice");
        let v2 = r2.get(1024).vm.get_vector("alice");
        assert_eq!(v1, v2);
    }

    #[test]
    fn same_seed_same_atom_at_different_d_yields_different_vectors() {
        // The whole point: a given atom's vector depends on d. At d=256
        // it's a 256-dim vector; at d=4096 it's a different 4096-dim
        // vector; cosine across them requires re-projection.
        let reg = EncoderRegistry::new(42);
        let v_small = reg.get(256).vm.get_vector("alice");
        let v_big = reg.get(4096).vm.get_vector("alice");
        assert_eq!(v_small.dimensions(), 256);
        assert_eq!(v_big.dimensions(), 4096);
    }
}
