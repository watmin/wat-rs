# Arc 165 — `tuple` → `Tuple` Pascal-case rename

**Status:** queued 2026-05-07. Not yet started.

**Gates:** arc 163 closes first (substrate canonical-form purification).

## Background

User direction 2026-05-07 mid-arc-163-slice-3e: *"queue up another rename
on this.. tuple => Tuple."*

`:wat::core::tuple` is the only canonical container head still in
lowercase. PascalCase canonical types are the convention (per arc 109
slice 1f: `Vec → Vector`; arc 109 slice 1g: `tuple → Tuple` for the
type and `Tuple` constructor verb). Substrate currently has mixed
casing — Vector/Option/Result/HashMap/HashSet/Tuple all PascalCase
EXCEPT `tuple` which lingers.

Arc 163 slice 3e Category A renamed `Value::Tuple(_) =>
"wat::core::tuple"` (lowercase) per the migrating-from-legacy state.
That choice carries forward the lowercase. Arc 165 lifts it to
PascalCase canonical.

## Scope

- `Value::Tuple(_) => "wat::core::Tuple"` (capitalize the canonical name)
- All `head: "wat::core::tuple"` writes in substrate → `"wat::core::Tuple"`
- Walker BareLegacyContainerHead's TUPLE arm (if any) tracks the rename
- Pattern 2 poison for legacy `:wat::core::tuple` redirects to `:wat::core::Tuple`
- Test fixtures + doc comments updated

## Why arc 165 is the right shape

Four questions:
- Obvious — Pascal-case canonical types is the convention; tuple is the
  outlier.
- Simple — mechanical sweep on one head string.
- Honest — closes the casing inconsistency.
- Good UX — readers see uniform PascalCase canonical types throughout.

## Cross-references

- Arc 163 slice 3e — substrate canonical-form purification
- Arc 109 slice 1f — `Vec → Vector` precedent
- Arc 109 slice 1g — `tuple → Tuple` first attempt (slice 1g shipped
  the constructor verb but lowercase persisted in some Value/head
  storage paths; arc 165 closes the gap)

## When this opens

After arc 163 INSCRIPTION ships.
