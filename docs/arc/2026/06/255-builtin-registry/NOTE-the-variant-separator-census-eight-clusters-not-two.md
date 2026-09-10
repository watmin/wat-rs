# NOTE — the variant separator: EIGHT clusters, not two

**Measured 2026-09-10 on the tree at `b0c4eb7d5`** (floor 5318/5318, clippy 0), by READING each
site, not by pattern-matching them.

## Why this exists

`DESIGN-the-codemod-asks-and-the-flip-is-a-PAIR.md` claimed the separator was *"spelled in exactly
these two function bodies."* Landing ③a refuted that within the hour, and a rider's STOP-5 answer
had checked the wrong set (callers of `variant_parent_enum`, not other decomposers). This is the
census that should have preceded the claim.

## Method, and its correction

```
raw separator ops (identifier::path/leaf · contains/rfind/rsplit/split on "::") ..... 55 lines
heuristic flag  (context mentions Enum / variant) ................................... 20
READ and CONFIRMED as variant-separator sites ....................................... 8 clusters
```

⚠ **The heuristic over-flagged.** `src/edn/render.rs:4619`'s `variant_tag` looked variant-shaped and
is not: it splits the enum's own path into NAMESPACE + enum leaf (`wat.core` + `Option`) and then
composes through `compose_variant_render`. That is already correct — a namespace split feeding the
variant composer. Seven of `render.rs`'s twenty-one flagged lines are that shape.

`[[feedback_a_pattern_that_matches_a_subset_is_not_a_census]]` — the flag was the search, not the
answer.

## THE EIGHT

```
✓ 1  src/types.rs            variant_parent_enum            ROUTED through decompose_variant (③a)
  2  src/record/construct.rs:261,264,265   the variant CTOR path — guard + path + leaf +
                                          `types.get(...) -> TypeDef::Enum`. Separator spelled
                                          THREE TIMES IN THREE LINES.
  3  src/check.rs:13689,13692             `literal_enum_variant_ctor` — same shape again,
                                          plus its own `is_variant` scan.
  4  src/check.rs:13896,13899,13900       a SECOND checker decomposer, same guard-then-split.
  5  src/closure_extract.rs:1589,1590     dependency recording — `contains("::")` then
                                          `path(name)` then `TypeDef::Enum`.
  6  src/match_arm.rs:134                 `is_namespaced_variant(path) { path.contains("::") }`
                                          — a variant-shape PREDICATE that is nothing but the
                                          separator.
  7  src/rete/expr_ir.rs:604              `k.starts_with(':') && k.contains("::")` -> `Pat::Variant`
                                          — the separator AS the variant discriminator.
  8  src/rete/expr_ir.rs:1321             MIXED: composes via `compose_variant` (the door) and
                                          then decomposes via `leaf(name)`. After a flip,
                                          `leaf("…Option.Some")` yields `Option.Some`, not `Some`,
                                          and this comparison silently changes meaning.
```

★ Cluster 8 is the sharpest illustration of why the pair must be complete: it already uses the
composition door, and it is STILL broken by the flip, because its other half went to the general
accessor.

## What is NOT in scope

The other ~35 lines are genuine NAMESPACE splits — `a::b::c` into path and leaf — and they must
stay general. `identifier::path`/`leaf` split every namespaced name in the substrate; giving them a
variant-specific separator would break each one.

★ And no hand-rolled `rfind("::")`/`rsplit("::")` exists outside `identifier.rs` — the
one-name-grammar lint holds. Every split above goes through the grammar's accessors, which is
exactly why they were findable at all.

## What this means for the flip

⛔ **③b cannot land until clusters 2–8 route through `decompose_variant`.** Seven sites, each a
mechanical repoint, each behaviour-preserving while the separator stays `::` — the same shape ③a
already proved on cluster 1.

Only then is the separator one decision, and only then is *"the flip is a line in each of two
bodies"* a true sentence rather than an aspiration.

⚠ **The count is 8 because I read them.** It was 2 when I asserted it, and 20 when a pattern
guessed. Anyone re-deriving it should read, not grep.
