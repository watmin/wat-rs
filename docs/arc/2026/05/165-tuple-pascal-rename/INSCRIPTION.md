# Arc 165 — INSCRIPTION

## Status

Shipped 2026-05-08. Substrate canonical-form alignment complete:
`:wat::core::Tuple` is the PascalCase canonical spelling everywhere
internal — eval-arm key, head-field storage, `Value::Tuple`
`type_name()` return, type-comparison literal, heterogeneous-tuple
type-check head, test fixture, USER-GUIDE table row, CONVENTIONS.md
example. Pattern 2 poison at `check.rs:3901` redirects legacy
`:wat::core::tuple` → `:wat::core::Tuple` (already the redirect target
since arc 109 slice 1g; arc 165 closes the storage gap so the redirect
target now matches storage).

`cargo test --release --workspace --no-fail-fast`: **0 failed** at
`e1f366b`.

| Slice | Subject | Commit |
|---|---|---|
| 1   | Substrate storage rename + 4 new tests | `e1f366b` |
| 1   | SCORE artifact | `af92ff0` |
| 1   | USER-GUIDE table row + CONVENTIONS example sweep | (this commit) |

## What this arc adds

Arc 109 slice 1f shipped `vec → Vector` (PascalCase canonical container
heads). Arc 109 slice 1g shipped `:wat::core::tuple` callable retirement
via Pattern 2 poison redirecting to `:wat::core::Tuple` — but the
SUBSTRATE-INTERNAL storage paths kept lowercase `tuple` (eval-arm key,
head field, `type_name()` return, type-shape-comparison literal). Arc
165 closes that storage gap.

Per user direction 2026-05-07 (mid-arc-163-slice-3e):
> *"queue up another rename on this.. tuple => Tuple."*

The asymmetry pre-arc-165:
- Pattern 2 poison redirected `:wat::core::tuple` callee → `:wat::core::Tuple`
  in the migration hint
- But the eval arm matched `:wat::core::tuple` as the dispatch key
- And `Value::Tuple` `type_name()` returned bare `"tuple"` (off the
  arc 163 slice 3f FQDN-everywhere convention; Vector / Option / Result
  / HashMap / HashSet were already FQDN)
- And the heterogeneous-tuple type-check stored `head: ":wat::core::tuple"`

Post-arc-165: every internal representation is `:wat::core::Tuple` /
`"wat::core::Tuple"` per the canonical PascalCase shape.

### Substrate changes (single-slice)

| Layer | What changed |
|---|---|
| `Value::type_name()` for `Value::Tuple` | `"tuple"` → `"wat::core::Tuple"` (FQDN PascalCase per arc 163 slice 3f convention) |
| `dispatch_keyword_head` eval arm | Lowercase `:wat::core::tuple` arm REMOVED (was duplicate; PascalCase arm `:wat::core::Tuple` already existed); Pattern 2 poison at check.rs catches any remaining lowercase consumer at type-check |
| `value_matches_type_pattern` for `TypeExpr::Tuple` | comparison literal flipped from `"wat::core::tuple"` → `"wat::core::Tuple"` |
| `eval_tuple_ctor` constructed-value head | `":wat::core::tuple"` → `":wat::core::Tuple"` |
| `infer_tuple_constructor` head field | `":wat::core::tuple"` → `":wat::core::Tuple"` |
| `destructure_tuple` `TypeMismatch` expected prose | `"tuple"` → `"wat::core::Tuple"` (FQDN alignment with the rest) |
| Pattern 2 poison at check.rs:3901-3914 | UNCHANGED in shape; matching key stays `:wat::core::tuple` (the legacy callee); redirect target stays `:wat::core::Tuple`; one-line comment added |
| Test fixture at check.rs:14463 | `(:wat::core::tuple counter driver)` → `(:wat::core::Tuple counter driver)` |
| `tests/wat_arc165_tuple_pascal.rs` | NEW — 4 cases covering canonical Pascal success, legacy lowercase Pattern 2 poison redirect, return-position via `:(T,U)` syntax, and `type_name` FQDN alignment |
| `docs/USER-GUIDE.md:3267` | Surface table row — `:wat::core::tuple` → `:wat::core::Tuple` |
| `docs/CONVENTIONS.md:519-520` | Example code — lowercase tuple → PascalCase Tuple |

### Pre-existing latent defect aligned

Pre-arc-165 `TypeExpr::Tuple(_) => v.type_name() == "wat::core::tuple"`
was effectively unreachable as a true positive: `type_name()` returned
`"tuple"` (bare, no prefix), the comparison expected `"wat::core::tuple"`
(with prefix but lowercase). The mismatch was silent — the comparison
always returned false. Post-arc-165 both sides aligned at
`"wat::core::Tuple"`; the runtime now correctly validates Tuple values
in dispatch arms. No pre-existing tests failed from the alignment,
confirming the path was unreachable in practice (sonnet's report
calibrated this in the EXPECTATIONS row C prediction).

This is the kind of alignment-at-the-margin that a structure-level
substrate sweep surfaces — it reads as "rename" but actually closes a
silent-correctness defect.

## What retired

| Pre-arc | Post-arc | Why |
|---|---|---|
| `Value::Tuple → "tuple"` (off-FQDN) | `→ "wat::core::Tuple"` (FQDN PascalCase) | Aligns with arc 163 slice 3f's FQDN-everywhere container-arm convention |
| `:wat::core::tuple` eval-arm key (lowercase) | `:wat::core::Tuple` arm only (PascalCase canonical) | Pattern 2 poison in check.rs blocks lowercase consumers at type-check; runtime never sees them |
| `head: ":wat::core::tuple"` storage in heterogeneous-tuple TypeExpr / value head | `head: ":wat::core::Tuple"` | Storage matches Pattern 2 poison's redirect target (the spellings now agree) |

## Out of arc 165's scope

**Arc 165 intentionally does NOT cover the USER-GUIDE example block
at lines 2026-2046** (the `:my::dedupe-step` example) because that
block contains multiple stale forms spanning several retired arcs —
legacy `:wat::core::vec` (arc 109 slice 1f surface retirement;
runtime arm hard-retired arc 163 slice 3d), bare primitive types in
the signature line (`:i64`, `:Option<i64>`, `:Vec<i64>`; FQDN'd arc
109 slice 1c), and bare `Some` / `None` grammar (FQDN'd arc 109
slice 1h). Touching only `tuple` at those lines would create worse
mixed-state than leaving the block alone. If/when a caller surfaces
demand for a comprehensive USER-GUIDE staleness sweep across all
retired forms, a new arc opens; arc 165's INSCRIPTION does not
commit to it.

Old-arc DESIGN.md references to lowercase `:wat::core::tuple` (arcs
005, 035, 068, 072, 073, 078, 089) are kept as historical record per
FM 11's "what is inscribed is inscribed" corollary — those documents
captured project state at write time and are immutable as historical
artifacts.

## Discipline notes

### Substrate-as-teacher waterfall

Sonnet shipped the substrate edits + 4 new tests in ~18 minutes, well
under the 30-45 min predicted band. The ~18 minutes includes one
cargo-test cycle that surfaced (a) the duplicate eval-arm at
runtime.rs:3081/3082 and (b) the return-position-type syntax mismatch
(`-> (:wat::core::Tuple ...)` is rejected; the canonical syntax is
`:(T,U,V)`). Both surprises were honest deltas reported, not bridged.

### Calibration

Single-slice mechanical-rename slices continue to ship under predicted
lower bounds (arc 163 slice 3f: ~25 min; arc 165 slice 1: ~18 min).
Future single-slice rename predictions can lean toward 15-30 min.

### BRIEF audit gap (caught by sonnet)

The BRIEF assumed only one eval arm existed for tuple (the lowercase).
In reality, both lowercase + PascalCase arms were registered (a
mid-migration state from arc 109 slice 1g). Sonnet caught this; the
correction was to REMOVE the lowercase arm rather than rename it.
**Calibration note for future rename BRIEFs:** grep BOTH the legacy
AND canonical spellings before listing expected sites; the substrate
may be mid-migration with both arms registered.

## Cross-references

- Arc 109 slice 1f — `Vec → Vector` precedent (PascalCase canonical
  container head)
- Arc 109 slice 1g — `tuple` callee retirement via Pattern 2 poison
  (set the redirect target to `:wat::core::Tuple`; arc 165 closes
  the storage gap)
- Arc 163 slice 3f — substrate primitive paths to FQDN; established
  the FQDN-everywhere container-arm `type_name` convention that arc
  165 brings `Value::Tuple` into compliance with
- Arc 164 — `:wat::core::List<T>` mint (queued; peer arc; mints a new
  USER-FACING type with cons-cell semantics, distinct from
  `:wat::core::Vector<T>`'s right-append semantics)

The substrate's canonical-form discipline now reads uniformly:
`Vector / Tuple / Option / Result / HashMap / HashSet` are PascalCase
canonical containers; primitive paths are `:wat::core::TYPE` FQDN.
