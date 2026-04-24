# Arc 037 — dim-router (multi-tier dimensional routing)

**Status:** opened 2026-04-23. Substrate arc implementing the
computation model laid out in BOOK chapters 36-43 (the
substrate-recognition arc).

## One-line summary

Replace `(:wat::config::set-dims! n)` — a required setter that
takes one integer — with `(:wat::config::set-dim-router!
tier-list router-fn)` — an optional setter that takes a tier
list and a router function. Defaults ship; zero-config works.
Migration is **safe and additive**; no breaking changes during
the transition.

## Why this arc exists

Config currently stores `dims: usize` — a single global
dimension. BOOK chapter 41 named the relation `d = K²` where K
= max statement size per tier. BOOK chapter 43 specified the
replacement API: tier list + router function, with the sizing
function as the default router.

`dims` is the last required magic value in Config. Arc 037
replaces it with a user-configurable routing strategy whose
default matches the current behavior.

## The end-state API

```scheme
;; Zero-config — defaults apply:
(:wat::core::define (:user::main ...) ...)

;; Single-tier override (matches current behavior at custom d):
(:wat::config::set-dim-router! [4096] (default-router))

;; Multi-tier override with domain-specific routing:
(:wat::config::set-dim-router!
  [256 1024 10000]
  my-classifier)

;; Capacity-mode override (stays forever as user override):
(:wat::config::set-capacity-mode! :warn)
```

## Defaults (opinionated, ship with substrate)

- **Tier list**: `[256 4096 10000 100000]`
- **Default router**: the sizing function — picks smallest
  tier where `√d ≥ immediate_item_count(ast)`.
- **Capacity mode**: `:error` (surfaces overflow; safe).
- **Other derived fields** (noise_floor, presence_sigma,
  coincident_sigma): already defaulted per arc 024, all
  functions of `dims`.

Zero-config entry files produce the same effective behavior as
today's `(:wat::config::set-dims! 10000)
(:wat::config::set-capacity-mode! :error)`.

## Migration — three phases

### Phase 1 — Add new, keep old (backward compat)

- Add `set-dim-router!` primitive with opinionated defaults.
- Make `dims` OPTIONAL in Config (default 10000).
- Make `capacity-mode` OPTIONAL in Config (default `:error`).
- Keep `set-dims!(n)` functional — if called, overrides the
  default `dims`. Old entry files continue to work unchanged.
- `set-capacity-mode!` stays functional — user override of the
  `:error` default.

**After Phase 1**: zero-config works. Explicit-config still
works. No breakage anywhere.

### Phase 2 — Migrate callers

- Sweep the codebase (tests, examples, lab code) and remove
  redundant `set-dims!` / `set-capacity-mode!` calls where
  they match the defaults.
- Migrate callers that need non-default dims to
  `set-dim-router!([d], default-router)`.
- Each file migrates independently; no all-at-once sweep
  required.

**After Phase 2**: most callers use zero-config. A small number
of explicit-d callers use `set-dim-router!`.

### Phase 3 — Remove dead primitive

- Once NO callers of `set-dims!` remain anywhere (verified by
  grep), remove the primitive itself.
- `set-capacity-mode!` STAYS FOREVER — it's a legitimate user
  override, not a dead migration artifact.
- Docs updated (USER-GUIDE, CONVENTIONS.md).

**After Phase 3**: `set-dim-router!` is the only
dim-specifying primitive. No backward-compat code.

## Slice plan

**CORRECTION (2026-04-23 late-session):** The initial slice plan
kept `dims: usize` as a "default dim" with literal `10000`
fallback. That preserves the exact magic value arc 037 is
killing — renaming it from "required" to "default" is the same
disease. The correct slice 1 eliminates the single-dim
consultation from Atom/Bundle encoders entirely; there is NO
"default dim number" in the runtime path after slice 1.

**There is no default dim.** The default is a FUNCTION — the
sizing function — that returns a dim per AST shape. Atom and
Bundle encoders CALL this function at construction time.
Different-shape ASTs get different dims. Vectors at different
dims coexist.

### Slice 1 — Sizing function as the substrate default

**This is slice 1 correctly scoped.** Larger than a trivial
config edit; the whole point is that the single-dim lookup
disappears from the encoder path.

What this slice ships:

- **Sizing function**: a Rust module implementing
  `default_dim_for_shape(immediate_item_count: usize,
  tier_list: &[usize]) -> Option<usize>`. Returns the smallest
  dim in `tier_list` whose `sqrt(d) >= count`. Returns `None`
  if no tier fits (caller decides: capacity overflow per mode).
- **Default tier list**: `[256, 4096, 10000, 100000]` as a
  substrate-level constant or Config field.
- **Atom / Bundle encoder sites**: replace every read of
  `ctx.config.dims` with a call to the sizing function using
  the AST's immediate shape. The function returns the dim for
  THIS construction. Store that dim alongside the resulting
  HolonAST (via cache entry or ctx, depending on structure).
- **`set-dims!` becomes a no-op** on the encoder path: the
  value still parses and stores in `config.dims` for backward
  compat, but NOTHING reads it. Existing test files continue
  to parse; their encoding dim is now the sizing function's
  output, not the literal they set.
- **`set-capacity-mode!` remains functional** with default
  `CapacityMode::Error` (the `DEFAULT_CAPACITY_MODE` constant
  stays — capacity-mode is the permanent user override, not a
  migration artifact).
- **Required-field machinery retires for `dims` and
  `capacity-mode`**: both become optional with safe defaults.

What this slice does NOT ship:
- `set-dim-router!` primitive (slice 5 — the user-supplied
  function override).
- `set-dim-tiers!` primitive (slice 2 — user override of the
  default tier list).
- Cross-dim validation in cosine / presence / coincident
  (slice 4).

Expected behavior change:
- Test vectors previously at `d=1024` (via `set-dims! 1024`)
  are now at whatever dim the sizing function picks for their
  shape. Many tests will pass unchanged (the shape determines
  the dim deterministically). Tests that hard-code expected
  values derived from a specific d may flip — those are
  expected to be few, and they surface real places the old
  code assumed the magic.

**Scope**: spans `src/config.rs`, `src/runtime.rs` (Atom +
Bundle encode sites), possibly `holon-rs` depending on where
dim actually enters the vector encoding. Not a 50-line change;
plan for a multi-file slice.
**Risk**: medium. The encoder path changes materially. Careful
test sweep required.
**Validates**: the sizing-function-as-default model works end-
to-end; `config.dims` becomes an unread residual field.

### Slice 2 — User-configurable tier list

- Add `set-dim-tiers!(list)` primitive.
- Config stores the user-supplied tier list (overrides the
  default).
- Sizing function uses the configured tier list rather than
  the built-in constant.
- `tier_list` unused by runtime yet — slice 3 starts reading it.

**Scope**: config.rs parser additions + tests.
**Risk**: near-zero. `tier_list` is a new field; nothing reads
it yet.

### Slice 3 — Sizing function as default router

- Implement the sizing function: `fn default_router(ast:
  &WatAST, tier_list: &[usize]) -> usize`.
- Atom / Bundle construction: call the router, get tier index,
  use `tier_list[index]` as d for that construction.
- Bundle's capacity check uses the Atom/Bundle-specific d (not
  `config.dims`).
- With single-tier `tier_list = [10000]`, behavior is
  identical (all constructions route to tier 0).

**Scope**: runtime.rs changes at Atom/Bundle sites +
new router module.
**Risk**: medium. Each construction now consults the router.
Performance: the sizing function is O(K) in item count,
cheap.

### Slice 4 — Cosine / presence / coincident validate matching d

- Cosine, `presence?`, `coincident?` require both operands at
  matching d.
- Mismatch → error (per capacity-mode) or `false` result.
- Tests that construct vectors at different d and compare now
  surface the mismatch.

**Scope**: holon op sites in runtime.rs.
**Risk**: medium. Existing tests at single d continue to pass.
New cross-d tests surface the validation.

### Slice 5 — `set-dim-router!` with user-supplied lambda

- New setter takes a wat function alongside the tier list.
- Config stores the user's lambda; Atom/Bundle invokes it.
- Default remains the built-in sizing function.

**Scope**: config.rs parser + runtime integration.
**Risk**: higher. User code evaluated at every construction.
Memoization likely needed for performance.

### Slice 6 — Sweep existing callers

- Find all `set-dims!` / `set-capacity-mode!` calls in tests,
  examples, lab.
- Remove the ones that match defaults.
- Migrate others to `set-dim-router!`.
- Each file migrates independently.

**Scope**: mechanical sweep across 50+ files.
**Risk**: near-zero. Each migration is a textual replacement
that preserves behavior.

### Slice 7 — Remove `set-dims!` primitive

- Once grep confirms zero callers, remove the primitive
  itself.
- Remove `dims` as a primary Config field (it becomes
  `tier_list[0]` for backward lookup).
- `:wat::config::dims` accessor: either retire or keep as
  "primary dim" shorthand.

**Scope**: small removals.
**Risk**: low. Verified pre-removal.

### Slice 8 — INSCRIPTION + doc sweep

- Arc 037 INSCRIPTION.
- `docs/README.md` arc list.
- USER-GUIDE zero-config section.
- CONVENTIONS.md dim-router conventions.
- Lab-side 058 FOUNDATION-CHANGELOG row.
- Task #53 closed.

## Non-goals (within this arc)

- **No cross-tier Atom lifting primitive.** Users build it
  manually via `Atom(hash, d=higher-tier)` if needed.
- **No multi-tier-aware CacheService.** Per arc 013 (queued)
  — cache entries carrying their d is a separate arc.
- **No router JIT/compilation.** User-supplied lambdas eval'd
  directly; optimization is a later concern.
- **No `set-capacity-mode!` removal.** It stays as a user
  override forever. Only `set-dims!` is migrating out.

## Complected/braid check (BOOK Chapter 42 discipline)

- `tier_list` and `router_fn`: orthogonal. Tier list declares
  the space; router picks within it. ✓
- `set-dim-tiers!` and `set-dim-router!`: separable setters.
  Either can be called independently. ✓
- Sizing function: generic over any tier list. Doesn't depend
  on which tier list is set. ✓
- `set-capacity-mode!`: separate concern from d selection.
  Always optional; always user-overridable. ✓

No braids. Each concern quantizes to its own shell in the
config space.

## Why slice-first

This arc touches:
- Config parser (slices 1-2, 5-7).
- Bundle capacity check (slice 3).
- Atom/Bundle construction (slice 3).
- Cosine / presence / coincident ops (slice 4).
- User lambda evaluation (slice 5).
- Existing callers (slice 6).
- Removal of dead primitive (slice 7).

Shipping all in one slice is too wide. Each slice ships
behind the existing substrate; behavior is unchanged until
slice 5 introduces router-driven d selection. Slice 1 is the
minimal start and validates the defaults match current
behavior.

## Starting point

**Slice 1** — make `dims` + `capacity-mode` optional with
defaults. 50 lines of `src/config.rs` plus test additions.
Verifies that removing explicit setters produces identical
behavior to calling them with the default values.

If slice 1 passes with all existing tests green, the migration
premise is validated and subsequent slices can proceed.
