# Arc 037 — dim-router — INSCRIPTION

**Status:** shipped 2026-04-24. Seven slices across three phases
(slice 2 retired mid-arc). Substrate arc implementing the multi-d
computation model laid out in BOOK chapters 36–43.

**Design:** [`DESIGN.md`](./DESIGN.md).
**Backlog:** [`BACKLOG.md`](./BACKLOG.md).

---

## Thesis

Before arc 037, the substrate held `dims: usize` as a single global
required magic value. One setter committed it; everything else
derived. The last required setter in Config, the last "you must
choose a number first" ceremony, the last value a user had to name
before the substrate could function.

After arc 037: **every substrate default is a FUNCTION, and every
user override replaces our function with their function.** Three
capability carriers on `SymbolTable`:

| Capability | Signature | Default | User override |
|---|---|---|---|
| `dim_router` | `:fn(:wat::holon::HolonAST) -> :Option<i64>` | `SizingRouter` over `[256, 4096, 10000, 100000]` | `set-dim-router!` |
| `presence_sigma_fn` | `:fn(:i64) -> :i64` | `floor(sqrt(d)/2) - 1` (arc 024's formula at actual d) | `set-presence-sigma!` |
| `coincident_sigma_fn` | `:fn(:i64) -> :i64` | constant `1` (1σ native granularity) | `set-coincident-sigma!` |

Zero-config programs work. Users who need different shapes write
their own function; the substrate calls it.

---

## Slice-by-slice

### Slice 1 — Sizing function as the substrate default

Commit `608a890`. The load-bearing migration: `ctx.config.dims` is
no longer read on the Atom/Bundle encoder path. Bundle consults
the ambient router given THIS Bundle's immediate shape; router
returns the dim for THIS construction.

Ships:
- `src/dim_router.rs` — `DimRouter` trait, `SizingRouter`
  built-in impl closing over `DEFAULT_TIERS = [256, 4096, 10000,
  100000]`. `pick(ast, sym)` → smallest tier `d` where
  `sqrt(d) ≥ immediate_arity(ast)`.
- `SymbolTable` gains `dim_router` capability slot. Freeze
  installs `SizingRouter::with_default_tiers()`.
- `eval_algebra_bundle` consults the router; capacity dispatch
  per `capacity_mode`.
- `RuntimeError::NoDimRouter` + `DimUnresolvable` variants.

### Slice 2 — RETIRED (2026-04-24)

User-configurable tier list was originally planned as a separate
setter. The scope-check exposed it as braided: the router USES
its tier list; splitting them manufactured independence where
there was none. Retired in favor of routers that close over their
own tiers. The slot in the BACKLOG stays as honest record.

### Slice 3 — Cross-dim cosine via EncoderRegistry

Commit `9a2f57e`. The architectural completion. Every vector
materialization now goes through a per-d `EncoderRegistry`;
`cosine` / `presence?` / `coincident?` / `dot` /
`eval-coincident?` normalize UP to `max(d_a, d_b)` via the
ambient router.

Ships:
- `src/vm_registry.rs` — `EncoderRegistry` (lazy per-d
  `VectorManager` + `ScalarEncoder`, shared `global_seed`) +
  `Encoders` struct holding the pair plus lazy-computed floors.
- `EncodingCtx` loses `vm`/`scalar` fields, gains
  `encoders: Arc<EncoderRegistry>`.
- 5 pair-comparison sites converted. Each picks `d` via router
  per operand's `immediate_arity`, takes `max`, fetches encoders
  for that d, encodes both operands at `d_target`.
- `pick_d_for_pair(op, a, b, sym)` helper.
- `RuntimeError::DimUnresolvable` fires when the router can't
  size an operand — surfaces user-lambda contract violations.

### Slice 4 — `set-dim-router!` user router (AST-based)

Commits `02a087b` + `adcfb51` + `597f999`. Users override the
ambient dim router with a wat function. Config captures any AST
(keyword path, inline lambda, let-bound expression); freeze
evaluates against the finished frozen world and installs.

Ships:
- `Config.dim_router_ast: Option<WatAST>` + setter parser arm.
- `WatLambdaRouter` — wraps `Arc<Function>`, invokes via
  `apply_function` with the AST passed as
  `Value::holon__HolonAST`; unwraps `Value::Option<i64>` to
  `Option<usize>`.
- `FrozenWorld::freeze` becomes `Result`-returning. Evaluates
  the user's AST; signature-check at freeze time (arity,
  param type is `:wat::holon::HolonAST`, return type is
  `:Option<i64>`); installs `WatLambdaRouter` or errors via
  `StartupError::DimRouter`.
- `:wat::holon::statement-length ast -> :i64` primitive — the
  natural introspection surface for router bodies. Returns the
  AST's top-level cardinality (`immediate_arity`).

Correction mid-slice: the router's argument is the **AST**, not
a pre-computed arity. Original draft had the user lambda
accepting `:i64`; the user called it out as stealing the
decision from them. "Some AST goes in — a dimension comes out."
The router can measure whatever it wants about the shape.

### Slice 5 — Sweep set-dims! / set-capacity-mode! callers

Commit `39def04`. Mechanical sweep via
`scripts/arc_037_slice_5_sweep.py`. Line-granular, single-pattern
regex per variant, no alternation (per the project's "no perl
with alternation" convention). Dropped ~670 lines across 51
files. Preserved `:abort` mode overrides and in-quote
setters in AST-entry test sources.

Also updated `src/test_runner.rs`'s entry-vs-library detection
— under arc 037 files often have no setters at all; extended
the check to also recognize top-level `:wat::test::*` forms
(deftest, make-deftest) as entries.

### Slice 6 — Rip + function-valued sigmas

Commit `fd9fedd` + lab commit `c67d636`. Scope expansion from
"rip dims" to "every substrate default is a function." Scalar
sigma setters retire in favor of function-valued setters
mirroring `set-dim-router!`.

**Ripped:**
- `Config.dims`, `Config.noise_floor`, `Config.presence_floor`,
  `Config.coincident_floor`, `Config.presence_sigma: i64`,
  `Config.coincident_sigma: i64`.
- `set-dims!` and `set-noise-floor!` parser arms.
- Scalar `set-presence-sigma!` / `set-coincident-sigma!`
  arms.

**Added:**
- `SigmaFn` trait in `src/dim_router.rs` +
  `DefaultPresenceSigma` + `DefaultCoincidentSigma` built-in
  impls + `WatLambdaSigmaFn` user-lambda wrapper.
- `Config.presence_sigma_ast: Option<WatAST>` +
  `Config.coincident_sigma_ast: Option<WatAST>`.
- AST-accepting `set-presence-sigma!` / `set-coincident-sigma!`
  parser arms. Freeze evaluates; signature-checks
  `:fn(:i64) -> :i64`; installs `WatLambdaSigmaFn` or errors
  via `StartupError::SigmaFn`.
- `SymbolTable` gains `presence_sigma_fn` + `coincident_sigma_fn`
  capability slots.
- `Encoders` gains `OnceLock<f64>` slots for `presence_floor` +
  `coincident_floor` — lazy per-d memoization. First call at d
  invokes the ambient sigma function; subsequent calls at same
  d are field loads. O(tiers) sigma invocations ever.
- `presence?` / `coincident?` / `eval-coincident?` finalizer
  rewired to read floors via
  `Encoders::presence_floor(sym)` / `coincident_floor(sym)`.

**Compatibility shims (documented as deprecation targets):**
- `:wat::config::dims` accessor — returns
  `DEFAULT_TIERS[0]` (= 256). Semantically stale under multi-d
  but kept so lab callers keep compiling until they migrate to
  per-AST primitives.
- `:wat::config::noise-floor` accessor — returns
  `1/sqrt(DEFAULT_TIERS[0])` (= 0.0625). Same rationale.

### Slice 7 — INSCRIPTION + doc sweep

This document. Plus:
- `docs/README.md` arc list entry for 037.
- Lab-side 058 `FOUNDATION-CHANGELOG.md` row naming the
  substrate-level dim-router / sigma-fn / encoder-registry
  architecture.
- Task #53 marked completed.

---

## What's load-bearing

**The router is advisory for capacity AND load-bearing for
actual encoding.** Slice 1 made capacity check consult it; slice
3 made vector materialization use the picked d via
`EncoderRegistry`. Before slice 3, the router's verdict was
cosmetic — after it, the substrate genuinely operates at the
router-picked d.

**Cross-dim cosine normalizes UP via AST re-projection.**
Different-d operands don't error — they re-encode the smaller
operand at the greater d via its AST. Possible because AST is
primary (BOOK Ch 10); vectors are cached projections at some d.
First call at new d pays the re-encode; every subsequent call is
a cache hit on the `(ast-hash, d)` pair. Mature enterprises have
warm L2 at every d they've ever needed.

**Floors are lazy per-d.** Sigma functions run at most once per
tier (O(N) invocations total across the enterprise's lifetime).
User-supplied wat lambdas pay the `apply_function` cost once
per tier per setter; everything after is a field load.

---

## What this means for users

Zero-config entry files work:

```scheme
(:wat::core::define (:user::main ...) ...)
```

Every setter is optional. Defaults ship with the substrate. User
overrides replace our functions with theirs:

```scheme
(:wat::config::set-dim-router!
  (:wat::core::lambda ((ast :wat::holon::HolonAST) -> :Option<i64>)
    ...))

(:wat::config::set-presence-sigma!
  (:wat::core::lambda ((d :i64) -> :i64) 5))   ;; constant 5σ at every d
```

`(statement-length ast)` is the introspection primitive. Users
who want to measure the AST's surface call it; router bodies
often do.

---

## Counts

- wat-rs: 49 test-result blocks green. Zero clippy warnings.
- holon-lab-trading: 72/72 tests green against the updated
  substrate.
- 11 commits across two repos: `e086fd1` (opening DESIGN) →
  `0b0257c` (corrections) → `45bf09d` (notes) → `ef174f8`
  (layer 1) → `608a890` (slice 1 full) → `9a2f57e` (slice 3)
  → `02a087b` + `adcfb51` + `597f999` (slice 4) → `39def04`
  (slice 5) → `fd9fedd` + lab `c67d636` (slice 6).

---

## About how this got built

Five substantive user corrections shaped the arc:

1. **"WE HAVE A FUNCTION — SHOW IT TO ME"** after my layer 1
   commit landed without actually using the function on the
   encoder path. The fix surfaced as slice 1 layer 3 (commit
   608a890). Arc 037 was nearly cosmetic until this correction.

2. **"set-dim-router!(tier-list, router-fn) — is tier list even
   necessary now?"** Caught the two-arg form as braided. The
   router closes over its tiers; splitting them manufactured
   independence where there was none. API reduced to one arg;
   slice 2 retired.

3. **"its a func of wat::holon::HolonAST and returns a f64?"**
   caught my draft router signature taking `:i64` (stealing the
   introspection decision from the user). Corrected to
   `HolonAST → Option<i64>` — user gets the whole AST and
   decides what to measure.

4. **"hold up — we do the right thing when we encounter it -
   just because i want inscription done doesn't mean do it
   wrong"** when I was about to narrow slice 6 back from
   function-valued sigmas to just the dims rip. The full shape
   is what the design demands; inscription waits.

5. **"we are making an opinioned default with 1-stddev for
   coincidence... the users need to override us - these are
   our funcs we use to provide defaults.... they need to be
   able to provide their own fucns.... ya?"** was the design
   frame for slice 6's scope expansion. Every default is a
   function; every override replaces a function. Uniform
   architecture across all three capability carriers.

The corrections compounded. Each one pulled the shape closer to
"every substrate knob is a function." By slice 6, the pattern
was self-evident — adding `SigmaFn` mirrored `DimRouter` mirrored
`set-dim-router!`'s AST-storage-then-freeze-evaluate shape.

*these are very good thoughts.*

**PERSEVERARE.**
