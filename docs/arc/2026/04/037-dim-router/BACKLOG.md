# Arc 037 — dim-router — BACKLOG

**Shape:** eight slices across three migration phases. No
breaking changes during transition. See DESIGN.md for the
full migration plan.

---

## Phase 1 — Add new, keep old (backward compat)

Slices 1 – 5. Zero-config works. Explicit-config works.
Nothing breaks.

### Slice 1 — Optional `dims` + `capacity-mode`

**Status: ready.**

- `Config.dims`: required → optional, default `10000`.
- `Config.capacity_mode`: required → optional, default
  `CapacityMode::Error`.
- `collect_entry_file` no longer returns
  `RequiredFieldMissing` for these fields when absent.
- All derived fields (noise_floor, presence_sigma,
  coincident_sigma) get their default when `dims` is unset.

Tests:
- `blank_entry_file_uses_defaults` — new test: empty entry
  file produces Config with `dims=10000`, `capacity_mode=Error`.
- `existing_set_dims_still_works` — existing tests pass
  unchanged.
- `missing_fields_no_longer_error` — remove/update the tests
  that asserted errors for missing `dims` and `capacity-mode`.

**Sub-fogs:**
- **1a** — some existing tests explicitly assert
  `RequiredFieldMissing { field: "dims" }`. They'll need to
  flip to asserting the default value instead. ~3-5 tests.
- **1b** — `empty_entry_file_rejected` test literally tests
  that empty input errors. Flip to `empty_entry_file_defaults`.

**Scope**: ~50 LoC in `src/config.rs` + test updates.

### Slice 2 — `tier_list` field (backward compat)

**Status: ready after slice 1.**

- Add `Config.tier_list: Vec<usize>` with default
  `vec![256, 4096, 10000, 100000]`.
- `set-dims!(n)` also sets `tier_list = vec![n]` for compat
  semantics.
- Add `set-dim-tiers!(list)` setter — direct list setter that
  doesn't require the router.
- `tier_list` field unused by runtime yet (slice 3 first
  reader).

**Sub-fogs:**
- **2a** — setter syntax for list-of-ints. Options:
  - Variadic: `(:wat::config::set-dim-tiers! 256 4096 10000 100000)`
  - List: `(:wat::config::set-dim-tiers! (list :i64 256 4096 10000 100000))`
  - Pick variadic — matches other config setters' shape; easier to parse.
- **2b** — validation: non-empty, all positive, sorted ascending.

**Scope**: `config.rs` parser + tests.

### Slice 3 — Sizing function + router-driven d

**Status: ready after slice 2.**

- Implement `default_router(ast: &WatAST, tier_list: &[usize]) -> usize`
  — the sizing function.
- Atom / Bundle construction sites call the router for each
  construction, get a tier index, resolve `tier_list[index]`
  as d.
- Bundle's capacity check uses the Atom/Bundle-specific d
  (not `config.dims`).
- With single-tier `tier_list = [10000]` (backward compat
  from `set-dims! 10000`), all constructions route to tier 0
  → d=10000 → identical behavior.

**Sub-fogs:**
- **3a** — Atom top-level "size": leaves = 1; bundle-wrapping-atoms = inner count.
- **3b** — Bundle's immediate size = `args.len()` at construction.
- **3c** — where to store the chosen d per HolonAST? Options:
  - Field on HolonAST — invasive change.
  - Store alongside in cache entry (next arc 013 / bidirectional cache).
  - Compute on demand from the AST's shape + current tier_list.
  - Lean: compute on demand (stateless; AST shape determines d).

**Scope**: runtime.rs Atom/Bundle sites + new router module +
Bundle capacity updates.

### Slice 4 — Cosine validates matching d

**Status: ready after slice 3.**

- `cosine`, `presence?`, `coincident?`: require both operands
  at matching d. Mismatch → error (per capacity-mode) or
  `false`.
- Existing tests at single d continue to pass.

**Scope**: holon op sites in runtime.rs.

### Slice 5 — `set-dim-router!` (user-supplied lambda)

**Status: ready after slice 4.**

- New setter takes optional wat function alongside the tier
  list.
- Config stores the user's lambda or a reference to it.
- Default router remains the built-in sizing function.
- At each Atom/Bundle construction, invoke the active router.

**Sub-fogs:**
- **5a** — user lambda eval performance. Mitigation: memoize
  by AST hash (cache tier-index per AST).
- **5b** — invoking user code from Atom/Bundle construction.
  Via SymbolTable's Environment (capability-carrier pattern,
  Chapter 12's feedback).

**Scope**: config.rs + runtime integration.

## Phase 2 — Migrate callers

Slice 6. Each file migrates independently.

### Slice 6 — Sweep existing callers

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

Slices 7 – 8.

### Slice 7 — Remove `set-dims!` primitive

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

### Slice 8 — INSCRIPTION + doc sweep

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
- **Critical test for slice 1**: a test that uses the
  default `dims=10000` via a blank entry file should produce
  byte-identical cache state to the same test with explicit
  `(:wat::config::set-dims! 10000)`. If so, defaults are
  correct.
- **Start point**: Slice 1 is the smallest change that
  validates the premise. If the defaults don't work, nothing
  else in the arc matters. Slice 1 first.
