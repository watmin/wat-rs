# DESIGN — STONE: `src/record/` — the aggregate family gets the home its edge was built for

> **Builder, 2026-09-01:** *"we continue."*
> `[[NOTE-partire-RECAST-on-the-current-runtime]]` item 2. **1,087 lines, 17 items.**

## ★ THE EDGE HAS BEEN WAITING, AND IT SAYS SO IN WRITING

`src/intrinsic/record.rs:12`, verbatim:

> *"`src/runtime.rs` (all seven now `pub(crate)` so this module can reach them) — **no body**
> [moves]"*

That visibility bump was made **in anticipation of a home that was never built.** The edge exists,
registers all ten aggregate verbs, and delegates every one back into the megafile.

```
10 of the 17 are its delegate targets:
  eval_struct_new · eval_variant · eval_struct_field · eval_aggregate_new · eval_kwargs_construct
  eval_to_core_record · eval_record_field_at · eval_record_to_map · eval_record_same_data
  eval_record_assoc
7 are their private helpers:
  construct_aggregate · project_surface_attrs · parse_projection_args · eval_record_q
  eval_list_q · record_field_map · record_assoc_inner
```

## What moves — 17 items, enumerated, re-derived on the CURRENT file

```
 7370   67  eval_struct_new         9086   39  eval_aggregate_new     9706   59  record_field_map
 7458  104  eval_variant            9131   95  construct_aggregate    9774   21  eval_record_to_map
 7692   79  eval_struct_field       9241  162  eval_kwargs_construct  9809   24  eval_record_same_data
                                    9415   41  project_surface_attrs  9851  175  record_assoc_inner
                                    9459   62  parse_projection_args 10029   23  eval_record_assoc
                                    9528   16  eval_to_core_record
                                    9557   73  eval_record_field_at
                                    9635   24  eval_record_q
                                    9670   23  eval_list_q
                                                                             1,087  TOTAL
```

⚠ **Two clusters, not one span** — `7370–7770` and `9086–10052`. Anything between them is not this
stone's.

## ⛔ `eval_retag_op` (7584) SITS BETWEEN `eval_variant` AND `eval_struct_field` AND IS NOT RECORD

Its sole caller is **`src/intrinsic/kernel/serve.rs`**. It reads like a record verb — retagging a
variant — and belongs to `kernel::serve` by caller evidence. The re-cast named it as an EXCLUDED item
under `record` for exactly this reason, and it is the eighth intruder this campaign has found sitting
inside a proposed range.

★ **The cast flagged it; the orchestrator verified it; the brief fences it.** That is the loop working
— the first casts' intruders were caught only after a stone shipped.

## THE ONE CONTRACT DECISION — pinned

**`src/record/` splits by ROLE, and the ten registered verbs land beside the private helpers that
serve them.** A helper is not a separate concern from the verb it exists for — `construct_aggregate`
belongs with `eval_aggregate_new`, `record_assoc_inner` with `eval_record_assoc`,
`parse_projection_args` with `project_surface_attrs`.

Proposed: `construct` (struct-new · variant · aggregate-new · construct_aggregate ·
kwargs-construct) · `access` (struct-field · record-field-at · record? · list?) · `project`
(project_surface_attrs · parse_projection_args · to-core-record) · `update` (record_field_map ·
to-map · same-data? · record_assoc_inner · record-assoc). ⚠ **The rider verifies this against the
bodies** — `reflect`'s `verbs.rs` shipped 12 of 13 because exactly this kind of assignment was wrong.

## ★ THE BLAST RADIUS IS 20 SITES IN 2 FILES

```
src/intrinsic/record.rs      18    the edge — every delegate re-points
src/intrinsic/holon/atom.rs   2
```

Everything else is in-file. ⚠ **`construct_aggregate` has exactly one caller outside its own
cluster — `runtime.rs` itself.** Nothing else in the tree constructs an aggregate directly, which is
what makes this domain cleanly liftable.

## ★ THE PREDICTION — falsifiable

```
runtime.rs      25,799  ->  ~24,750   (-1,050)
src/record/     ~1,100 lines, 4-5 files split by ROLE
eval_retag_op   UNTOUCHED, still in runtime.rs
20 sites        crate::runtime::X -> crate::record::<role>::X
src/record/     imports crate::value / crate::ast / crate::span directly; crate::runtime:: only for
                genuine residents; crate::intrinsic NEVER
behaviour       every aggregate verb identical
```

## Out of scope = REJECTED (not deferred)

- **`eval_retag_op`.** Not record; proven by its sole caller. `kernel::serve`'s business when the
  kernel family moves.
- **The other five re-cast modules** — the kernel family · the died-error cluster (home deliberately
  unassigned) · `holon::outcome` · `option`/`result` · the purity classifier. One stone each.
- **`Nature` / the aggregate TYPE definitions** (`src/types.rs`). This stone moves the *verbs* that
  construct and read aggregates, not the type system that declares them.
- **numeric stone 2's promotion lattice.** Unblocked as of the last stone; still its own work.

## THE FOUR QUESTIONS — flat YES/NO

| option | Obvious? | Simple? | Honest? | Good UX? | verdict |
|---|:---:|:---:|:---:|:---:|---|
| **`src/record/`, split by role, helpers beside their verbs** | YES | YES | YES | YES | ✅ **ADMITTED** |
| take the span `7370–10052` whole | YES | YES | **NO** | — | ⛔ **DISQUALIFIED** |
| helpers into a `src/record/helpers.rs` | YES | YES | **NO** | — | ⛔ **DISQUALIFIED** |
| put the bodies in `src/intrinsic/record.rs` | **NO** | YES | **NO** | — | ⛔ **DISQUALIFIED** |
| one flat `src/record.rs` of 1,087 lines | YES | **NO** | YES | — | ⛔ **DISQUALIFIED** |

- **whole-span Honest? NO** — it takes `eval_retag_op` and ~1,300 lines of unrelated code between the
  two clusters. Eight intruders in eight modules is not bad luck; it is what a range IS.
- **`helpers.rs` Honest? NO** — `helpers` is the name a module takes when nobody named its concern.
  A helper's reason to change is its verb's; separating them splits one concern by visibility.
- **bodies-into-the-edge Obvious? NO / Honest? NO** — collapses edge and impl. The builder corrected
  the orchestrator on precisely this, and eleven existing pairs are the counter-example.
- **one-flat-file Simple? NO** — relocates the megafile problem.

## Acceptance

| what | command | expected |
|---|---|---|
| ★ `eval_retag_op` did not move | `grep -c "fn eval_retag_op" src/runtime.rs` | **1** |
| the megafile sheds it | `wc -l src/runtime.rs` | ~24,750 |
| split by role, no `helpers.rs` | `ls src/record/` | named concerns only |
| ★ no facade imports | each file's `use` block | `crate::value::` direct |
| the impl does not know its edge | `grep -c "crate::intrinsic" src/record/*.rs` | 0 |
| the 20 sites re-point | `crate::runtime::` for the 17, outside `src/record/` | 0 |
| behaviour unchanged | every aggregate verb | identical |
| floor | `scripts/floor.sh`, exit read UNPIPED | 5114/5114, 0 failed |
| clippy | `cargo clippy --release --all-targets -- -D warnings` | 0 |
