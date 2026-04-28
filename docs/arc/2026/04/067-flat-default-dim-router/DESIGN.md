# Arc 067 — Flat default dim router (always 10000)

**Status:** DISCARDED 2026-04-29. Premise (flatten `DEFAULT_TIERS` to `[10000]`) was overtaken by arc 077 (kill the dim router entirely; one program-d via `:wat::config::set-dim-count!`). The flat-default was a halfway step; the aggressive answer that shipped was no router at all.

**Predecessor:** arc 037 introduced the tier-router pattern with
`DEFAULT_TIERS = [256, 4096, 10000, 100000]` — pick smallest tier
where `sqrt(d) ≥ immediate_arity`. The hierarchy was a perf
optimization: small forms got small vectors.

**Consumer:** `holon-lab-trading/docs/proofs/2026/04/008-soundness-gate/`
empirically demonstrated that the perf optimization backfires for
soundness measurement. Small forms (3-5 leaf arity) get d=256;
the noise floor at d=256 is `1/sqrt(256) ≈ 0.0625`, large enough
to swallow the signal we're trying to measure (cosine
discrimination between sound and unsound claims). Forcing d=10000
gives a 6× S/N improvement; the soundness-gate experiment had to
install its own `set-dim-router!` override to function.

Builder direction (2026-04-26):

> "i think we should just update the default func to just return
> 10k for any input - the users can override with their own func
> if they need to"

The cost of small-tier defaults (perf for tiny forms) is dominated
by the cost of small-tier defaults silently degrading measurement
quality for the canonical use case. Default switches.

---

## What's already there (no change needed)

| Surface | Status |
|---------|--------|
| `SizingRouter::with_tiers(vec)` | shipped — user can override with arbitrary tiers |
| `set-dim-router!` (wat-side override) | shipped — user can register a custom router lambda |
| `WatLambdaRouter` (lambda-backed router) | shipped — config path |
| Capacity-mode dispatch on overflow | shipped — None from router → Err per `:error` mode |

The override paths are all in place. This arc just changes the
default the substrate ships with when no override is supplied.

## What's missing (this arc)

| Change | What it does |
|----|----|
| `DEFAULT_TIERS = [10000]` | One-line constant change in `dim_router.rs` |
| `dim_router.rs` unit tests | Update `default_router_picks_smallest_tier`, `leaf_atom_fits_smallest_tier`, `overflow_past_largest_tier_is_none`, `bind_arity_is_2` — tier expectations shift |
| `tests/wat_bundle_capacity.rs` | The 400-atom bundle test no longer fits the default; update to install a custom router, OR adjust bundle size |
| `tests/wat_run_sandboxed.rs` | The "largest tier ≈ 316" comment + assertion adjusts |
| Doc updates | `dim_router.rs` doctest comment; `USER-GUIDE.md` if it references the tier list |

Estimated effort: ~30 lines Rust changed (mostly test
expectations) + ~10 lines doc updates. Single commit.

---

## Decisions to resolve

### Q1 — Single-tier or always-return-10000?

Two readings of "return 10k for any input":

- **(a)** `DEFAULT_TIERS = [10000]` — single-tier router. Anything
  with `arity² ≤ 10000` (arity ≤ 100) returns `Some(10000)`;
  larger returns `None`; capacity-mode dispatches.
- **(b)** A new `FlatRouter` type that returns `Some(10000)`
  unconditionally. Larger inputs hit the Bundle's internal
  capacity check and fail there.

**Recommended: (a).** Minimal code change. Same end result for
overflow (both paths produce a capacity-exceeded signal). Reusing
the existing `SizingRouter` keeps the substrate's router story
uniform — there's still one `SizingRouter` type, just with one
tier instead of four.

### Q2 — What about workloads that genuinely need d=256?

If a consumer is encoding billions of small atoms and the perf
hit of d=10000 matters, they install a custom router via
`set-dim-router!`. The substrate's default is now optimized for
*correctness* (signal/noise headroom for measurement); perf
optimization is opt-in.

This inverts the prior default. The user's framing was explicit:
"users can override with their own func if they need to."

### Q3 — What about workloads that bundle > 100 items?

Same story — install a custom router. Prior default supported up
to 316-arity bundles via the d=100000 tier; new default caps at
100. Consumers who need higher use:

```scheme
(:wat::core::define
  (:my::router (a :wat::holon::HolonAST) -> :Option<i64>)
  (Some 100000))

(:wat::config::set-dim-router! :my::router)
```

Or a tier list:

```rust
SizingRouter::with_tiers(vec![10000, 100000])
```

### Q4 — Is there a migration risk for existing consumers?

The known-good marker (2026-04-24, commit `194778f`) was on the
old default. Consumers that relied on smaller-tier picks:

- For *tests*: pre-existing wat-rs tests that bundle small
  numbers of items will now run at d=10000 instead of d=256.
  Functionally identical; just slower per-test.
- For *production code*: any `cosine` / `presence?` /
  `coincident?` calls compared against fixed thresholds may need
  re-calibration if the threshold was tuned at d=256. The
  substrate-level wat-rs tests that check exact cosine values are
  the audit set.

The known-good marker's intent (a fixed reference point) means
the new default ships under a *new* known-good marker post-arc-067
— callers who pin to the old marker keep the old behavior;
callers who roll forward get the new default.

---

## What ships

One slice. Single commit.

- `DEFAULT_TIERS = [10000]` constant change
- 4 unit tests updated in `dim_router.rs::tests`
- 2 integration tests updated (`wat_bundle_capacity.rs`,
  `wat_run_sandboxed.rs`)
- Doc comment update in `dim_router.rs` explaining the new default
- `USER-GUIDE.md` reference if any (search confirms one mention
  to update)

After this arc lands:
- `holon-lab-trading/wat-tests-integ/experiment/013-soundness-gate/`
  removes its `set-dim-router!` override (no longer needed)
- The proof artifact at proof-008 documents the calibration
  finding that motivated the substrate change

Estimated total: ~50 lines changed. Pattern matches arcs 058–066
in scope.

---

## Open questions

- **Should the tier hierarchy be removed entirely?** No — it's
  still useful for consumers building custom routers. Just shouldn't
  be the default. The infrastructure stays; the default policy
  changes.
- **Is 10000 the right new default?** It's the standard HDC
  reference dim cited throughout the substrate's docs and chapter
  references (Ch 28's slack lemma, Ch 61's adjacent infinities).
  Lower would re-introduce the noise problem; higher would cost
  perf without much measurement-quality gain. 10000 is calibrated.
- **Should we surface the dim choice to the consumer at runtime?**
  E.g., a `(:wat::config::current-dim ast)` primitive. Out of
  scope for this arc; existing `(:wat::config::dims)` returns the
  shim value (unchanged in semantics post-arc-067).

## Slices

One slice. Single commit. Pattern matches arcs 058–066.

## Consumer follow-up

After this arc lands, `holon-lab-trading` updates:

- Remove `set-dim-router!` override from
  `wat-tests-integ/experiment/013-soundness-gate/explore-soundness-gate.wat`
- Verify proof-008 still passes (should — the override produced
  the same dim that's now the default)
- Update proof-008 PROOF.md to note that the explicit override
  is no longer required; the substrate now defaults correctly.

The lab session does the consumer-side cleanup after this arc
ships.
