# Strike seq-1a — split `mappable()` into `mappable()` + `ordered()`

**Arc 278 collection campaign. Pure refactor — ZERO behavior change. Floor must stay identical (lib 953/36/1, warnings 26).**

## Why

The seq HOF fills (List, WatAstList, HashSet) are next. List + WatAstList are full
ordered sequences (all 8 HOFs). HashSet, by nature, supports the **order-agnostic**
element transforms (`map`/`filter`/`foldl`/`foldr` — produce a set / fold in
unspecified order) but **not** the **order-dependent** sequence ops
(`reverse`/`take`/`drop`/`concat` — there is no defined element order to slice or join).

Today a single `mappable()` capability gates **all eight** ops. Flipping it true for
HashSet would wrongly accept `reverse`/`take`/`drop`/`concat`. So the capability must
split along the order-dependency seam — the substrate forcing the honest distinction.

## The split (grid: `docs/COLLECTION-CAPABILITIES.md`)

- **`mappable()`** — order-agnostic element transform: `map` / `filter` / `foldl` / `foldr`.
- **`ordered()`** — order-dependent sequence ops: `reverse` / `take` / `drop` / `concat`.

One `ordered()` cap (not separate `sliceable`/`concatenable`): all four share one nature
predicate — *ordered, homogeneous, variable-length sequence* — with an identical truth
table across all six containers, by nature, never diverging.

**Both caps keep today's truth table this strike** — `{Vector, PersistentVector} = true`,
all else `false`. No container gains or loses an op. The fills (1b/1c/1d) flip the bools.

## Rooms (exact)

1. `src/collection/seq_container.rs:194` — `mappable()`. Narrow its doc-comment to
   map/filter/foldl/foldr. Add a sibling `ordered()` method directly below it with the
   **same body shape** (`{Vector,PersistentVector} => true`, `List|Tuple|WatAstList|HashSet => false`)
   and a doc-comment naming reverse/take/drop/concat and the order-dependent nature.
   Update the capability-matrix table in the module header (lines 33-42): rename the
   `Mappable` column to two columns `Mappable` (map/filter/foldl/foldr) + `Ordered`
   (reverse/take/drop/concat), both `✓` for Vector/PV, `○ gap`/`∅` as today for the rest.
2. `src/collection/transform.rs` — change the gate guard from `if container.mappable()`
   to `if container.ordered()` in **reverse** (`eval_vec_reverse`, ~line 47), **take**
   (`eval_vec_take`, ~121), **drop** (`eval_vec_drop`, ~170). Update each `unreachable!`
   comment+message in those three from "mappable() gate excludes…" to "ordered() gate
   excludes…". **Leave map (~333), foldl (~397), foldr (~459), filter (~520) on
   `mappable()` unchanged.**
3. `src/collection/eval.rs:756` — `vector_concat_inner`: change `if left_container.mappable()`
   to `if left_container.ordered()`; update the `unreachable!` comment/message to "ordered()".
4. `src/collection/infer.rs:561` — `extract_seq_elem`: add a parameter
   `cap: fn(SeqContainer) -> bool`, replace the hardcoded `if !container.mappable()` (line
   575) with `if !cap(container)`. Update the doc-comment to say the caller supplies the
   capability gate. At the 9 call-sites (636, 706, 773, 853 = map/filter/foldl/foldr →
   pass `SeqContainer::mappable`; 928, 979, 1041, 1108, 1113 = reverse/take/drop/concat →
   pass `SeqContainer::ordered`). Verify each call-site's `infer_*` fn by name before
   choosing the cap.

## Discipline (non-negotiable)

- The gate is the **genuine** `Some(c) if c.ordered() => match c { …named arms…, <gated-out
  named arms> => unreachable!("ordered() gate excludes …") }` form. The capability DRIVES
  the accepted set.
- Do **not** silence any signal: no `#[allow(dead_code)]`, no making items `pub` to dodge
  dead_code, no `debug_assert!(c.cap())` shadow, no `_ =>` catch-all. Every match over
  `SeqContainer` stays exhaustive with named arms.
- `ordered()` is consumed by USE (the four gate sites in rooms 2+3 and the four checker
  call-sites in room 4) — so it will not be dead_code. If `cargo build` reports `ordered()`
  as dead, a gate site was missed — fix the site, never silence the warning.

## STOP triggers

- **STOP-1** if any of the 8 runtime gate sites or 9 checker call-sites does not exist at
  the named location — report the real location; do not guess.
- **STOP-2** if making the change forces a behavior difference (a test flips) — that means
  a cap truth-table value changed; this strike must be behavior-identical. Report it.

## Verification (the sonnet runs these and reports raw output)

- `cargo build 2>&1 | tail -5` — compiles.
- `cargo build 2>&1 | grep -c warning` — must be **26** (unchanged).
- `cargo test --lib 2>&1 | tail -3` — must be **953 passed; 36 failed; 1 ignored** (identical floor).
