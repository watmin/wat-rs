# DESIGN — Stone 249.5f: canonical scope renumbering at hash time (close the hygiene class)

> Status: STRIKE-DRAWING.
> The THIRD and final identifier-keying site of the macro-hygiene class. Closes the
> documented-deferred caveat in `src/hash.rs` (lines 44-65, "Hygiene-scope caveat"),
> named by the original `DESIGN-STONE-249.5-hygiene-completion.md` ("canonical scope
> renumbering at hash time … ships WITH resolution; they are coupled") — but it did
> NOT ship with 249.5b; only resolution did.

## Why

`hash(expanded AST) IS identity` is the substrate's FOUNDATION claim — it backs
content-addressed caching AND the **deterministic cross-node consensus** the
`vector_manager` design rests on (same seed/program → same vector/hash everywhere).

But the canonical hasher (`write_canonical_wat`'s Symbol arm, `hash.rs:165-173`)
emits the **raw per-process `ScopeId` u64s**:
```rust
out.extend_from_slice(&(ident.scopes().len() as u32).to_le_bytes());
for scope in ident.scopes() { out.extend_from_slice(&scope.as_u64().to_le_bytes()); }
```
`fresh_scope()` is a monotonic process-global counter (`identifier.rs:61`), so the
SAME macro-using program expanded in two processes gets DIFFERENT scope IDs →
different canonical bytes → **different hashes.** Two nodes running identical code
compute different identities → consensus breaks; the cache misses across runs. The
caveat admits it: *"deterministic within a run but not across runs."*

The probe proves it: two programs identical up to scope renaming hash to two
different 32-byte digests at HEAD.

This is the third keying site of the hygiene class. 249.5b/d made **runtime**
resolution scope-aware; 249.5e made **check** resolution scope-aware; this stone
makes **hash identity** scope-aware-AND-deterministic. Closing it completes the
class: no identifier-consuming site left name-only or non-deterministic.

## What it delivers

Hash identity invariant to the absolute scope-id values, while PRESERVING scope
STRUCTURE. Two expansions of the same program hash equal; capture vs non-capture
(different scope structure) still hash differently.

## The contract decision (pinned)

**Before emitting scope IDs, renumber them to canonical indices in first-appearance
DFS order, threaded across the whole program.**

- A `ScopeRenumber` (private to `hash.rs`): a `HashMap<ScopeId, u64>` + a `next`
  counter. `canonical(s)` returns the existing index or assigns `next++`.
- Created ONCE per `canonical_edn_wat` / `canonical_edn_program` call and threaded
  through `write_canonical_wat` so a scope shared across forms gets ONE canonical
  index program-wide.
- The Symbol arm emits `renumber.canonical(scope)` (a `u64` index) instead of
  `scope.as_u64()`. Within a symbol's `BTreeSet<ScopeId>`, iteration stays sorted
  (ascending raw id) — deterministic first-appearance order.

**Why this is correct for the real case:** the expander assigns scopes via the
monotonic `fresh_scope`, so two runs of the same program differ by a constant
OFFSET (order-preserving renaming). An order-preserving renaming yields identical
canonical indices → identical hash. And it is a RENUMBER, not a STRIP: distinct
scope structure produces distinct index sequences → distinct hash (the
discrimination guard).

## Sites (grounded this session)

- `src/hash.rs:135` `write_canonical_wat(ast, out)` → thread a `&mut ScopeRenumber`.
- `src/hash.rs:165-173` the Symbol arm → emit `renumber.canonical(scope)`.
- `src/hash.rs:113` `canonical_edn_wat` + `:125` `canonical_edn_program` → create
  the renumberer and pass it in (program-wide for the multi-form case).
- `src/hash.rs:44-65` the "Hygiene-scope caveat" doc → RETIRE it (the deferral is
  closed; replace with a statement that hashing renumbers scopes canonically, with
  the cross-run determinism claim now TRUE).

## Out of scope = rejected (affirmative cuts)

- **Re-keying the runtime/check by canonical indices** — NO. Runtime/check key by
  `env_key` over the RAW scopes (correct within a process, which is all they need).
  Canonical renumbering is a HASH-TIME concern only (cross-process identity). The
  two keyings are deliberately separate: `env_key` for in-process resolution,
  canonical-renumber for cross-process identity.
- **Persisting/interning canonical scope IDs back into the AST** — NO. The
  renumbering is computed transiently during serialization; the AST keeps its raw
  scopes. No mutation of `Identifier`.

## Probe (committed, RED at HEAD)

`tests/probe_hash_scope_renumber.rs`:
- `renamed_scopes_hash_equal` (THE BUG) — two programs identical up to scope
  renaming must hash EQUAL. RED at HEAD; GREEN after. `#[ignore]`'d for
  STRIKE-READY; the strike un-ignores it.
- `distinct_scope_structure_hashes_differently` (DISCRIMINATION GUARD) — different
  scope structure must hash DIFFERENTLY. GREEN at HEAD and after — proves the fix
  renumbers canonically, never strips. Stays live.

Verified this session: bug RED, guard GREEN, at HEAD.

## Trap-door

A lib test asserting a GOLDEN hash byte-value over a *scoped (macro-expanded)* AST
would change (those bytes were non-deterministic before — the old value was garbage;
update it). Non-macro ASTs (empty scope sets) emit ZERO scope bytes → renumberer
never assigns → hashes byte-identical to before → no change. So only broken-anyway
macro hashes move. STOP-3 surfaces any lib regression for assessment.

## Decomposition

One atomic strike, self-contained in `hash.rs` (+ the probe un-ignore). The
renumberer + the threaded signature land together; splitting leaves a half-threaded
hasher that doesn't compile.
