# Arc 037 — dim-router (multi-tier dimensional routing)

**Status:** opened 2026-04-23. Substrate arc implementing the
computation model laid out in BOOK chapters 36-43 (the
substrate-recognition arc).

## One-line summary

Replace `(:wat::config::set-dims! n)` — a required setter that
takes one integer — with `(:wat::config::set-dim-router!
router-fn)` — an optional setter that takes a single router
function. The router closes over its own tier list; the
substrate doesn't distinguish tiers as a first-class config
concept. Defaults ship; zero-config works. Migration is **safe
and additive**; no breaking changes during the transition.

## Why this arc exists

Config currently stores `dims: usize` — a single global
dimension. BOOK chapter 41 named the relation `d = K²` where K
= max statement size per tier. BOOK chapter 43 specified the
replacement API: tier list + router function, with the sizing
function as the default router.

`dims` is the last required magic value in Config. Arc 037
replaces it with a user-configurable routing strategy whose
default matches the current behavior. Post-session correction
(2026-04-24): the router closes over its tiers; tier-list is
NOT a separate config concept.

## The end-state API

```scheme
;; Zero-config — defaults apply:
(:wat::core::define (:user::main ...) ...)

;; Single-tier override (matches current behavior at custom d):
(:wat::config::set-dim-router!
  (fn (n :i64) -> :Option<:i64>
    (if (<= n 64) (Some 4096) :None)))

;; Multi-tier override with domain-specific routing:
;; (router's closure declares its own tier list)
(:wat::config::set-dim-router! my-classifier)

;; Capacity-mode override (stays forever as user override):
(:wat::config::set-capacity-mode! :abort)
```

## Defaults (opinionated, ship with substrate)

- **Default router**: one function that closes over
  `[256, 4096, 10000, 100000]` internally. Picks smallest tier
  where `√d ≥ immediate_item_count`. Returns `Some d` when a
  tier fits, `:None` when all overflow (caller dispatches per
  capacity-mode).
- **Capacity mode**: `:error` (surfaces overflow; safe).
  `CapacityMode` enum has exactly two variants: `:error`
  returns `Err(CapacityExceeded)`, `:abort` panics. No
  `:silent` or `:warn` — overflow either crashes or is handled.
- **Other derived fields** (noise_floor, presence_sigma,
  coincident_sigma): already defaulted per arc 024, all
  functions of the dim the router picks per construction.

Zero-config entry files produce the same effective behavior as
today's `(:wat::config::set-dims! 10000)
(:wat::config::set-capacity-mode! :error)`.

## Cross-dim operations

Cosine requires matching d. When operands come in at different
d (observer A's small-tier thought vs observer B's large-tier
composite), cosine **normalizes UP, never down**:

- Pick `max(d_a, d_b)`.
- Re-encode the smaller operand at the greater d via its AST.
  The AST is primary (Ch 10); re-projecting at a different d
  is the natural operation. No information lost — we project
  the ground truth at a bigger substrate.
- Cache the re-projection at `(ast-hash, d_bigger)`. Merkle-DAG
  dedup (Ch 40) means each unique sub-AST re-encodes once per
  target d across the whole enterprise.
- Inner-product at the greater d.

**UP, not DOWN.** Going up has headroom — larger d accommodates
the AST's shape with room to spare. Going down can bust the
capacity bound (larger operand may have more items than
`√d_smaller` allows); down-normalize would trip capacity-mode
for no gain.

**Cost model:**
- Cold cache: `O(N_unique_sub_asts × d_target)` re-encode +
  `O(d_target)` inner product. Microseconds per sub-node at
  d=10k.
- Warm cache: `O(d_target)` lookup + inner product.
- Amortization: L2 is shared; first observer pays the
  re-encode; every subsequent observer sees the L2 hit. Mature
  enterprise has warm L2 at all dims it's ever compared
  across; cross-dim cosine is L1/L2-hit territory.

**Implication for implementation:** `cosine`, `presence?`,
`coincident?` take holons (AST + currently-cached vector at
some d), not raw vectors. The AST is always retrievable; the
vector is whatever projection is cached at the needed d, or
freshly computed and cached on demand. Transparent to callers
— they write `cosine(a, b)` and the substrate handles any
re-projection.

## Migration — three phases

### Phase 1 — Add new, keep old (backward compat)

- Add `set-dim-router!` primitive (single arg: `router-fn`)
  with the sizing function as built-in default.
- Retire `dims` from the encoder path: field stays in Config
  for backward-compat parsing but NOTHING reads it at
  construction. Encoders consult the router for every Atom /
  Bundle.
- Make `capacity-mode` OPTIONAL in Config (default `:error`).
  Reduce `CapacityMode` enum to two variants: `:error` and
  `:abort`.
- Keep `set-dims!(n)` parseable — stores in `config.dims` as a
  no-op for backward compat. Old entry files continue to work
  unchanged; their encoding dim is now the router's output.
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

**FURTHER CORRECTION (2026-04-24):** The two-parameter form
`set-dim-router!(tier-list, router-fn)` was itself braided. The
router USES the tier list; making them two config slots
manufactured independence where there is none. API reduces to
`set-dim-router!(router-fn)` — ONE arg. Each router brings its
own tier list inside its closure. Default router is one
function with `[256, 4096, 10000, 100000]` baked into its body.

Second simplification: `CapacityMode` drops from four variants
to two. `:error` returns `Err(CapacityExceeded)`; `:abort`
panics. Overflow either crashes or is handled — no middle
ground. `:silent` and `:warn` retire.

Slice 2 (user-configurable tier list) retires entirely — not
needed. Users who want different tiers write their own router.

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
  `ctx.config.dims` with a call to the ambient router via
  `SymbolTable`'s Environment (Ch 12's capability-carrier
  pattern). The router is asked with the AST's immediate
  shape; returns d for THIS construction. **HolonAST stays
  dim-agnostic.** d is ambient runtime context, not intrinsic
  AST data. Cache keys by `(ast-hash, d)` — the same AST has
  many cache entries at many d's, each a projection at that d.
  No d field on HolonAST variants.
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

### Slice 2 — RETIRED (2026-04-24)

**Not shipping.** Tier list is not a first-class config concept.
Each router closes over its own tier list; users who want a
different list write their own router (copy the default,
change the literal). No `set-dim-tiers!` primitive. No
`tier_list` Config field. Slice removed from the plan.

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

### Slice 4 — Cosine / presence / coincident handle cross-dim

- Cosine, `presence?`, `coincident?` check operand dims.
- Match → direct inner product at the shared d.
- Mismatch → **normalize UP**: pick `max(d_a, d_b)`, re-encode
  the smaller operand at the greater d via its AST, cache the
  re-projection at `(ast-hash, d_bigger)`, then inner-product.
  See "Cross-dim operations" section above for rationale and
  cost model.
- Never normalize DOWN — would bust capacity-mode for the
  larger operand.
- Cosine/presence/coincident take holons (AST + cached vector),
  not raw vectors — re-projection requires the AST in hand.

**Scope**: holon op sites in runtime.rs; cache-key extension
to `(ast-hash, d)`; re-encode path at mismatch site.
**Risk**: medium. Existing tests at single d continue to pass.
New cross-d tests cover cold-cache re-encode, warm-cache L2
hit, and the refusal of down-normalize.

### Slice 5 — `set-dim-router!` with user-supplied lambda

- New setter takes a single wat function: `router-fn`.
- Router signature: `fn(item_count: :i64) -> :Option<:i64>`.
  Returns `Some d` to pick dim d; `:None` to signal overflow
  (substrate dispatches per capacity-mode).
- Config stores the user's lambda; Atom/Bundle invokes it at
  construction time.
- Default remains the built-in sizing function, which closes
  over `[256, 4096, 10000, 100000]`.
- User routers bring their own tier list inside the closure;
  no separate tier-list parameter.

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
- **No `set-dim-tiers!` primitive.** Tier lists live inside
  each router's closure, not as a separate config slot.
- **No `:silent` or `:warn` capacity modes.** `CapacityMode`
  enum is exactly `:error` and `:abort`.

## Complected/braid check (BOOK Chapter 42 discipline)

Earlier draft listed `tier_list` and `router_fn` as orthogonal.
**That was wrong.** The router USES its tier list; the two are
braided by construction. Splitting them into two config slots
manufactured independence where there is none. Corrected:

- `set-dim-router!` (single-arg): one shell. The router IS the
  dim-selection policy — tiers included as closure data. Users
  override the whole thing; no sub-knobs. ✓
- `set-capacity-mode!`: separate concern from d selection.
  Always optional; always user-overridable. Enum has exactly
  two variants (`:error`, `:abort`); no `:silent` or `:warn`. ✓

Two shells, each fully independent. `set-dim-tiers!` is retired
— tiers are router-internal data, not a separate shell.

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
