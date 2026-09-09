# SCORE — STONE ①′: the registry gains a MEMBERSHIP facet

## Result: landed. 75 names entered the membership set.

`cargo build --release` was confirmed green before any change (`Finished` release profile,
exit 0). Only `src/intrinsic/mod.rs` was touched — 54 insertions, 2 deletions, entirely
additive (one new struct field, one new private method, one new `#[allow(dead_code)]`
`pub(crate)` query method, one new fold loop at the end of `registry()`'s builder closure).
No other file changed.

## The work, mirrored field-for-field against `TypeEnv`

`IntrinsicRegistry` (`src/intrinsic/mod.rs:529`) gained:

```
membership_names: HashSet<&'static str>   — mirrors TypeEnv.builtin_names (HashSet<String>;
                                             kept &'static str here since every other key on
                                             this registry, including `entries`'s own, is
                                             &'static str, not owned String)
register_membership(&mut self, name)      — private, mirrors register_builtin_leaf
contains(&self, name) -> bool             — self.entries.contains_key(name) ||
                                             self.membership_names.contains(name),
                                             mirrors TypeEnv::contains exactly
```

`registry()`'s builder closure, after both submission folds and the alias-axis-copy pass
(i.e. once `entries` is fully finished), now runs:

```rust
for op in crate::rete::vocabulary::RETE_OPS {
    r.register_membership(op.rete_name);
}
```

**Names only** — `op.rete_name` is the sole field read off `ReteOp`. No `IntrinsicEntry` is
constructed for any rete row; `category` and `ret` are never touched.

## Count: 75, not 74

The BRIEF and the prior SCORE both say "74." Measured directly against the current tree:

```
grep -oP '^\s+rete_name: "\K[^"]+' src/rete/vocabulary.rs | wc -l        → 75
grep -oP '^\s+rete_name: "\K[^"]+' src/rete/vocabulary.rs | sort -u | wc -l → 75  (no dupes)
grep -rn "rete_name:" src/ | grep -v src/rete/vocabulary.rs               → (empty; the
    field is assigned nowhere else, so this count is exhaustive)
```

`RETE_OPS` has grown by one row since the "74" figure was written into the BRIEF/prior
SCORE (both pre-date this stone; the table is not read-only across the whole arc, only
within specific sibling stones that said so for their own scope). The fold loop reads
`crate::rete::vocabulary::RETE_OPS` directly rather than a hardcoded count, so the
membership set tracks the table's actual size — **75 names entered `membership_names`** as
of this run, and the loop will track future growth automatically without needing a rider to
update a literal.

## `lookup_entry` — unchanged for every existing caller

```rust
pub(crate) fn lookup_entry(&self, name: &str) -> Option<&IntrinsicEntry> {
    self.entries.get(name)
}
```

Byte-for-byte the same body as before this stone (only the doc comment grew, restating the
STOP-2 asymmetry). It still consults `entries` alone. `lookup` and `all_entries` are also
untouched. `contains` is new, `#[allow(dead_code)]`, and has zero production callers — this
stone adds the facet only; wiring `resolve`'s `where`/`:then` boundary to ask it is explicitly
future work per the BRIEF ("the question `resolve` *will* ask").

Overlap check: 50 of the 75 `rete_name`s are ALSO independently registered as real
`Kind::Intrinsic`/`Kind::SpecialForm` entries under the identical FQDN (e.g.
`:wat::rete::core::and`, `:wat::rete::i64::>`) — confirmed by diffing the rete-name list
against every `#[wat_intrinsic("...")]`/`#[wat_special_form("...")]` string in `src/`. This is
harmless by design: `register_membership` just inserts into the `HashSet`, `contains` ORs
the two populations, and `lookup_entry` was never touched, so these 50 names answer exactly
as they did before — a full entry via `lookup_entry`, and (new) `true` via `contains`, same
as any of the other 25 rete-only names.

## STOP triggers — disposition

- **STOP-1** (fabricating a full `IntrinsicEntry` for a rete row): did NOT fire. No
  `IntrinsicEntry` was constructed anywhere for a `RETE_OPS` row; only `op.rete_name` (a
  `&'static str`) crosses into the registry. `category` and `ReteOp.ret` are never read.
- **STOP-2** (`lookup_entry` starts answering for membership-only names): did NOT fire.
  `lookup_entry`'s body is unchanged; it still returns `None` for any of the 25 rete-only
  names that carry no `entries` row (verified: those 25 are absent from every
  `#[wat_intrinsic]`/`#[wat_special_form]` name list).
- **STOP-3** (an existing registry caller's behaviour changes): did NOT fire. `lookup`,
  `lookup_entry`, `all_entries`, and `register` are byte-identical in logic to before this
  stone (doc-comment-only changes on `lookup_entry`); `new()` only grew to initialize the new
  field. The three targeted floor filters ran clean at the exact counts the BRIEF named:
  `p1_annotation` 10/10, `a1_one_rule` 4/4, `a2_a_variant` 15/15 — all pre-existing tests
  neither know about nor are affected by the new field.
- **STOP-4** (a second membership set appears outside the registry): did NOT fire. Exactly
  one new `HashSet` exists, on `IntrinsicRegistry` itself; nothing was added to
  `rete/vocabulary.rs`, `check.rs`, or anywhere else.

## Checks run

```
cargo build --release                                  Finished, exit 0 (pre-check)
cargo clippy --release --all-targets -- -D warnings     Finished, exit 0
cargo nextest run --release -E 'test(p1_annotation)'    10 passed, 0 failed
cargo nextest run --release -E 'test(a1_one_rule)'       4 passed, 0 failed
cargo nextest run --release -E 'test(a2_a_variant)'     15 passed, 0 failed
```

No `scripts/floor.sh`, no unfiltered `cargo nextest run`, and no corpus sweep were run, per
tier. Tree not committed.

## Honest deltas

- Names entered `membership_names`: **75** (not 74 — see the count section above; the
  discrepancy is `RETE_OPS` having grown by one row since the BRIEF/prior SCORE were
  written, not a miscount in this stone).
- `lookup_entry`: unchanged, confirmed byte-identical logic and confirmed via the three
  targeted filters that no existing caller's observed behaviour moved.
- New surface: `IntrinsicRegistry::contains` — `pub(crate)`, `#[allow(dead_code)]`, zero
  production callers as of this stone (wiring `resolve` to it is out of scope here).
- Nothing removed, renamed, or fabricated. `Category::Unreviewed` was not added.
  `ReteOp.ret` was not read anywhere in this change.
