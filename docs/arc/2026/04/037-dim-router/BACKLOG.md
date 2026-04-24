# Arc 037 — dim-router — BACKLOG

**Shape:** eight slices across three migration phases. No
breaking changes during transition. See DESIGN.md for the
full migration plan.

---

## Phase 1 — Add new, keep old (backward compat)

Slices 1 – 5. Zero-config works. Explicit-config works.
Nothing breaks.

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
  every `ctx.config.dims` read with a call to the sizing
  function using the AST's immediate shape. Each construction
  picks its own dim.

- **`config.dims` field**: stays in Config struct for backward
  compat parsing. NOTHING reads it on the encoder path. Its
  value is stored when `set-dims!` is called but has no runtime
  effect on vector dim selection.

- **`config.capacity_mode`**: stays functional with default
  `CapacityMode::Error`. User override via `set-capacity-mode!`.
  Permanent — not migrating out.

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
- **1a** — Where the sizing function's output gets STORED per
  HolonAST. Options: (a) attach d to each HolonAST variant;
  (b) ctx-scoped; (c) per-cache-entry. Decide at slice start.
- **1b** — What "immediate_size" means for each AST variant.
  Leaf atom: 1. Bundle: `args.len()`. Bind: 2 (or 1, treat as
  opaque composite?). Needs a small policy table.
- **1c** — Holon-rs vs wat-rs boundary. The actual vector
  encoding (bit generation from the seed + dim + ast hash)
  lives in holon-rs. wat-rs Atom/Bundle construction sites
  are where dim gets DECIDED. Need to audit the interface.

**Scope**: multi-file. `src/config.rs` + `src/runtime.rs` +
possibly `holon-rs`. Not a 50-line change. Medium risk due to
encoder-path change. High validates — proves the sizing-
function model works end-to-end.

### Slice 2 — User-configurable tier list

**Status: ready after slice 1.**

- Add `set-dim-tiers!(list)` setter.
- Config stores the user-supplied tier list; sizing function
  reads from config instead of the Rust constant.
- Default tier list still `[256, 4096, 10000, 100000]` when
  unset.

**Sub-fogs:**
- **2a** — setter syntax for list-of-ints. Variadic:
  `(:wat::config::set-dim-tiers! 256 4096 10000 100000)` —
  matches other config setters.
- **2b** — validation: non-empty, all positive, sorted ascending.

**Scope**: `config.rs` parser + tests.

### Slice 3 — Cosine validates matching d

**Status: ready after slice 2.**

- `cosine`, `presence?`, `coincident?`: require both operands
  at matching d. Mismatch → error (per capacity-mode) or
  `false`.
- Existing tests at single d continue to pass.

**Scope**: holon op sites in runtime.rs.

### Slice 4 — `set-dim-router!` (user-supplied lambda)

**Status: ready after slice 3.**

- New setter takes a wat function (alongside or instead of the
  tier list). Config stores user's lambda or a reference.
- Default router remains the built-in sizing function.
- At each Atom/Bundle construction, invoke the active router
  (user's lambda if set; built-in otherwise).

**Sub-fogs:**
- **4a** — user lambda eval performance. Mitigation: memoize
  by AST hash (cache tier-index per AST).
- **4b** — invoking user code from Atom/Bundle construction.
  Via SymbolTable's Environment (capability-carrier pattern,
  Chapter 12's feedback memory).

**Scope**: config.rs + runtime integration.

## Phase 2 — Migrate callers

Slice 5. Each file migrates independently.

### Slice 5 — Sweep existing callers

**Status: ready after Phase 1.**

- Grep all `set-dims!` / `set-capacity-mode!` call sites.
- Remove the ones matching defaults (d=10000, mode=:error).
- Migrate non-default callers to `set-dim-router! [d]` or
  `set-dim-router! [d] custom-fn`.
- `set-capacity-mode!` calls stay where users want non-default
  mode.
- Test files mostly shift from explicit setters to zero-config.

**Scope**: mechanical sweep, 50+ files. Per-file migration;
no all-at-once rewrite.

## Phase 3 — Remove dead primitive

Slices 6 – 7.

### Slice 6 — Remove `set-dims!` primitive

**Status: ready after Phase 2 complete.**

- `grep set-dims!` returns zero hits → safe to remove.
- Remove parser arm in `config.rs`.
- Remove `Config.dims` as a primary field? Or keep as derived
  "primary tier" shorthand? Decision at slice time.
- `:wat::config::dims` accessor: retire or keep as
  "primary dim" shorthand (reads `tier_list[0]` or `default
  tier`).
- `set-capacity-mode!` STAYS. Not removed. Legitimate user
  override.

**Scope**: small removals + one decision point (keep or
remove `:wat::config::dims` accessor).

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
