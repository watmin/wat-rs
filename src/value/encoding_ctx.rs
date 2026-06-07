//! Program-wide encoding context — EncodingCtx carries the frozen dim, capacity, and encoder registry for the lifetime of a running program.
use crate::config::Config;
use crate::vm_registry::EncoderRegistry;
use std::fmt;
use std::sync::Arc;

/// Arc 077 — program-wide encoding context.
///
/// Holds `Arc`s so it can be cloned cheaply by the runtime when a
/// primitive needs encoding access; the underlying `VectorManager` and
/// `ScalarEncoder` are pure caches that can be shared across threads.
#[derive(Clone)]
pub struct EncodingCtx {
    /// Per-dim encoder registry. Arc 037-era multi-tier shape; arc 077
    /// retires the multi-tier story but keeps the registry as the
    /// underlying encoder cache so consumers transition incrementally.
    /// In the new world, the registry only ever holds one entry — the
    /// one at `dim_count`.
    pub encoders: Arc<EncoderRegistry>,
    /// Arc 077 — the program's encoding dim. Read from
    /// `Config.dim_count` at freeze; same value for the whole program
    /// lifetime. All encoder lookups go to `encoders.get(dim_count)`.
    pub dim_count: usize,
    /// Arc 077 — capacity of any `:wat::holon::Hologram` constructed
    /// in this program: `floor(sqrt(dim_count))`. Cached at
    /// construction.
    pub capacity: usize,
    // rune:solvere(load-bearing-coupling) — Config on EncodingCtx is the sole inherited-config carrier through SymbolTable into spawned sub-programs; 5 spawn-driver sites (fork.rs×2, spawn.rs, spawn_process.rs×2) read ctx.config. Coupled by the config-inheritance design; any decoupling consolidates in the spawn/ home (docs/arc/2026/06/251-types-as-forms/SCOUT-LIFT-MAP.md), not before it lifts.
    pub config: Config,
}

impl EncodingCtx {
    /// Build an encoding context from the frozen [`Config`].
    ///
    /// Per arc 057 the `AtomTypeRegistry` is gone — primitives ARE
    /// HolonAST (typed leaves), so the dyn-Any payload registry that
    /// once dispatched on `Atom(Arc<dyn Any>)` no longer has work to do.
    pub fn from_config(cfg: &Config) -> Self {
        let dim_count = cfg.dim_count;
        let capacity = ((dim_count as f64).sqrt().floor() as usize).max(1);
        EncodingCtx {
            encoders: Arc::new(EncoderRegistry::new(cfg.global_seed)),
            dim_count,
            capacity,
            config: cfg.clone(),
        }
    }

    /// The `Encoders` (vm + scalar) at this program's dim. Replaces
    /// arc-074-era `ctx.encoders.get(d)` once the d came from the
    /// router; arc 077 makes the dim ambient.
    pub fn encoder(&self) -> Arc<crate::vm_registry::Encoders> {
        self.encoders.get(self.dim_count)
    }
}

impl fmt::Debug for EncodingCtx {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("EncodingCtx")
            .field("global_seed", &self.config.global_seed)
            .field("dim_count", &self.dim_count)
            .field("capacity", &self.capacity)
            .finish()
    }
}
