# BRIEF — Stone 249.5f: canonical scope renumbering at hash time

## The work (one paragraph)

The canonical hasher emits the raw per-process `ScopeId` u64s, so the same
macro-using program hashes differently across runs — breaking `hash(expanded AST) IS
identity` (cross-node consensus + content-addressed caching). Renumber the scope IDs
to canonical indices in first-appearance DFS order, threaded across the whole
program, before emitting them. Identical-up-to-renaming programs then hash equal;
distinct scope structure still hashes differently (it is a RENUMBER, not a STRIP).
Self-contained in `src/hash.rs`.

## The contract (pinned)

A private `ScopeRenumber` in `hash.rs` — a `HashMap<ScopeId, u64>` + a `next: u64`
counter; `canonical(&mut self, s: ScopeId) -> u64` returns the existing index or
assigns `next` then increments. Created ONCE per `canonical_edn_wat` /
`canonical_edn_program` call, threaded through `write_canonical_wat`, so a scope
shared across forms gets one canonical index program-wide. The Symbol arm emits the
canonical index instead of `scope.as_u64()`.

## Read in order (the rooms)

1. **`src/hash.rs:44-65`** — the "Hygiene-scope caveat" doc-comment. Read it; you are
   closing it.
2. **`src/hash.rs:135`** `fn write_canonical_wat(ast: &WatAST, out: &mut Vec<u8>)` —
   add a `renumber: &mut ScopeRenumber` parameter; thread it through EVERY recursive
   call (List, Vector, StructPattern arms — wherever `write_canonical_wat` recurses).
3. **`src/hash.rs:165-173`** the `WatAST::Symbol(ident, _)` arm — replace
   ```rust
   for scope in ident.scopes() { out.extend_from_slice(&scope.as_u64().to_le_bytes()); }
   ```
   with
   ```rust
   for scope in ident.scopes() { out.extend_from_slice(&renumber.canonical(*scope).to_le_bytes()); }
   ```
   (the `len` prefix stays; `ident.scopes()` iterates the `BTreeSet` in sorted order —
   keep that order, it is the deterministic first-appearance order within a symbol).
4. **`src/hash.rs:113`** `canonical_edn_wat` + **`:125`** `canonical_edn_program` —
   create `let mut renumber = ScopeRenumber::new();` and pass `&mut renumber` into
   `write_canonical_wat`. For `canonical_edn_program`, ONE renumberer spans all forms.
5. **`src/hash.rs` — define `ScopeRenumber`** (near the top of the impl, private). Use
   `std::collections::HashMap<ScopeId, u64>`.
6. **`src/hash.rs:44-65`** — RETIRE the caveat doc: replace it with a short statement
   that scoped symbols are renumbered to canonical first-appearance indices at hash
   time, so canonical-EDN is deterministic ACROSS runs (the cross-node/cross-run
   determinism claim now holds). Cite Stone 249.5f.
7. **`tests/probe_hash_scope_renumber.rs`** — remove the `#[ignore = ...]` on
   `renamed_scopes_hash_equal` (the only edit to this file). It must then pass; the
   discrimination guard `distinct_scope_structure_hashes_differently` must STAY green.

## Implementation sketch

```rust
use crate::scope::ScopeId;
use std::collections::HashMap;

struct ScopeRenumber { map: HashMap<ScopeId, u64>, next: u64 }
impl ScopeRenumber {
    fn new() -> Self { Self { map: HashMap::new(), next: 0 } }
    fn canonical(&mut self, s: ScopeId) -> u64 {
        if let Some(&i) = self.map.get(&s) { return i; }
        let i = self.next; self.next += 1; self.map.insert(s, i); i
    }
}
```
Then thread `&mut ScopeRenumber` through `write_canonical_wat`'s signature and every
recursive call; the Symbol arm calls `renumber.canonical(*scope)`.

## Blast radius (bounded)

`src/hash.rs` ONLY (+ the one `#[ignore]` removal in the probe). No new public API
(`ScopeRenumber` is private; the `canonical_edn_*` / `hash_canonical_*` signatures are
unchanged). No mutation of `Identifier` or the AST.

## STOP triggers (rejection criteria — surface, do not improvise)

- **STOP-1:** the discrimination guard `distinct_scope_structure_hashes_differently`
  goes red — that means the renumbering collapsed distinct structure (a strip, not a
  renumber). Surface it; do not weaken the guard.
- **STOP-2:** a `write_canonical_wat` recursion site is missed (the renumberer isn't
  threaded everywhere it recurses) — the build will catch the signature mismatch;
  thread it through all arms, do not add a second renumberer.
- **STOP-3:** a lib test regresses. If it is a GOLDEN-hash assertion over a
  *scoped/macro* AST, the old value was non-deterministic — surface it with the test
  name so the orchestrator assesses updating it. Do NOT silently change a test to
  pass, and do NOT touch any non-hash test.

## Verify (the load-bearing checks)

```
cargo build --release
cargo test --release --test probe_hash_scope_renumber     # → 2 passed (bug + guard)
cargo test --release --test probe_macro_hygiene_capture --test probe_argspec_rest_param_hygiene --test probe_check_scoped_param_resolution   # → 2 + 1 + 2 = 5 passed (unchanged)
cargo test --release --lib -p wat                         # → no regressions vs 907/0/1
```

## Comparable prior result (copy for shape)

Stone 249.5b/d/e — the runtime + check sides of the same hygiene class, each a
representation/keying change verified by a hygiene probe flipping RED→GREEN with a
discrimination/control guard. This stone is the third and final keying site (hash
identity), same shape.
