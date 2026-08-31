# DESIGN — STONE: `@see` can cross the boundary

> **Builder, 2026-08-31:** *"i was expecting we'd add the metadata map such that `sort$native` can
> express `@see sort` …. so.... i think declared is the only real answer here...."*
>
> Ruled: **declared**, not merely *exists*.

## Why "declared" is the right rule, and not a stricter one for its own sake

`@see` points a reader **at documentation**. A target that exists but declares nothing is a dead link
wearing a name — the reader follows it and arrives nowhere. So the gate's question is not *"is this a
name?"* but *"does this reference lead somewhere?"*, and only a declaration answers that.

★ It also keeps the gate honest about its own job: `all_see_fqdns_resolve_to_registered_intrinsics`
exists to catch a **dangling** reference. Under *exists*, `@see` would silently stop meaning "has
docs" — the gate would keep its name and quietly change its meaning.

## The blocker was never `sort` — the gate has no symbol table

```rust
pub(crate) fn check_see_refs() -> Vec<String> {
    let reg = crate::intrinsic::registry();      // ← the ONLY store it can reach
    …  if reg.lookup_entry(see_fqdn).is_none() { dangling.push(…) }
```

★ This is the structural difference from `metadata-of`. `metadata-of` consults both stores because it
runs **inside a program**, holding a `SymbolTable`. `check_see_refs` runs as a Rust `#[test]`,
**outside any program** — the wat store does not exist at that moment, because loading is what builds
it.

## THE ONE CONTRACT DECISION — pinned

**The gate resolves a `@see` target against BOTH stores, and a wat target must be DECLARED.**

```
registry().lookup_entry(target).is_some()                    a Rust intrinsic
    OR
startup_bare()'s symbols().binding_metadata[target]          a wat verb …
    carries an AXIS_DECLARATION_KEY                          … that DECLARES
```

⚠ **`binding_metadata.contains_key` alone is NOT the test.** A capability-only `{:restricted-to […]}`
map is in that table and declares nothing — accepting it would make `@see` point at a verb with no
documentation, which is the exact dead link this rule exists to forbid. **Reuse the same
`AXIS_DECLARATION_KEYS` predicate** the storage door and the reflection surface already use; a third
notion of "is this a declaration" is how the three drift apart.

## What ships

1. `:wat::core::sort` and `:wat::core::sort-by` declare metadata maps — the public ordering surface,
   which is what `sort$native` should point a reader at.
2. `check_see_refs` gains the second store via `startup_bare()` (`src/freeze.rs:1185` — a
   `FrozenWorld` with the stdlib loaded and no user program; `world.symbols()` reaches
   `binding_metadata`).
3. `sort$native` regains `@see :wat::core::sort`, which it lost when the gate went red.

## Out of scope = REJECTED (not deferred)

- **Declaring the other 407 wat verbs.** This stone declares the two `sort$native` needs. `@see` into
  the wat half stays narrow until a corpus migration, and that is honest — it is a real consequence
  of ruling *declared*, not a gap to paper over.
- **`:layer`.** Untouched, still not guessed.
- **Widening `@see` to types, macros, or special forms.** The gate's population is intrinsic entries;
  this stone adds a second store, not a second kind of target.

## THE FOUR QUESTIONS — flat YES/NO

| option | Obvious? | Simple? | Honest? | Good UX? | verdict |
|---|:---:|:---:|:---:|:---:|---|
| **both stores; a wat target must DECLARE** | YES | YES | YES | YES | ✅ **ADMITTED** |
| accept any name in `binding_metadata` | YES | YES | **NO** | — | ⛔ **DISQUALIFIED** |
| accept any *defined* wat verb (exists) | YES | YES | **NO** | — | ⛔ **DISQUALIFIED** |
| leave `@see` registry-only; drop the reference | YES | YES | **NO** | — | ⛔ **DISQUALIFIED** |

- **any-name-in-the-table Honest? NO** — a `{:restricted-to …}` map declares nothing; the link would
  dangle into a verb with no docs while the gate reported success.
- **exists Honest? NO** — the gate would keep the name *"resolve"* while silently meaning *"is a
  name"*. `@see` would stop implying documentation, and nothing would say so.
- **drop-the-reference Honest? NO** — it hides a real limitation of the doc surface by removing the
  evidence, which is what the current state already does.

## Acceptance

| what | command | expected |
|---|---|---|
| ★ `sort$native` can point at `sort` | `@see :wat::core::sort` restored | gate green |
| a wat target that DECLARES resolves | `sort` after its metadata map | resolves |
| ⛔ a wat target that does NOT declare still dangles | `@see` at any undeclared wat verb | **still flagged** — the rule must bite |
| capability-only maps are not targets | `@see` at a `{:restricted-to …}` verb | flagged as dangling |
| `sort`/`sort-by` still work | the public surface | `[1 2 3]` · `[3 2 1]` |
| floor | `scripts/floor.sh`, exit read UNPIPED | 5109/5109, 0 failed |
| clippy | `cargo clippy --release --all-targets -- -D warnings` | 0 |
