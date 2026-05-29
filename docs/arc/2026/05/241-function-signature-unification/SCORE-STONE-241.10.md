# SCORE — Stone 241.10: `src/remedy/` + ranked-remedy schema upgrade

**Mode:** A (substrate-only, namespaced home mint)
**Runtime:** two sessions (context boundary mid-cascade); vigilia-gated
**Cascade size:** 160 sites in `src/check.rs` + 3 sites in `src/argspec/error.rs` + 1 site deduplication
**Lib tests:** 864 / 0 (1 pre-existing ignored) — net +30 from arc 241 stones
**Clippy:** 883 warnings (≤ 902 gate)
**Vigilia:** 8/8 CONVERGED — L1+L2=0

---

## Phase A Scorecard (12 rows)

| # | Contract | Status | Notes |
|---|----------|--------|-------|
| 1 | C01–C03: typo + retirement remedy production | PASS | variant-typo wired via `variant_typo_remedies()` helper |
| 2 | C04: distant-unknown → no remedy | PASS | threshold filter; no false positives |
| 3 | C05–C07: Display formatting (single/multi/retirement-annotation) | PASS | `render_remedies` canonical format verified |
| 4 | C08: threshold honors far typos (no remedy) | PASS | `typo_threshold` formula verified |
| 5 | Probe whole-suite 8/8 | PASS | `probe_arc241_stone10_remedy` |
| 6 | Stone 241.9 probe preserved | PASS | 8/8 `probe_arc241_stone9_defenum` |
| 7 | Stone 241.8 probe preserved | PASS | 8/8 `probe_arc241_stone8_defstruct` |
| 8 | Stone 241.1–241.7 probes preserved | PASS | all prior arc 241 probes green |
| 9 | Lib baseline ≥ 834 | PASS | 864 / 0 (net +30) |
| 10 | Workspace test-build clean | PASS | `cargo build --release --tests --workspace` exit 0 |
| 11 | Clippy gate ≤ 902 | PASS | 883 |
| 12 | `src/lib.rs` adds `pub mod remedy;` only | PASS | single-line addition |

---

## Phase B — Vigilia Convergence (8 spells; L1+L2=0)

### Cast summary

| Spell | Concern | Cycle count | Result |
|-------|---------|-------------|--------|
| **intueri** | Names speak; WHY comments; spark lives | 0 cycles | PASS — L1+L2=0 |
| **solvere** | No braided concerns; concerns hang straight | 0 cycles | PASS — each of 4 files holds exactly one concern |
| **purgare** | No dead code | 1 amend cycle | PASS — `pub` → `pub(crate)` for `levenshtein`, `typo_threshold`, `TOP_N` (L2 visibility honesty fix) |
| **struere** | Function craft; types enforce contracts | 0 cycles | PASS — all functions pure; signatures honest |
| **sequi** | State visible through chain | 0 cycles | PASS — no hidden globals, no lazy_static; pure throughout |
| **temperare** | No redundant work; threshold formula documented | 0 cycles | PASS — threshold + TOP_N named + documented; Wagner-Fischer table size justified |
| **complectens** | Test weave; each test proves one thing | 0 cycles | PASS — unit tests: 1 claim per test; probe tests: 1 contract per function; `try_startup_display` is a thin named layer |
| **vocare** | Caller-perspective tests | 0 cycles | PASS — unit tests call internal functions (appropriate for module-internal unit tests); probe tests call `startup_from_source` (public API surface) |

**Purgare cycle log:**
- Finding: `pub fn levenshtein`, `pub(crate) fn typo_threshold`, `pub const TOP_N` all declared `pub` but their modules (`distance`, `rank`) are private (`mod distance`, `mod rank`). The `pub` misleads — no external consumer can see them. L2 mumble.
- Amend: `levenshtein` → `pub(crate)` (honest: used by sibling `rank.rs` via `crate::remedy::distance::levenshtein`); `typo_threshold` → `pub(crate)` (used internally by `nearest_match` + tests); `TOP_N` → `pub(crate)`. Doc strings updated to reflect crate-internal scope.
- Post-amend state: L1+L2=0.

---

## Structural Verification (9 rows)

| Verification | Result |
|---|---|
| `src/remedy/mod.rs` exists | ✓ |
| `src/remedy/distance.rs` exists | ✓ |
| `src/remedy/retirement.rs` exists | ✓ |
| `src/remedy/rank.rs` exists | ✓ |
| `Remedy` struct present in `mod.rs` | ✓ |
| `RemedyKind` enum with `Typo` + `Retirement` | ✓ |
| `nearest_match` public API re-exported from `mod.rs` | ✓ |
| `hint: Option<String>` retired (0 active uses) | ✓ — 0 live field declarations; 1 comment reference only |
| `remedies: Vec<Remedy>` added to error variants | ✓ — 4 field declarations across `types.rs` + `check.rs` |

---

## Migration Cascade

### Schema upgrade strategy

Adding `remedies: Vec<crate::remedy::Remedy>` to `CheckError::MalformedForm` and `CheckError::ReturnTypeMismatch` (2 variants, 160 construction sites) triggered 159 E0063 compiler errors. The substrate-as-teacher cascade discipline was followed:

1. Minted standalone `crates/fix-remedies/` (no `wat` dependency) with a state-machine text-transformer
2. Ran `cargo run -p fix-remedies -- src/check.rs` → 157 automatic insertions
3. Fixed 3 residual sites manually (one duplicate insertion at the `variant_typo_remedies` call; two edge-case construction sites the script missed)
4. Removed `crates/fix-remedies/` after migration complete (temporary tool, deleted at completion)
5. Removed `src/bin/fix_remedies.rs` (wrong-crate attempt during cascade, cleaned up)

### Wire-in points

| Path | Change |
|---|---|
| `src/types.rs` — `TypeError::MalformedVariant` | HARD CUT: `hint: Option<String>` → `remedies: Vec<crate::remedy::Remedy>` |
| `src/check.rs` — `CheckError::MalformedForm` | NEW FIELD: `remedies: Vec<crate::remedy::Remedy>` (160 construction sites migrated) |
| `src/check.rs` — `CheckError::ReturnTypeMismatch` | NEW FIELD: `remedies: Vec<crate::remedy::Remedy>` |
| `src/check.rs` — Stone 241.8 hard-cut arm | `remedies: crate::remedy::remedies_for(k, std::iter::empty())` — retirement table hit |
| `src/check.rs` — Stone 241.9 hard-cut arm | `remedies: crate::remedy::remedies_for(k, std::iter::empty())` — retirement table hit |
| `src/check.rs` — `check_function_body` | `variant_typo_remedies(&func.body, &resolved_ret, env.types())` wired into `ReturnTypeMismatch` |
| `src/argspec/error.rs` | `From<ArgSpecError> for CheckError::MalformedForm` updated with `remedies: vec![]` |
| `src/lib.rs` | `pub mod remedy;` added |

### `variant_typo_remedies` helper

The variant-constructor typo path is the structurally interesting wire-in. When:
- The function body is `WatAST::Keyword` (types as `:wat::core::keyword`)
- The declared return type is a `TypeExpr::Path` that maps to a `TypeDef::Enum` in the type environment

...the helper extracts the needle from the keyword text, builds `EnumPath::VariantName` candidates from the enum definition, and returns `nearest_match` results. This fires lazy (at `ReturnTypeMismatch` construction) and is empty for all non-keyword body cases.

---

## `src/remedy/` Module Architecture

```
src/remedy/
├── mod.rs       — public API surface: Remedy, RemedyKind, render_remedies, remedies_for
│                  (re-exports: retirement_lookup, nearest_match)
├── distance.rs  — Wagner-Fischer two-row Levenshtein; crate-internal
├── retirement.rs — explicit RETIREMENT_TABLE + retirement_lookup; public
└── rank.rs      — typo_threshold, TOP_N, nearest_match; public API re-exported
```

**Retirement table at ship time (3 entries):**
```
:wat::core::struct            → :wat::core::defstruct  (Stone 241.8)
:wat::core::struct-restricted → :wat::core::defstruct  (Stone 241.8)
:wat::core::enum              → :wat::core::defenum    (Stone 241.9)
```

Future arc HARD CUTs append entries at ship time. No future-vapor entries (D6).

---

## Honest Deltas

### Context boundary mid-cascade

Stone 241.10 crossed a context boundary mid-flight (during the schema cascade, before the fix-remedies script ran). The continuation session resumed directly from the script execution — no re-exploration, no backtracking. The two-row summary in the compacted context accurately described the pending state.

### Cascade size increase vs prediction

EXPECTATIONS predicted "~20-50 E0063 errors after initial schema edit." Actual: 160 sites (157 fixed by script + 3 manual). The prediction assumed only the targeted variants; the actual count reflects how many functions in `check.rs` construct `MalformedForm` (by far the more common variant). The script strategy was appropriate; the manual fixes were straightforward.

---

## What This Unblocks

**Stone 241.11** — `define ⇒ defn` HARD CUT. With the retirement table live and `remedies_for` wired into HARD CUT arms, Stone 241.11 appends a single retirement entry and the substrate teaches itself. The `[retirement replacement]` annotation appears in every `:wat::core::define` error message — no additional Display work needed.

**Stone 241.12** — INSCRIPTION closes arc 241.

**Arc 237.8b** — reopens after 241.12.
