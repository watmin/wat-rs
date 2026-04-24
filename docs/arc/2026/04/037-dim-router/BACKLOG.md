# Arc 037 — dim-router — BACKLOG

**Shape:** seven slices across three migration phases (slice 2
retired 2026-04-24). No breaking changes during transition.
See DESIGN.md for the full migration plan.

**2026-04-24 correction:** `set-dim-router!` takes ONE arg
(`router-fn`), not two. Tier list is the router's closure data,
not a separate config slot. `CapacityMode` enum reduces to two
variants (`:error`, `:abort`). `:silent` and `:warn` retire.

**2026-04-24 scope expansion:** Slice 6 extends from "rip dims"
to "every substrate default is a function, user replaces with
their own function." Scalar sigma setters retire in favor of
function-valued setters mirroring `set-dim-router!`. Derived
Config fields (`noise_floor`, `presence_floor`,
`coincident_floor`) and their accessors rip as cascade.
`Encoders` gains lazy per-d floor memoization.

---

## Phase 1 — Add new, keep old (backward compat)

Slices 1, 3, 4 (slice 2 retired). Zero-config works.
Explicit-config works. Nothing breaks.

### Slice 1 — Sizing function as the substrate default

**Status: ready.**

**Corrected 2026-04-23 late-session.** The prior slice 1 scope
("make dims optional with default 10000") preserved the magic
value arc 037 is killing. The correct slice eliminates the
single-dim consultation from encoders entirely. There is NO
"default dim number" after slice 1 — just the sizing function.

What ships:

- **Sizing function module**: `src/dim_router.rs` (or similar).
  Exposes a pure Rust fn:
  ```
  default_dim_for_shape(immediate_size: usize, tiers: &[usize]) -> Option<usize>
  ```
  Returns the smallest tier d where `sqrt(d) >= immediate_size`.
  `None` when no tier fits (caller decides per capacity-mode).

- **Default tier list** as a Rust constant:
  `[256, 4096, 10000, 100000]`.

- **Atom / Bundle encoder sites in `src/runtime.rs`**: replace
  every `ctx.config.dims` read with a call to the ambient
  router via `SymbolTable`'s Environment (Ch 12 capability-
  carrier pattern). Each construction queries the router with
  its own immediate shape; router returns d for THIS
  construction. **HolonAST stays dim-agnostic** — no d field.
  Cache keys by `(ast-hash, d)`. Same AST, many cache entries,
  many d's.

- **`config.dims` field**: stays in Config struct for backward
  compat parsing. NOTHING reads it on the encoder path. Its
  value is stored when `set-dims!` is called but has no runtime
  effect on vector dim selection.

- **`config.capacity_mode`**: stays functional with default
  `CapacityMode::Error`. User override via `set-capacity-mode!`.
  Permanent — not migrating out.

- **`CapacityMode` enum reduces to two variants**:
  `CapacityMode::Error` (returns `Err(CapacityExceeded)`) and
  `CapacityMode::Abort` (panics). Retire `Silent` and `Warn` —
  overflow either crashes or is handled; no middle ground.
  Sweep parser arms, runtime dispatch, and any callers.

- **Required-field check retires** for `dims` and
  `capacity-mode`. Both become optional; `collect_entry_file`
  no longer returns `RequiredFieldMissing` for them.

Tests expected to flip:
- Tests asserting `RequiredFieldMissing { field: "dims" }` or
  `"capacity-mode"` — flip to assert Config commits with
  defaults.
- Tests that encode a Bundle of N items and hand-check
  specific-d-derived values (e.g., noise_floor at d=1024) —
  these may fail because the encoding dim is now chosen by the
  sizing function on N, not by config.dims. Expected fallout;
  each flip is a real signal that the test was coupled to the
  magic number.

**Sub-fogs:**
- **1a RESOLVED (2026-04-24)** — AST stays dim-agnostic. Router
  is ambient runtime capability, attaches to `SymbolTable`'s
  Environment (Ch 12 capability-carrier pattern). Atom/Bundle
  construction queries the router. Cache keys by `(ast-hash, d)`
  — same AST has many cache entries at many d's. No d on
  HolonAST variants. Nothing stored per-AST; d is runtime
  context, asked on demand.
- **1b** — What "immediate_size" means for each AST variant.
  Leaf atom: 1. Bundle: `args.len()`. Bind: 2 (or 1, treat as
  opaque composite?). Needs a small policy table.
- **1c** — Holon-rs vs wat-rs boundary. The actual vector
  encoding (bit generation from the seed + dim + ast hash)
  lives in holon-rs. wat-rs Atom/Bundle construction sites
  are where the router is CONSULTED. Need to audit the
  interface.

**Scope**: multi-file. `src/config.rs` + `src/runtime.rs` +
possibly `holon-rs`. Not a 50-line change. Medium risk due to
encoder-path change. High validates — proves the sizing-
function model works end-to-end.

### Slice 2 — RETIRED (2026-04-24)

**Not shipping.** Tier list is not a first-class config concept.
Each router closes over its own tier list; users who want a
different list write their own router. No `set-dim-tiers!`
primitive. No `tier_list` Config field.

Slice numbers below are stable (3 / 4 / 5 / 6 / 7). Slice 2's
slot stays present as an honest record that the concept was
considered and dropped.

### Slice 3 — Cosine / presence / coincident handle cross-dim

**Status: ready after slice 1.**

- `cosine`, `presence?`, `coincident?` check operand dims.
- Match → direct inner product at the shared d.
- Mismatch → **normalize UP**: pick `max(d_a, d_b)`, re-encode
  the smaller operand at the greater d via its AST, cache the
  re-projection at `(ast-hash, d_bigger)`, then inner-product.
  See DESIGN's "Cross-dim operations" section for rationale
  and cost model.
- Never normalize DOWN — larger operand may exceed
  `√d_smaller`; down-normalize would bust capacity-mode.
- Cosine/presence/coincident take holons (AST + cached vector),
  not raw vectors — re-projection requires AST in hand.
- Existing tests at single d continue to pass. New tests cover
  cold-cache re-encode, warm-cache L2 hit, and down-normalize
  refusal.

**Scope**: holon op sites in runtime.rs; cache-key extension
to `(ast-hash, d)`; re-encode path at mismatch site.

### Slice 4 — `set-dim-router!` (user-supplied lambda)

**Status: ready after slice 3.**

- New setter takes a SINGLE wat function: `router-fn`.
- Router signature:
  `fn(item_count: :i64) -> :Option<:i64>`.
  `Some d` → pick dim d for this construction.
  `:None` → no tier fits; substrate dispatches per
  `capacity-mode` (`:error` returns Err; `:abort` panics).
- Config stores user's lambda. Default is the built-in sizing
  function (closes over `[256, 4096, 10000, 100000]`).
- At each Atom/Bundle construction, invoke the active router
  (user's lambda if set; built-in otherwise).
- User routers bring their own tier list inside the closure;
  no separate tier-list parameter.

**Sub-fogs:**
- **4a** — user lambda eval performance. Mitigation: memoize
  by AST hash (cache router output per AST shape).
- **4b** — invoking user code from Atom/Bundle construction.
  Via SymbolTable's Environment (capability-carrier pattern,
  Chapter 12's feedback memory).

**Scope**: config.rs + runtime integration.

## Phase 2 — Migrate callers

Slice 5. Each file migrates independently.

### Slice 5 — Sweep existing callers

**Status: ready after Phase 1.**

- Grep all `set-dims!` / `set-capacity-mode!` call sites.
- Remove the ones matching defaults (dim picked by sizing
  function, mode=:error).
- Migrate non-default callers to `set-dim-router! custom-fn`
  — where `custom-fn` is a lambda that returns the caller's
  desired dim (single-tier or multi-tier; the lambda's
  closure carries whatever tier list it needs).
- `set-capacity-mode!` calls stay where users want `:abort`
  instead of `:error`.
- Any callers passing `:silent` or `:warn` must migrate to
  `:error` or `:abort` — the other two variants retire.
- Test files mostly shift from explicit setters to zero-config.

**Scope**: mechanical sweep, 50+ files. Per-file migration;
no all-at-once rewrite.

## Phase 3 — Remove dead primitive

Slices 6 – 7.

### Slice 6 — Rip `set-dims!` + cascade + function-valued sigmas

**Status: ready after Phase 2 complete.**

**Scope expansion (2026-04-24):** The user correction made clear
that EVERY substrate default we ship is a function, and EVERY
user override replaces our function with theirs. Slice 6 extends
to retire the scalar sigma setters (arc 024 shipped them as
integers; under arc 037 that's wrong because integer-sigma means
different confidence at different d) and replace with function-
accepting setters mirroring `set-dim-router!`.

**Rip:**
- `set-dims!` parser arm.
- `Config.dims` field.
- `:wat::config::dims` accessor — there is no single "dims"
  at runtime; each construction picks its own via the router.
- `Config.noise_floor`, `Config.presence_floor`,
  `Config.coincident_floor` stored fields. All derived from dims;
  without dims they can't compute. Floors are per-d formulae now.
- `:wat::config::noise-floor` accessor. Per-d now, not a global.
- `set-noise-floor!` setter. No field to set.
- `Config.presence_sigma: i64`, `Config.coincident_sigma: i64`
  scalar fields. Scalar sigma is the wrong shape under multi-d.

**Add:**
- `Config.presence_sigma_ast: Option<WatAST>` + setter
  `(:wat::config::set-presence-sigma! <expr>)` — expr must
  evaluate to `:fn(:i64) -> :i64`. Signature-checked at freeze.
- `Config.coincident_sigma_ast: Option<WatAST>` + setter — same
  shape.
- Built-in default `SigmaFn` impls:
  - `DefaultPresenceSigma`: `floor(sqrt(d)/2) - 1` (arc 024's formula).
  - `DefaultCoincidentSigma`: constant 1.
- `SymbolTable` gains `presence_sigma_fn` +
  `coincident_sigma_fn` capability slots (mirrors `dim_router`).
- Freeze installs user-supplied `WatLambdaSigmaFn` if configured,
  else the default.
- `Encoders` gains `presence_floor: OnceLock<f64>` and
  `coincident_floor: OnceLock<f64>` — lazy per-d memoization via
  the ambient sigma function. O(tiers) sigma-fn invocations ever.

**Keep:**
- `capacity_mode`, `global_seed`, `dim_router_ast`.
- `set-capacity-mode!`, `set-global-seed!`, `set-dim-router!`.

**Scope**: medium. Touches runtime (two new capability slots +
invocation via SigmaFn trait), freeze (eval sigma ASTs), config
(parser arms + field rip/add), Encoders (lazy floors), check.rs
(retired accessors + new sigma setter signatures).
**Risk**: medium. Same shape as slice 4's `set-dim-router!` so
the pattern is proven. The scalar-sigma-setter retirement flips
the one wat-tests file that hand-rolled `noise-floor` comparison.

### Slice 7 — INSCRIPTION + doc sweep

**Status: obvious in shape.**

- `docs/arc/2026/04/037-dim-router/INSCRIPTION.md`.
- `docs/README.md` arc list.
- `docs/USER-GUIDE.md` — zero-config section; deprecate
  explicit `set-dims!` in examples.
- `docs/CONVENTIONS.md` — multi-tier + router conventions.
- Lab-side 058 `FOUNDATION-CHANGELOG.md` row.
- Task #53 marked completed.

---

## Working notes

- Opened 2026-04-23 following BOOK ch 36-43 recognition arc.
- **Key constraint**: migration is SAFE and ADDITIVE. No
  breaking changes during Phase 1-2. Old code keeps working
  until Phase 3 ripout.
- **Decision made 2026-04-23**: `set-capacity-mode!` stays
  forever as user override. Only `set-dims!` is migrating
  out.
- **Decision made 2026-04-24**: `set-dim-router!` takes ONE
  arg, not two. Tier list is router-internal; the substrate
  doesn't know about tiers as a first-class config concept.
  Earlier braid-check claim that "tier-list and router-fn are
  orthogonal" was wrong — the router uses its list; they are
  braided by construction.
- **Decision made 2026-04-24**: `CapacityMode` reduces from
  four variants to two. `:error` returns Err; `:abort` panics.
  `:silent` and `:warn` retire. Overflow either crashes or is
  handled — no middle ground.
- **Decision made 2026-04-24**: Router is ambient runtime
  capability on `SymbolTable`'s Environment (Ch 12 pattern),
  not config-stored-and-read-per-call. HolonAST stays
  dim-agnostic; cache keys `(ast-hash, d)`. Same AST has many
  cache entries at many d's. Sub-fog 1a dissolves: nothing
  stored per HolonAST, d asked on demand.
- **Decision made 2026-04-24**: Cross-dim cosine normalizes UP,
  never down. `cosine(a@d1, b@d2)` picks `max(d1,d2)`, re-encodes
  smaller via its AST, caches at new d, inner-products.
  Amortized via shared L2; cold path is Merkle-DAG-bounded.
  Down-normalize is banned (would bust capacity-mode for the
  bigger operand).
- **Decision made 2026-04-24**: Every substrate default is a
  FUNCTION; every user override replaces our function with theirs.
  Three capability carriers on `SymbolTable`: `dim_router`,
  `presence_sigma_fn`, `coincident_sigma_fn`. Defaults shipped
  as built-in Rust impls; user overrides via AST-accepting
  setters (`set-dim-router!`, `set-presence-sigma!`,
  `set-coincident-sigma!`). Scalar sigma setters retire —
  integer-sigma gives different confidence at different d;
  function-of-d is the honest shape.
- **Slice 1 correction (2026-04-23 late)**: the prior slice 1
  scope preserved `dims=10000` as a default literal, which IS
  the magic value arc 037 is killing. The corrected slice 1
  eliminates the single-dim consultation from encoders and
  introduces the sizing function as the substrate default.
  There is NO default dim number; the default is a FUNCTION.
- **Invariant for slice 1 tests**: same AST produces same dim
  (deterministic sizing function). Vector size is determined
  by shape, not by any setter. `set-dims!` is a parseable
  no-op on the encoder path — it stores in config but nothing
  reads it.
- **Start point**: Slice 1 is the smallest change that
  validates the premise. If the defaults don't work, nothing
  else in the arc matters. Slice 1 first.
