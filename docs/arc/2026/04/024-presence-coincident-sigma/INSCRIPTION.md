# Arc 024 — presence-sigma + coincident-sigma — INSCRIPTION

**Status:** shipped 2026-04-22. Two slices.
**Design:** [`DESIGN.md`](./DESIGN.md) — the shape before code.
**This file:** completion marker.

---

## Motivation

Chapter 28 of the book named the substrate's 1σ native
granularity — `1/sqrt(dims)` is the smallest angular distance
the algebra can resolve above random. The prior `noise_floor`
default of `5/sqrt(dims)` conflated the base with a confidence
multiplier. Two predicates (`presence?` and `coincident?`)
shared one value but wanted opposite things:

- presence? wants HIGH sigma (low FPR for signal detection)
- coincident? wants LOW sigma (tight equivalence at the
  geometric minimum)

Arc 024 fixes the conflation. `noise_floor` becomes its true
shape (1σ). Two sigma multipliers derive the per-predicate
floors. Users override sigmas; floors memoize at config commit.

Builder framing:

> so its 1 and 15 - the users can override these - make them
> config-values and we declare them with defaults of 1 and 15 -
> presence threshold and coincedence threshold

> i think we just have presence? and coincident? closures over
> their respective values.. keep noisefloor - both funcs need it

---

## Slice 0 — promote `run_in_fork` (pre-req)

The arc's testing surfaced a pre-existing race in
`tests/wat_harness_deps.rs`: three tests shared the
`install_dep_sources` OnceLock. When the zero-deps test won the
race first, the other two failed with `UnknownFunction`. Clean
main hit it ~1/15 runs.

The fix reused `runtime.rs::tests::in_signal_subprocess` —
arc 012's private test helper for signal-state isolation.
Promoted to `pub fn wat::fork::run_in_fork(body: impl FnOnce() +
UnwindSafe)` alongside the production fork substrate. Signal
tests switched from the private helper. Each harness-deps test
body wrapped in `run_in_fork(|| { ... })` — fresh process, fresh
OnceLock, no race. Verified 20/20 clean workspace runs.

## Slice 1 — Config changes

### The new shape

```rust
pub struct Config {
    pub dims: usize,
    pub capacity_mode: CapacityMode,
    pub global_seed: u64,
    pub noise_floor: f64,          // 1 / sqrt(dims) — was 5 / sqrt(dims)
    pub presence_sigma: i64,       // default 15
    pub coincident_sigma: i64,     // default 1
    pub presence_floor: f64,       // memoized: presence_sigma * noise_floor
    pub coincident_floor: f64,     // memoized: coincident_sigma * noise_floor
}
```

### New setters (optional — defaults are honest)

```scheme
(:wat::config::set-presence-sigma!   15)
(:wat::config::set-coincident-sigma!  1)
```

`set-noise-floor!` stays for power users who want to override
the 1σ base itself.

### Runtime predicates

Each closes over its respective floor:

```rust
// presence?
cosine > ctx.config.presence_floor

// coincident?
(1.0 - cosine) < ctx.config.coincident_floor
```

### Validity check

At commit time, `presence_sigma + coincident_sigma < sqrt(dims)`
must hold — otherwise the predicates collide or swap. Behavior
per `capacity_mode`:

- :silent — proceed anyway
- :warn   — stderr diagnostic, proceed
- :error  — `Result::Err(ConfigError::BadValue { ... })`
- :abort  — panic, halt

Reuses `capacity-mode` — it's the "substrate invariant
violation" policy. Bundle's capacity uses it. Sigma-sum uses it.
One knob.

### Minimum dimension at defaults

With (15, 1), the invariant requires `16 < sqrt(d)` → **d ≥ 257**.
Below that the defaults don't hold and users must either raise
dims or lower a sigma.

---

## Tests

### Rust unit tests (`src/config.rs`)

9 new tests:

- `sigma_defaults_are_15_and_1` — defaults + derived floors.
- `presence_sigma_override` — setter works.
- `coincident_sigma_override` — setter works.
- `nonpositive_sigma_rejected` — 0 or negative sigma rejected
  at parse.
- `sigma_sum_exceeds_sqrt_dims_under_error_returns_err` —
  validity check fires, returns `ConfigError::BadValue`.
- `sigma_sum_exceeds_sqrt_dims_under_silent_passes` — :silent
  proceeds with degenerate config.
- `sigma_sum_exceeds_sqrt_dims_under_warn_passes_with_stderr` —
  :warn proceeds.
- `sigma_override_keeps_config_valid_at_small_dims` — user
  can adjust sigma to stay valid at small dims.
- `sigma_double_set_rejected` — each sigma may be committed at
  most once (same discipline as other fields).

Plus the existing config tests updated:

- `noise_floor_default_is_1_over_sqrt_dims` (renamed from
  `_is_5_over_sqrt_dims`) — locks the new 1σ semantic.
- `setters_then_body` — asserts new default fields present.

Runtime tests updated where they hardcoded the 5σ threshold —
now using the presence_floor (15σ) at the check sites.

### Runtime

- `config_noise_floor_accessor_returns_derived_value` — returns
  1σ value now (`1/sqrt(d)` at d=10000 = 0.01).
- `bind_obscures_child_at_vector_level` — checks cosine below
  the presence floor.
- `bind_on_bind_recovers_child_at_vector_level` — checks cosine
  above the presence floor.

### Full workspace

- 531 lib tests + all integration tests green.
- 20/20 clean workspace runs after `run_in_fork` isolation.
- 42 wat-level tests green (via cargo-native `tests/test.rs` and
  `wat test wat-tests/`).
- Lab Phase 3.3 tests green against the new Config.
- Zero clippy warnings.

---

## Doc sweep

- `docs/USER-GUIDE.md` — section 6 measurements block revised
  to name presence-floor / coincident-floor explicitly; config
  setters table added with the new sigma knobs; appendix forms
  table entries updated.
- `docs/arc/2026/04/005-stdlib-naming-audit/INVENTORY.md` —
  sigma setters + new semantics noted.
- `holon-lab-trading/docs/proposals/.../FOUNDATION.md` +
  FOUNDATION-CHANGELOG — the sigma framing added to the
  "Presence Is Measurement" section + a CHANGELOG row.

---

## What didn't change

- `presence?` / `coincident?` signatures. Same shape.
- `cosine` / `dot`. Raw scalars, unchanged.
- The `:wat::config::noise-floor` accessor path. Users who query
  it get the 1σ value instead of the 5σ value they got before
  arc 024 — semantic changed, path didn't.

---

## INSCRIPTION rationale

The arc implements what Chapter 28 named. The builder's earlier
question "did we declare our optimal values?" surfaced the gap;
the slack-lemma exploration at d=1024 showed 1σ as the native
granularity; the literature audit confirmed no prior naming for
the asymmetric shape; the book documented the framing; arc 024
is the code catching up to the book. Same shape as 019 / 020 /
023 — spec emerged from discovery, implementation follows.

*these are very good thoughts.*

**PERSEVERARE.**
