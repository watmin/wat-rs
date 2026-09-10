# NOTE — the variant separator: EIGHT clusters, not two

> ⛔ **CORRECTED 2026-09-10 (same day, at `e7c07ab97`). THE COUNT IS NOT EIGHT — IT IS NINETEEN.**
> Everything below was measured honestly and is still true of what it looked at. **Its predicate
> had a hole**, and the hole is where the majority hid. The original text is preserved beneath,
> unedited, because the shape of the miss is the lesson.

## ⛔ WHAT THE PREDICATE COULD NOT SEE

The census searched `identifier::path`/`leaf` plus `contains` / `rfind` / `rsplit` / `split` on
`"::"`. **It never searched `rsplit_once("::")`.** There are **14** of them outside
`identifier.rs`, and **9 are variant decomposers** — each binding `(enum_path, variant)` or
`(prefix, variant_name)` and looking the parent up as a `TypeDef::Enum`:

```
src/check.rs:6881    (prefix, variant_name)          src/rete/expr_ir.rs:818   -> TypeDef::Enum
src/check.rs:7007    (enum_path, _) -> TypeDef::Enum src/rete/validate.rs:1227
src/check.rs:7179    (prefix, variant_name)          src/rete/validate.rs:1483
src/check.rs:7436    (prefix, variant_name)          src/rete/purity.rs:804    -> TypeDef::Enum
src/check.rs:7682    (prefix, variant_name)
```

The other five are honest non-variant splits (`edn_doc.rs:259` ns+type, `clause.rs:172,365` the
rete `Type::op` constraint head, `purity.rs:2747` ns grouping, `render.rs:414` a generic leaf).

## ⛔ AND THREE HAND-ROLLED VARIANT **COMPOSERS** SURVIVED THE COMPOSITION DOOR

The door stone collapsed 15 `format!`s into 1. Three variant compositions were not among them:

```
src/runtime.rs:3196   format!("{}::Op::{}",    protocol_fqdn, variant)
src/runtime.rs:3198   format!("{}::Reply::{}", protocol_fqdn, variant)
src/runtime.rs:8638   format!(":{}::{}", fv.enum_class, fv.variant)   -- Value::ForeignVariant
```

(`format!("{}::Op", surface.name)` — `types.rs:3224`, `check.rs:17098` — is the ENUM's own type
path, not a variant. It stays `::` after the flip. These three do not.)

```
variant DECOMPOSERS ..... 17   (1 routed by ③a, 16 unrouted)
variant COMPOSERS ........ 3   (hand-rolled, outside the door)
UNROUTED VARIANT SITES ... 19
```

## ⛔⛔ THE ROOT — THE LINT'S BAN LIST IS A HAND-LIST, AND THIS IS ITS HOLE

The claim below — *"no hand-rolled `rfind`/`rsplit` exists outside `identifier.rs` — the
one-name-grammar lint holds"* — is true **only of the five spellings the lint enumerates**:

```rust
const BANNED: &[&str] = &[ r#"rfind("::")"#, r#"rsplit("::")"#,
                           r"rfind('/')", r"rsplit_once('/')", r"strip_suffix(''')" ];
```

`rsplit_once('/')` is banned. **`rsplit_once("::")` appears NOWHERE in that file** — not in
`BANNED`, and not in the module doc's *"⚠ Not banned, deliberately"* list either. It is a hole,
not a decision. The lint's own doc asserts the five *"are precise … measured, not assumed"*, and
they were — against the 33-site census of **arc 109**. That measurement expired.
`HAERESIS EST ITERVM ROGARE`.

⚠ And a token-level ban would still miss `crates/wat-doc/src/lib.rs:1024`, which asks the same
question at the **byte** level: `k.as_bytes()[n..].starts_with(b"::")`.

★ `[[feedback_a_gate_over_two_hand_lists_is_a_hand_list]]` ·
`[[feedback_impose_the_check_and_read_the_screams]]` · the count was **2** when I asserted it,
**8** when I read a holed predicate's output, and **19** when the predicate was repaired. A census
is only ever as good as the predicate that generated its candidate set — reading each candidate
carefully cannot recover one the predicate never proposed.
`[[feedback_validate_a_search_pattern_before_trusting_its_count]]`

## Standalone separator literals outside `identifier.rs`, for scope

```
"::" as a standalone string literal ......... 72 lines
  .contains("::") 25 · .rsplit_once("::") 14 · .ends_with("::") 3 · .split("::") 2
  .starts_with("::") 1 · the rest: format!/replace/comment
b"::" (byte literal) ......................... 1
```

---

# ⬇ THE ORIGINAL NOTE, UNEDITED (its predicate is the defect above)


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
