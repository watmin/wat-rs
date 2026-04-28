# Arc 077 — Kill the dim router; one program-d, capacity at the call site

**Status:** PROPOSED 2026-04-28. Pre-implementation reasoning artifact.

**Predecessors:**
- Arc 014 — `set-dim-router!` (the wat lambda primitive that user-customizes the per-form d).
- Arc 053 — `Reckoner.observe / predict` typed against the routed d.
- Arc 056 — Router module + ambient on `SymbolTable` + encoder sites.
- Arc 067 — flat default dim router (`DEFAULT_TIERS = [10000]`). The simplification that quietly retired the multi-tier story; arc 077 closes it.
- Arc 076 — therm-routed Hologram. Hologram/make and filter factories read d from `:wat::config::dims` ambient; capacity = `floor(sqrt(d))`. The router ambient was already collapsing into a single value here; arc 077 makes that explicit.

**Surfaced by:** Arc 076 mid-build (2026-04-28). After dropping `pos` from Hologram and discovering capacity could be derived from d-at-construction, the dim router's role collapsed to "return DEFAULT_TIERS[0]." It always did. From the conversation:

> "this dim router thing... has been a strange concept that i didn't want to remove..."
>
> "i think we've come full circle... we need the user to choose their dims for their program and we derive the capacity for this..."
>
> "the router can finally be killed... the program runs in one dimension size."
>
> "config holds dim-count and dim-capacity as ambient values."

The recognition: real wat programs run at one d. The router was a place-holder for a multi-tier story that never materialized; once arc 067 collapsed `DEFAULT_TIERS` to `[10000]`, the trait, the picker, and the per-d encoder cache became overhead with no behavioral benefit. Arc 077 retires the infrastructure and replaces it with one user-visible knob: **`(:wat::config::set-dim-count! n)` declares the program's d once; capacity = `floor(sqrt(n))` derives at that call.** Both are ambient; everything reads them.

---

## What this arc is, and is not

**Is:** a substrate teardown + restoration:
- Drop `dim_router.rs` entirely (or shrink to a constant accessor).
- Drop the `DimRouter` trait, the per-form `pick(ast, sym)` dispatch, the multi-tier `EncoderRegistry` per-d caching.
- Restore `(:wat::config::set-dim-count! n)` — the wat-side primitive that sets the program-d.
- Add `(:wat::config::dim-capacity)` — reads the derived capacity (`floor(sqrt(dims))`).
- Repoint every router consumer (`require_dim_router`, `router.pick(...)`, the 11 sites in `runtime.rs`) at the single ambient.

**Is not:**
- A re-introduction of multi-tier dims. The path forward is one d per program; if a future workload genuinely needs multiple, that's a separate arc that re-specs the surface.
- A change to `:wat::config::*` semantics for unrelated keys (`global-seed`, `noise-floor`). Those stay.
- A change to per-form encoding behavior. Same encoder, same atom→vector mapping, just no per-d cache layer.

---

## The shift

### Before (arc 056 / 067 era)

```
SymbolTable
  └── encoding_ctx
        ├── encoders: EncoderRegistry  (HashMap<usize, EncoderPair>)
        └── config:   EncodingConfig
  └── dim_router: Arc<dyn DimRouter>
                   └── pick(ast, sym) -> Option<usize>
```

Every cosine / coincident? / encode-side primitive does:

```rust
let router = require_dim_router(OP, sym)?;
let d = router.pick(&ast, sym).ok_or(...)?;
let ctx = require_encoding_ctx(OP, sym)?;
let enc = ctx.encoders.get(d);
let v = encode(&ast, &enc.vm, &enc.scalar);
```

Three indirections per call. The router always returns `DEFAULT_TIERS[0]` in practice.

### After (arc 077)

```
SymbolTable
  └── encoding_ctx
        ├── encoder: EncoderPair          (one, not a map)
        ├── dims:    usize                (the program-d)
        ├── capacity: usize               (floor(sqrt(dims)))
        └── config:  EncodingConfig
```

Same call becomes:

```rust
let ctx = require_encoding_ctx(OP, sym)?;
let v = encode(&ast, &ctx.encoder.vm, &ctx.encoder.scalar);
```

One indirection. `(:wat::config::dim-count)` reads `ctx.dims`; `(:wat::config::dim-capacity)` reads `ctx.capacity`; `Hologram/make filter` reads both via `ctx`.

### `set-dim-count!` semantics

```scheme
(:wat::config::set-dim-count! 10000)   ; program declares d=10000
;; — the substrate computes capacity = floor(sqrt(10000)) = 100
;; — the substrate (re)builds the encoder pair at d=10000
;; — `:wat::config::dims` returns 10000
;; — `:wat::config::capacity` returns 100
;; — every Hologram/make from now on uses capacity=100
;; — every cosine readout encodes at d=10000
```

Call-once-at-startup; calling it again rebuilds the encoder (test scaffolds use this). Failure mode for `n <= 0`: `RuntimeError::InvalidArgument`.

`(:wat::config::set-dim-count! n)` and `(:wat::config::dim-count)` aren't new names — they're a return of the pre-arc-067 surface. Arc 067 ripped them in favor of `DEFAULT_TIERS`; arc 077 admits that was the wrong direction and brings them back. The semantics are simpler this time: one tier, no tier-router lambda, no `Config.dims` field-vs-router split.

---

## API summary

### Killed

| Surface | Replacement |
|---|---|
| `:wat::dim::set-dim-router!` | gone — single d, no router |
| `Arc<dyn DimRouter>` trait + `SizingRouter` impl | gone — `EncodingCtx` carries d directly |
| `EncoderRegistry` (HashMap<d, EncoderPair>) | gone — `EncodingCtx` carries one `EncoderPair` |
| `router.pick(ast, sym)` (11 call sites in runtime.rs) | replaced by `ctx.dims` |
| `RuntimeError::DimUnresolvable` | gone — d always resolvable from ambient |
| `RuntimeError::NoDimRouter` | gone — there's no router |

### Restored

| Surface | Behavior |
|---|---|
| `(:wat::config::set-dim-count! n)` | sets `ctx.dims = n`; recomputes `ctx.capacity = floor(sqrt(n))`; rebuilds `ctx.encoder`. Returns `:()`. |
| `(:wat::config::dim-count)` | reads `ctx.dims`; nullary. |
| `(:wat::config::dim-capacity)` | reads `ctx.capacity`; nullary. New surface; the canonical "what's my Hologram slot count." |

### Unchanged

| Surface | |
|---|---|
| `(:wat::config::global-seed)` | reads `ctx.config.global_seed` |
| `(:wat::config::noise-floor)` | reads `1/sqrt(ctx.dims)` (now off the single d) |
| `Hologram/make filter`, `Hologram/{put,get,len,capacity}` | unchanged shape; just reads from new ambient |
| `filter-coincident`, `filter-present`, `filter-accept-any` | unchanged shape; read from new ambient |
| All cosine / coincident? / presence? / encode primitives | unchanged shape; one less indirection internally |

---

## What changes in `EncodingCtx`

**Before (arc 056 era):**

```rust
pub struct EncodingCtx {
    pub encoders: EncoderRegistry,    // HashMap<usize, EncoderPair>
    pub config:   EncodingConfig,
}
```

**After (arc 077):**

```rust
pub struct EncodingCtx {
    pub encoder:  EncoderPair,        // one pair, built at d=dims
    pub dims:     usize,              // user's pick (default DEFAULT_DIMS=10000)
    pub capacity: usize,              // floor(sqrt(dims)), cached
    pub config:   EncodingConfig,
}
```

`set-dim-count!` rebuilds `encoder` and updates `dims` + `capacity`. The whole struct is `Arc`-shared and lives behind a `ThreadOwnedCell` per existing pattern; mutation goes through the cell's `with_mut`.

---

## Slice plan

### Slice 1 — `EncodingCtx` shape change

Touches `src/encoding_ctx.rs` (or wherever the struct lives), `src/freeze.rs` (the freeze surface), `src/runtime.rs` (constructor sites), `src/lib.rs` (re-exports). Lift the multi-tier registry out, drop in the single encoder + dims + capacity. Default `dims = DEFAULT_DIMS = 10000`.

### Slice 2 — `set-dim-count!` + `dims` + `capacity` primitives

Touches `src/runtime.rs` dispatch + handlers, `src/check.rs` schemes. `set-dim-count!` validates n > 0, rebuilds encoder, updates ctx. `dims` and `capacity` are read-only accessors on the ctx.

### Slice 3 — repoint router consumers

The 11 `router.pick(...)` call sites in `runtime.rs` (cosine, coincident?, presence?, encode-time vector ops, etc.) all repoint to `ctx.dims`. `require_dim_router` deletes; `RuntimeError::{DimUnresolvable, NoDimRouter}` delete.

### Slice 4 — drop `dim_router.rs`

Delete the module and its references from `lib.rs`. `set-dim-router!` dispatch entry deletes. Any tests against the router specifically delete (the consumer-facing behavior is exercised through cosine/encode tests).

### Slice 5 — wat-side stdlib + tests

`(:wat::config::set-dim-count! n)` becomes the canonical "configure your program" prelude in test files. The `test-capacity-at-d-4096` arc-076 test gets re-enabled with `(:wat::config::set-dim-count! 4096)` then asserting capacity = 64.

### Slice 6 — INSCRIPTION + USER-GUIDE sweep

Document the simplification. The user-guide concurrency / encoding sections lose the "router picks d for your form" complexity. INSCRIPTION records the retirement.

---

## Test strategy

- **T1** — `set-dim-count!` round-trip: set 10000; `dims` returns 10000; `capacity` returns 100. Set 4096; `dims` returns 4096; `capacity` returns 64.
- **T2** — `set-dim-count!` rebuilds encoder: encode the same form before and after `set-dim-count! 4096`, expect distinct vectors (different d = different encoder seeded the same but produces different-length output).
- **T3** — `Hologram/make` after `set-dim-count!`: make → put therm → get; capacity comes from the post-set d. (Re-enables the dropped d=4096 capacity test under the new mechanism.)
- **T4** — `set-dim-count! 0` → RuntimeError::InvalidArgument. Same for negative.
- **T5** — `cosine` / `coincident?` / `presence?` after `set-dim-count!`: same behavior as before, just no router resolution overhead.
- **T6** — `set-dim-count!` is process-global within a SymbolTable lifetime; calling twice replaces the encoder. Tests that need a clean encoder for assertions can `set-dim-count!` at the start of each.

---

## What this arc deliberately does NOT do

- **Bring back multi-tier dims.** The user's framing is explicit: one program runs at one d. If two-d cases surface, that's a future spec; we're not paying carrying cost for it.
- **Change atom→vector seeding.** `global_seed` and the deterministic VM-vector allocation stay. Atom IDs round-trip the same.
- **Change Encoder internals.** `EncoderPair { vm, scalar }` is the same; what changes is that there's just one of them per program instead of one per d.
- **Touch `Bigram` / `Trigram` / `Reckoner` / `Engram` etc.** Those consume the encoder via the same `ctx`; signature unchanged.

---

## What this unblocks

- **Arc 076's wat-test for d=4096 returns** under the canonical `set-dim-count!` surface. The Rust unit test stays as the always-on guardrail; the wat test demonstrates user-facing usage.
- **Lab umbrella 059** simplifies — no per-form pos, no per-form router pick, one d, one capacity. Trader code drops several adapter layers.
- **Cross-domain consumers (MTG, truth-engine)** lose the "what d does this pick" question. They set their dim once and write code against one ambient.
- **Documentation** loses the multi-tier story everywhere it appears. Reduces what new readers have to model.

---

## Open questions

### Q1 — Where does `EncodingCtx` get its initial `dims`?

Default `DEFAULT_DIMS = 10000` baked at `EncodingCtx::new()`. User overrides via `(:wat::config::set-dim-count! n)` at program start. Tests do the same.

This mirrors how `global-seed` already works: there's a default (42), and the user's allowed to override it. Same pattern: substrate carries a sane default, user has a knob, no required ceremony before using the substrate.

Add `(:wat::config::set-global-seed! n)` if it isn't already a primitive — the symmetry is worth the consistency. (Probably out of scope for this arc; flag for follow-up if the surface doesn't already support it.)

### Q2 — Does `set-dim-count!` rebuild atom IDs?

No. `global_seed` drives atom seeding; `dims` only affects vector length. Same atom IDs across `set-dim-count!` calls; vectors get longer/shorter. Reckoner / Engram observations made before a `set-dim-count!` are NOT portable to the new d (this matches existing behavior — vector portability arc 061 only handles cross-machine / cross-process round-trip at the same d).

### Q3 — Is `set-dim-count!` idempotent at the same value?

Yes. `set-dim-count! 10000` then `set-dim-count! 10000` again is a no-op; the encoder rebuilds redundantly but the deterministic seeding produces the same encoder. No external observable difference.

### Q4 — What happens when consumers run `set-dim-count!` mid-flight?

The user's code is responsible for ordering. The substrate makes no guarantees about Hologram instances built at the previous d after a `set-dim-count!` — those instances have their own `capacity` baked in at construction; their cosine readouts still use the EncodingCtx's current encoder. That can be inconsistent (a Hologram at capacity=100 looking up against an encoder at d=4096), but it's a caller bug. The substrate doesn't try to migrate.

The right pattern: `set-dim-count!` ONCE at program start. Tests that need different d construct a fresh test harness.

### Q5 — Does `set-dim-count!` belong in a `freeze` or `prelude` phase?

Not necessarily. The current arc adds it as a runtime call. A future "freeze d at startup, refuse later changes" mode could be added as a separate arc if real workloads want it. For now, keep it dynamic — it matches how tests and the trader actually use it.

---

## Risks

- **Test churn.** Every test that touched the router or assumed multi-tier behavior changes. Most are invisible (the router was only ever returning DEFAULT_TIERS[0]); a few that explicitly tested the router lambda surface will need to delete.
- **Frozen state implications.** `freeze.rs` includes `dim_router` in the freeze surface. Removing the router from freeze means the freeze format changes (smaller, simpler). Migration path: any frozen state from pre-arc-077 doesn't load on the new substrate. Acceptable since freeze is not a long-term storage format yet.
- **Atom-ID coherence.** `global_seed` controls atom IDs across encoder rebuilds. Verifying that `set-dim-count!` doesn't perturb atom IDs is T-something in the test plan; if it DID perturb them, two Hologram instances built at different d couldn't share keys.

---

## Slice 1 acceptance

- `EncodingCtx` rebuilt with single encoder + dims + capacity fields.
- Existing tests (cosine, encode, etc.) still green; the substrate is API-identical for non-router-aware callers.
- `cargo test --workspace` green.

PERSEVERARE.
