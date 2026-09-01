# DESIGN — STONE: `src/reflect/` — the introspection surface leaves the megafile

> **Builder, 2026-09-01:** *"next module — keep going."*
>
> `src/numeric/` took 892. `src/declare/` took 3,506 and put `runtime.rs` under 30,000.
> This one takes **~2,500** — the largest remaining contiguous block.

## ⛔ FIRST: PARTIRE'S SPANS ARE NOW STALE, BY ~3,506 LINES

The declare stone removed 3,506 lines from `runtime.rs:789–4496`, so **every partire range after
that point shifted down**. `reflection` was cast as `12481–15030`; it now sits at **8096–10646**.

★ Re-derived from disk by FUNCTION NAME, not by arithmetic on the old numbers. Every range in this
DESIGN was measured against the current file. **This is why the campaign's standing rule is move by
the function list, never by span** — the spans are not merely risky now, they are provably wrong.

## What moves — 33 items, `8096–10646`

```
render      8096 eval_struct_to_form · 8189 type_expr_to_ast · 8231 binder_head_nodes
            8256 function_to_signature_ast · 8307 function_to_define_ast
            8330 type_scheme_to_signature_ast · 8361 primitive_to_define_ast
            8416 macrodef_to_signature_ast · 8471 macrodef_to_define_ast
            8496 typedef_to_signature_ast · 8529 typedef_to_define_ast
            8573 name_from_keyword_or_fn
lookup      8608 enum Binding · 8656 lookup_form · 8765 eval_lookup_define
verbs       8880 eval_signature_of_defn · 9002 eval_signature_of_fn · 9068 eval_return_type_of
            9162 eval_body_of · 9281 eval_metadata_of · 9509 require_ast_children
            9568 eval_rename_callable_name · 9764 eval_extract_arg_names
            9870 eval_extract_arg_types · 9979 eval_field_names_of · 10037 eval_field_types_of
            10078 resolve_type_keyword_arg · 10130 resolve_aggregate_def_for_reflection
match       10203 eval_form_matches · 10306 walk_match_clause · 10493 eval_forms
expand      10508 eval_macroexpand_1 · 10561 eval_macroexpand
```

## ⛔ ONE ITEM INSIDE THE RANGE IS NOT REFLECTION — and partire was right to omit it

`require_bundle` (**9486**) sits between `eval_metadata_of` and `require_ast_children`. It is **not**
in partire's list, and its callers say why:

```
src/intrinsic/holon/atom.rs:1470   require_bundle(OP, &holon_arc, h.span())?
src/intrinsic/holon/atom.rs:1521   require_bundle(OP, &holon_arc, h.span())?
```

**Both callers are the HOLON edge.** It is a holon helper living in the reflection range by
proximity. ★ This is the fourth time this campaign a line range has contained something that does
not belong to the concern — and the first time the CAST was right and the orchestrator's own
enumeration was wrong. `require_bundle` **stays in `runtime.rs`**; giving it a home is
`src/holon/`'s business, not this stone's.

## THE ONE CONTRACT DECISION — pinned

**The home is `src/reflect/`, because `src/intrinsic/reflect.rs` is already its EDGE.** This is the
architecture the builder stated: `src/intrinsic/<domain>` registers and delegates; `src/<domain>/`
implements. `src/reflect/` does not exist yet — that absence IS the defect, and the impls are
squatting in the megafile because of it.

Within the home, split by ROLE — `render` (internal state → AST) · `lookup` (find a binding) ·
`verbs` (the `*-of` API surface) · `expand` (macroexpand). Same shape as `collection`(6),
`numeric`(5), `declare`(5).

## ★ THE BLAST RADIUS IS TINY, AND THAT IS THE SURPRISE

```
external call sites: 2      src/intrinsic/reflect.rs 1 · tests/reflection/… 1
```

Two. The block is almost entirely reached through `runtime.rs`'s own dispatch, so nearly all
re-pointing is in-file. ⚠ **A near-zero external surface is a reason to move it NOW** — every later
stone that touches `runtime.rs` re-shifts these lines.

## ★ THE PREDICTION — falsifiable

```
runtime.rs      29,757  ->  ~27,300   (-2,450)
src/reflect/    ~2,500 lines, 4-5 files, split by ROLE
require_bundle  STAYS in runtime.rs — 1 grep, must still be there
external sites  2 re-pointed; the rest in-file
behaviour       every introspection verb identical
```

## Out of scope = REJECTED (not deferred)

- **`require_bundle`.** Not reflection; proven by callers. `src/holon/` exists and is where it
  belongs — a later stone, not a by-proximity grab here.
- **The five remaining evaluator modules** — `peer_protocol` (3 ranges, ~3,000) · `stepper` (~1,390) ·
  `defclause_dispatch` · `pattern_matching` · `quasiquote` · `kernel_signal`. ⚠ **Their spans are
  stale too**; each stone re-derives.
- **The facade re-point sweep**, and numeric stone 2's lattice. Untouched.

## THE FOUR QUESTIONS — flat YES/NO

| option | Obvious? | Simple? | Honest? | Good UX? | verdict |
|---|:---:|:---:|:---:|:---:|---|
| **`src/reflect/`, split by role, `require_bundle` stays** | YES | YES | YES | YES | ✅ **ADMITTED** |
| take the whole 8096–10646 span including `require_bundle` | YES | YES | **NO** | — | ⛔ **DISQUALIFIED** |
| put the impls in `src/intrinsic/reflect.rs` | **NO** | YES | **NO** | — | ⛔ **DISQUALIFIED** |
| one flat `src/reflect.rs` of 2,500 lines | YES | **NO** | YES | — | ⛔ **DISQUALIFIED** |
| take `peer_protocol` first because it is bigger | YES | **NO** | YES | — | ⛔ **DISQUALIFIED** |

- **whole-span Honest? NO** — it moves a holon helper into a reflection home by proximity, which is
  the exact defect this campaign has hit four times.
- **impls-into-the-edge Obvious? NO / Honest? NO** — collapses edge and impl, the boundary the
  builder corrected the orchestrator on. Seven existing pairs are the counter-example.
- **one-flat-file Simple? NO** — relocates the megafile rather than solving it.
- **`peer_protocol`-first Simple? NO** — three non-contiguous ranges against one clean block, and
  its home is contested (`src/kernel/` already holds `peer.rs`). Bigger, and not simpler.

## Acceptance

| what | command | expected |
|---|---|---|
| ★ `require_bundle` did NOT move | `grep -c "fn require_bundle" src/runtime.rs` | **1** |
| the megafile sheds it | `wc -l src/runtime.rs` | ~27,300, from 29,757 |
| split by role | `ls src/reflect/` | render · lookup · verbs · expand · mod |
| ★ no facade imports | each file's `use` block | `crate::value::` direct, never via `crate::runtime` |
| the impl does not know its edge | `grep -c "crate::intrinsic" src/reflect/*.rs` | 0 |
| behaviour unchanged | every `*-of` verb, `macroexpand` | identical |
| floor | `scripts/floor.sh`, exit read UNPIPED | 5114/5114, 0 failed |
| clippy | `cargo clippy --release --all-targets -- -D warnings` | 0 |
