# Arc 024 — presence-sigma + coincident-sigma config knobs

**Status:** opened 2026-04-22.

**Motivation.** Chapter 28 of the book ("The Measurement") named
the substrate's native granularity: `1/sqrt(dims)` is the 1σ
cosine unit, the smallest angular distance the algebra can
resolve above its own random-pair distribution. The prior
`noise_floor` default was `5/sqrt(dims)` — which conflates the
base unit with a confidence multiplier. Two predicates (`presence?`
and `coincident?`) consumed the single value, but their
interests pull in opposite directions:

- **presence?** wants HIGH σ — low FPR, "don't claim signal
  when there isn't any." 5σ gives FPR ~3×10⁻⁷; 15σ gives
  ~10⁻⁵¹.
- **coincident?** wants LOW σ — tight equivalence, "only say
  'same' when they really are." 1σ = the geometric minimum;
  the substrate physically cannot distinguish below this.

Sharing one σ for both predicates forces a compromise neither
wants. The honest shape is two knobs: the substrate's 1σ base
unit (the true `noise_floor`), and per-predicate multipliers
(`presence_sigma`, `coincident_sigma`).

Builder framing:

> so its 1 and 15 - the users can override these - make them
> config-values and we declare them with defaults of 1 and 15 -
> presence threshold and coincedence threshold

> i think we just have presence? and coincident? closures over
> their respective values. users are free to change them.. we
> just use them.. keep noisefloor - both funcs need it - ya?..

---

## What the arc ships

### Config changes

Retire the 5σ conflation. `noise_floor` becomes its true shape:
the 1σ native granularity.

```rust
pub struct Config {
    pub dims: usize,
    pub capacity_mode: CapacityMode,
    pub global_seed: u64,

    /// 1σ native granularity. Default: 1.0 / sqrt(dims).
    /// Was 5.0 / sqrt(dims) pre-arc-024.
    pub noise_floor: f64,

    /// Presence-sigma multiplier. Default: 15.
    pub presence_sigma: i64,

    /// Coincident-sigma multiplier. Default: 1.
    pub coincident_sigma: i64,

    /// Memoized presence_sigma * noise_floor.
    pub presence_floor: f64,

    /// Memoized coincident_sigma * noise_floor.
    pub coincident_floor: f64,
}
```

### New setters

```scheme
(:wat::config::set-presence-sigma! <i64>)     ;; default 15
(:wat::config::set-coincident-sigma! <i64>)   ;; default 1
```

`set-noise-floor!` stays — power users can still override the
base unit directly.

### Runtime predicates

```rust
// presence?
cosine > ctx.config.presence_floor     // was: ctx.config.noise_floor

// coincident?
(1.0 - cosine) < ctx.config.coincident_floor   // was: ctx.config.noise_floor
```

Each predicate closes over its respective floor. Users don't
compute thresholds; they set sigmas and we memoize.

### Validity check at commit

The two predicates have distinct meanings only when
`presence_sigma + coincident_sigma < sqrt(dims)` — otherwise the
presence threshold rises above (or meets) the coincident
threshold, and the predicates either collide or swap.

Validity check at `collect_entry_file`:

```rust
let sigma_sum = presence_sigma + coincident_sigma;
let dims_sqrt = (dims as f64).sqrt();
if (sigma_sum as f64) >= dims_sqrt {
    // reuses capacity_mode — same four-mode policy Bundle uses
    match capacity_mode {
        CapacityMode::Silent => { /* proceed */ }
        CapacityMode::Warn   => { eprintln!(...); }
        CapacityMode::Error  => { return Err(...); }
        CapacityMode::Abort  => { panic!(...); }
    }
}
```

Reuses `capacity-mode`: it's the "what to do when a substrate
invariant violates" policy. Bundle's capacity overflow uses it.
Sigma-sum overflow uses it. One knob, same semantic.

### Minimum dimension

With defaults (15, 1), the invariant requires `16 < sqrt(d)` →
**d ≥ 257**. Below that, the defaults collapse and users must
either raise dims or lower a sigma.

---

## Slice 0 — promote the fork test helper

Arc 024 testing surfaced a pre-existing flake in
`tests/wat_harness_deps.rs`: three tests share the
`install_dep_sources` OnceLock; if the zero-deps test wins the
race first, the other two fail with `UnknownFunction`. Clean
main hit the flake ~1/15 runs; the rate was the same on arc 024
branches — not caused by arc 024 but surfaced by its testing
pressure.

The fix reuses existing infrastructure. `runtime.rs::tests`
already has `in_signal_subprocess(body)` — a `libc::fork()`
wrapper for signal-test isolation (arc 012 side quest).
Promoted it to `pub fn wat::fork::run_in_fork(body)`. Same
body, public surface.

- Signal tests in `runtime.rs` switch from the private
  `in_signal_subprocess` to the public `run_in_fork`. Body
  identical.
- Each test in `wat_harness_deps.rs` wraps its body in
  `wat::fork::run_in_fork(|| { ... })`. Fresh process, fresh
  OnceLock, no race.

Verified: 20/20 clean workspace runs after the fix.

---

## Non-goals

- **Not** splitting `noise-floor` into separate accessors like
  `presence-floor` / `coincident-floor`. Users interact via
  sigma setters; the derived floors are internal. The
  `noise-floor` accessor stays as the 1σ base.
- **Not** introducing a new validity-mode config separate from
  `capacity-mode`. Reusing the existing one — both are
  substrate-capacity policies.
- **Not** changing `presence?` or `coincident?` shape beyond
  the threshold they consume. Same signatures, same semantics.

---

## Doc sweep

- `docs/CONVENTIONS.md` — namespace table row for `:wat::config::*`
  mentions the new setters.
- `docs/USER-GUIDE.md` — section 6 (algebra forms), section 12
  (error handling), mental model.
- `docs/arc/2026/04/005-stdlib-naming-audit/INVENTORY.md` — new
  setter keys + note that `noise-floor` now defaults to 1σ.
- `holon-lab-trading/docs/proposals/.../FOUNDATION.md` — the
  "Presence Is Measurement" section + "Algebra Measurements"
  block get the dual-sigma framing.
- FOUNDATION-CHANGELOG row.

---

## Tests

- **Rust unit tests** (`src/config.rs`): defaults, override
  accepted, sigma-sum validity check fires under :error,
  passes under :silent / :warn, panics under :abort.
- **Runtime tests** (`src/runtime.rs`): `presence?` uses
  `presence_floor`; `coincident?` uses `coincident_floor`; both
  dynamic under user override.
- Plus the pre-existing wat-level tests in
  `wat-tests/holon/coincident.wat` still pass at the new
  thresholds (tests using `presence?` / `coincident?` are
  structurally correct under any σ above the validity floor).

---

## Why this is inscription-class

The decision surfaced during Chapter 28's writing — naming the
1σ native granularity made the conflation visible. The
implementation follows a discovery, not a design. Same shape as
019 (round), 020 (assoc), 023 (coincident?).
