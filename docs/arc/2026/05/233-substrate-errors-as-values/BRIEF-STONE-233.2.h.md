# BRIEF — Arc 233 Stone 233.2.h — mint `TrackedValue` struct + adapter

## What we're doing

Mint the `TrackedValue` struct that wraps a `Value` with a `Provenance`. Parallel to the existing `Value::Tracked` variant — both shapes coexist after this stone. Stone 233.2.i+ flip the eval signature to use `TrackedValue`; Stone 233.2.k retires `Value::Tracked` entirely.

This is the **scaffolding stone** of the Shape A pivot per `DESIGN-STONE-233.2.g.md`. No behavioral change to existing code. Lib tests baseline held exactly.

## Design substrate (READ FIRST; MANDATORY)

1. **`docs/arc/2026/05/233-substrate-errors-as-values/DESIGN-STONE-233.2.g.md`** — sub-DESIGN; verdict Shape A; full execution decomposition. Read the "Verdict: Shape A" + "Execution decomposition" sections.

2. **`tests/probe_tracked_value_mint_contract.rs`** (commit `0f4e318`) — FM 2-bis disconfirming probe. Pre-stone FAILS with `E0432 unresolved import 'wat::runtime::TrackedValue'`. **The probe IS the success criterion** — sonnet's task is "flip this probe from FAIL to PASS." Each test asserts a specific contract.

3. **`docs/arc/2026/05/233-substrate-errors-as-values/SCORE-STONE-233.2.a.md`** — the original Value::Tracked mint precedent. Same shape of work; mirror the structure.

4. **`src/runtime.rs`** — locate `pub enum Provenance` (search). Place `pub struct TrackedValue` ADJACENT to Provenance (logical neighbor). Both are `pub` and re-exported via `wat::runtime` (verify by inspecting the existing pub-use of Provenance).

## Implementation surface (the type)

```rust
/// TrackedValue — the eval-boundary type pairing a Value with its Provenance.
///
/// Parallel to Value::Tracked variant during the Shape A pivot (Stone 233.2.h
/// scaffolds; 233.2.i flips eval signature; 233.2.j migrates producers;
/// 233.2.k retires Value::Tracked).
///
/// NOT derived: Eq/PartialEq/Hash — callers compare .value()/.provenance()
/// explicitly. TrackedValue is a transient eval-boundary handoff, not a
/// HashMap key or collection element.
#[derive(Clone, Debug)]
pub struct TrackedValue {
    value: Value,
    provenance: Provenance,
}

impl TrackedValue {
    /// Construct a TrackedValue from a value + provenance.
    pub fn new(value: Value, provenance: Provenance) -> Self {
        Self { value, provenance }
    }

    /// Borrow the inner Value.
    pub fn value(&self) -> &Value {
        &self.value
    }

    /// Borrow the provenance metadata.
    pub fn provenance(&self) -> &Provenance {
        &self.provenance
    }

    /// Consume self, yielding the bare Value.
    pub fn value_owned(self) -> Value {
        self.value
    }
}

/// `Value::into()` wraps with Provenance::Unknown — adapter for sites
/// that produce bare Values without producer-level provenance.
impl From<Value> for TrackedValue {
    fn from(value: Value) -> Self {
        Self::new(value, Provenance::Unknown)
    }
}
```

## Where it lives

`src/runtime.rs`, immediately after the `Provenance` enum definition (search for `pub enum Provenance`). Keep the visual proximity — TrackedValue is Provenance's structural sibling.

If the existing `Value::Tracked` variant is co-located with the Value enum elsewhere, do NOT touch it. The variant stays as-is.

## Re-exports

Verify `Provenance` is re-exported via `wat::runtime` (look for `pub use` in lib.rs or runtime.rs's module-level re-exports). Re-export `TrackedValue` via the same surface. The probe imports as `wat::runtime::TrackedValue`.

## Out of scope (affirmative scope-bounding)

- **NO behavioral change.** Value::Tracked variant stays; eval signature stays; producers stay wrapping with Value::Tracked.
- **NO Eq/PartialEq/Hash derive.** Callers extract via .value()/.provenance() and compare explicitly. Forced clarity. The probe asserts this discipline by NOT requiring Eq.
- **NO Display impl yet.** Stone 233.2.i + later may add it; this stone keeps shape minimal.
- **NO migration of existing code** to TrackedValue. Stone 233.2.i is the eval-signature flip; this stone JUST mints the type.
- **NO touching Value enum.** Value::Tracked variant + Value::inner() stay.
- **NO touching producers.** keyword/from-string + 4 others stay wrapping with Value::Tracked.
- **HARD CUT — no aliases.** TrackedValue is the canonical name.
- **holon-rs** — NOT touched.

## Verification flow

```
cargo test --release --test probe_tracked_value_mint_contract 2>&1 | tail -5   # 6/6 PASS post-stone
cargo build --release -p wat 2>&1 | tail -5                                    # 0 errors
cargo test --release --lib -p wat --no-fail-fast 2>&1 | tail -3                # ≥ 827 passed; 0 failed
cargo test --release --test probe_substrate_symmetry_list_span_threading 2>&1 | tail -3   # 1/1 PASS
cargo test --release --test probe_diagnostic_value_snapshot_in_errors 2>&1 | tail -3      # 8/8 PASS
cargo test --release --test probe_value_tracked_transparency 2>&1 | tail -3               # 8/8 PASS
cargo test --release --test probe_diagnostic_dynamic_keyword_invocation 2>&1 | tail -3    # 8/8 PASS
cargo clippy --release --lib -p wat -- -D warnings 2>&1 | grep -c "warning"    # ≤ 54
git -C /home/watmin/work/holon/holon-rs/ status --short                        # empty
```

## STOP triggers (REJECTION criteria)

- **STOP-1:** unexpected compile errors (not solving probe's E0432)
- **STOP-2:** baseline lib tests regress below 827
- **STOP-3:** 45 min elapsed
- **STOP-4:** holon-rs touched
- **STOP-5:** new clippy warning above 54
- **STOP-6:** scope creep — touching Value::Tracked, eval signature, producers, or any existing code beyond the new struct + re-export
- **STOP-7:** probe still FAILS post-mint (any of 6 contracts unmet)
- **STOP-8:** existing arc 233 probes regress

Per FM 2-bis: STOP triggers are REJECTION criteria; never permission-to-defer.

## Trap-door audit

- **NO Eq/PartialEq/Hash derive.** If the borrow checker or test infrastructure demands it (it shouldn't — the probe uses `matches!` patterns), STOP and surface as honest delta; do NOT add the derive without dialogue.
- **NO touching Value enum or Value::Tracked variant.** They live in this stone unchanged.
- **NO Display impl.** Defer to later if needed.
- **Verify re-export path.** Probe uses `wat::runtime::TrackedValue` — ensure visibility chain works (pub struct + module-level pub use as needed).
- **Provenance enum visibility.** TrackedValue uses Provenance in its public API — verify Provenance is publicly accessible (it is per Stone 233.2.a, but confirm).
- **From<Value> impl** — straightforward. Provenance::Unknown is the right default per probe 4.

## Scope reminders

- Mode `model: "sonnet"` (orchestrator sets explicitly per FM 12)
- HARD CUT — no aliases
- Per `feedback_inscription_immutable`: SCORE is a new file
- Per `feedback_no_broken_commits`: do NOT commit. Orchestrator commits after independent verification

## Cross-references

- `docs/arc/2026/05/233-substrate-errors-as-values/DESIGN-STONE-233.2.g.md` — sub-DESIGN; structural pivot rationale + execution plan
- `docs/arc/2026/05/233-substrate-errors-as-values/SCORE-STONE-233.2.a.md` — Value::Tracked mint precedent (same shape of work)
- `tests/probe_tracked_value_mint_contract.rs` — FM 2-bis probe (commit `0f4e318`)
- `feedback_sonnet_writes_substrate` — protocol
- `feedback_inscription_immutable` — SCORE is new file
- `feedback_no_broken_commits` — no commit by sonnet
