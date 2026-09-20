# RULING — `wat.type` must exist PROPER. It is a registry thing, so it is 255.

**Builder's ruling, 2026-09-20.** Recorded with the measurements that size it. **Queued behind
251.8d** — see *When* below.

> *"wat.type doesn't exist?..... it must be able to hold… wat.type/i64 wat.type/f64 wat.type/string
> wat.type/keyword wat.type/set wat.type/map wat.type/vector wat.type/u8 wat.type/list
> wat.type/bool… wat.type not being real is a serious flaw… we will grow all of rust's scalers
> into this."*
>
> *"i don't know if all of the names i want are ready.. i think we have hashmap and persistentmap…
> Vector vs vec vs vector… i don't know if we're ready for the end state names.. **but wat.type must
> exist proper**."*
>
> *"uhm... i say this is a registry thing.. so 255 i guess."*

## ⛔ THE FLAW, IN THE CODE'S OWN WORDS

`src/types.rs:632`:

```rust
/// Canonicalize `:wat::type::X` → `:wat::core::X`. Every store answers
/// `false` for a `:wat::type::` spelling; callers must not forget this.
fn canonicalize_type_kw<'b>(kw: &'b str) -> Cow<'b, str> {
    match kw.strip_prefix(":wat::type::") {
        Some(tail) => Cow::Owned(format!(":wat::core::{tail}")),
        None => Cow::Borrowed(kw),
    }
}
```

⭐ **`wat.type` has ZERO members.** It is a `format!` that redirects into `wat.core`, plus a doc
comment asking every caller to remember. *"Every store answers `false` for a `:wat::type::`
spelling"* is the defect, written down at the site.

**Three sites, and it has SPREAD** — `types.rs:635`, `:5121`, `:5394`. `docs/SEAM.md` recorded *"a
`strip_prefix("wat::type::")` at exactly two sites … an ALIAS, not a namespace"*; that was 2, it is
now 3.

## MEASURED — what `wat.type` holds today

**It is case-sensitive passthrough, so half the builder's list fails:**

| works | fails | why it fails |
|---|---|---|
| `i64` `f64` `String` `keyword` `Vector` `u8` `bool` `nil` `char` | `string` | the type is `String` |
| | `vector` | the type is `Vector` |
| | `set` `map` | the types are `HashSet` / `HashMap` |
| | `list` | **arc 118 renamed `List` → `Seq`** |

**Rust scalar coverage — 6 of 18:**

```
HOLDS  : i64  u8  f64  bool  char  String
MISSING: i8 i16 i32 i128 isize  u16 u32 u64 u128 usize  f32  str
```

⛔ **And a member exists in one position but not the other:** `:wat::type::Vector` **annotates
fine** but is an `UnresolvedReference` **in call position**. A real namespace cannot do that.

## ⭐ THE CUT — the container becomes real; the contents are deferred

The builder's own split, and it is what makes this strikeable now:

| | |
|---|---|
| **today** | `wat.type` = a `format!` at 3 sites. **0 members.** |
| **THIS ARC** | `wat.type` is a real namespace whose members are the builtin types **under whatever names they have now**. Membership is queryable. One position, not two. |
| **LATER, SEPARATELY** | curate: short names (`set`/`map`/`vector`), the case convention (`Vector` vs `vec` vs `vector` — arc 109's territory), `HashMap` vs `PersistentMap`, and the 12 missing scalars |

⭐ **Making the container real makes the deferred question ANSWERABLE.** Once `wat.type` has members,
*"is `wat.type/set` a thing?"* has a yes/no, and the curation becomes a visible diff instead of an
argument.

## Why this belongs to 255, and why 255 is now FREE

- `docs/SEAM.md` already records it as 255's debt: *"255 … owes `wat.type` a real home."*
- `types.rs:697` names the population as 255's: *"Deliberately unchanged by the builtin-leaf
  population (**stone 255-builtin-registry**)."* Part of the registry (`builtin_names`) exists.
- ⭐ **255's recorded blocker is DISCHARGED.** The SEAM's chain reads `296/298 → 255 → 251` with 251
  *"BLOCKED on #95"* and ⛔ *"THE REGISTRY UNBLOCKS THE CLOJURIFICATION, NOT THE REVERSE."*
  **#95 was closed by 251.8c on 2026-09-19** — by the check-path fix, not by the registry. So the
  chain inverted: 251 went ahead without 255, and 255 is free.

## ⛔ WHEN — after 251.8d, and the builder agreed

> *"you are arguing for 8d first... so we continue but ........... wat.type needs to be proper soon"*

Three reasons, the third measured:

1. **8d is semantically inert; this is a language change.** Mixing them makes both unreviewable and
   destroys 8d's whole defence ("no semantics move").
2. **8d unblocks the branch merges.** Several large divergent branches are not merge-ready and need
   8d's codemods.
3. ⭐ **8d makes THIS arc cheaper.** The codemod emits **~13,600 `wat.type/X` sites across 20
   distinct names** (`wat.type/i64` ×6,766, `wat.type/String` ×3,064, `wat.type/nil` ×1,725,
   `wat.type/bool` ×1,353, `PersistentMap` ×260 …). After the flip they are in **ONE spelling**.
   Doing this first means fixing the namespace in two dialects and re-fixing whatever the codemod
   then rewrites.

⚠ **8d is NOT blocked by this flaw and does not worsen it semantically** — the codemod does a
namespace *swap that preserves the name*, and all 20 emitted names annotate today. What it does is
make the debt **13,600× more load-bearing**, which is the argument for striking this arc soon rather
than an argument against the flip.

## Acceptance, when it is struck

1. `wat.type` has **members**. A query answers *"is this a valid namespace?"* with a bool, and
   *"what is in it?"* with a set. ⛔ **`contains()` must stop answering `false` for a `:wat::type::`
   spelling** — that sentence in `canonicalize_type_kw`'s doc is the thing being deleted.
2. **One position, not two.** Whatever `wat.type/Vector` means, it means it in annotation AND call
   position — or it is refused in both, with the same error.
3. The **three** `strip_prefix(":wat::type::")` sites collapse to the registry. A fourth appearing
   later is a regression a gate should catch.
4. **A non-vacuity control**: a name that is NOT a member must be refused, and the refusal must say
   *"not a member of `wat.type`"* — distinct from *"unknown type"*, or a typo and a real absence read
   the same.
5. Floor green, clippy 0, census `no STOP-8`.
